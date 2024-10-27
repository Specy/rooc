use crate::parser::il::PreExp;
use crate::parser::model_transformer::TransformError;
use crate::parser::model_transformer::TransformerContext;
use crate::runtime_builtin::rooc_std::{std_fn_to_latex, std_fn_to_string};
use crate::traits::{escape_latex, ToLatex};
use crate::type_checker::type_checker_context::{FunctionContext, TypeCheckerContext};
use crate::{
    primitives::{Primitive, PrimitiveKind},
    type_checker::type_checker_context::{TypeCheckable, WithType},
    utils::InputSpan,
};
use core::fmt;
use pest::Span;
use serde::Serialize;
use std::fmt::Debug;
/*TODO
the PreExp should have only the function call which includes the parameters expressions and the function name
the actual function (which can be kept like this trait, but implemented as a struct, only the `call` method should be implemented)
should be saved inside the transformer context, this way there can also be user defined functions, and functions builtin
perhaps i could add instance functions to objects too
 */

#[derive(Debug, Clone, Serialize)]
pub struct FunctionCall {
    pub args: Vec<PreExp>,
    pub name: String,
    span: InputSpan,
}
impl FunctionCall {
    pub fn new(args: Vec<PreExp>, name: String, span: Span) -> Self {
        Self {
            args,
            name,
            span: InputSpan::from_span(span),
        }
    }
}

pub fn default_type_check(
    args: &[PreExp],
    expected: &[(String, PrimitiveKind)],
    context: &mut TypeCheckerContext,
    fn_context: &FunctionContext,
) -> Result<(), TransformError> {
    let type_signature = expected;
    if type_signature.len() != args.len() {
        return Err(TransformError::WrongNumberOfArguments {
            signature: type_signature.to_owned(),
            args: args.to_owned(),
        });
    }
    for (arg, (_, kind)) in args.iter().zip(type_signature) {
        if kind == &PrimitiveKind::Any {
            continue;
        }
        let arg_type = arg.get_type(context, fn_context);
        //allow any if they are both iterable and expected is iterable of any
        if *kind == PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)) && arg_type.is_iterable()
        {
            continue;
        }
            //allow anything that can be converted to a number
        if kind == &PrimitiveKind::Number && arg_type.is_numeric() {
            continue;
        }
        if kind == &PrimitiveKind::Integer {
            //allow anything that can be converted to a boolean
            if matches!(
                arg_type,
                PrimitiveKind::Integer | PrimitiveKind::PositiveInteger
            ) {
                continue;
            }
        }
        if arg_type != *kind {
            return Err(TransformError::WrongArgument {
                expected: kind.clone(),
                got: arg_type,
            }.add_span(arg.get_span()));
        }
    }

    Ok(())
}

pub fn default_wrong_type(
    args: &[PreExp],
    fun: &dyn RoocFunction,
    context: &TypeCheckerContext,
    fn_context: &FunctionContext,
) -> TransformError {
    let type_signature = fun.get_type_signature();
    let args = args.to_owned();
    TransformError::WrongFunctionSignature {
        signature: type_signature,
        got: args
            .iter()
            .map(|a| a.get_type(context, fn_context))
            .collect(),
    }
}

pub fn default_wrong_number_of_arguments(fun: &dyn RoocFunction) -> TransformError {
    TransformError::WrongNumberOfArguments {
        signature: fun.get_type_signature(),
        args: vec![],
    }
}

impl TypeCheckable for FunctionCall {
    fn type_check(
        &self,
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        for arg in &self.args {
            arg.type_check(context, fn_context)
                .map_err(|e| e.add_span(&self.span))?;
        }
        let f = fn_context
            .get_function(&self.name)
            .ok_or_else(|| TransformError::NonExistentFunction(self.name.clone()))?;
        f.type_check(&self.args, context, fn_context)
            .map_err(|e| e.add_span(&self.span))
    }

    fn populate_token_type_map(
        &self,
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) {
        self.args
            .iter()
            .for_each(|arg| arg.populate_token_type_map(context, fn_context));
        if let Some(f) = fn_context.get_function(&self.name) {
            let return_type = f.get_return_type(&self.args, context, fn_context);
            context.add_token_type_or_undefined(return_type, self.span.clone(), None);
        }
    }
}

impl fmt::Display for FunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let builtin = std_fn_to_string(self);
        if let Some(string) = builtin {
            return write!(f, "{}", string);
        }
        write!(f, "{}", default_rooc_function_to_string(self))
    }
}

impl ToLatex for FunctionCall {
    fn to_latex(&self) -> String {
        //some builtin functions have
        std_fn_to_latex(self).unwrap_or(default_rooc_function_to_latex(self))
    }
}

pub fn default_rooc_function_to_latex(function: &FunctionCall) -> String {
    format!(
        "{}({})",
        escape_latex(&function.name),
        function
            .args
            .iter()
            .map(|p| p.to_latex())
            .collect::<Vec<String>>()
            .join(",\\")
    )
}

pub fn default_rooc_function_to_string(function: &FunctionCall) -> String {
    format!(
        "{}({})",
        function.name,
        function
            .args
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    )
}

pub trait RoocFunction: Debug {
    fn call(
        &self,
        args: &[PreExp],
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError>;
    fn get_type_signature(&self) -> Vec<(String, PrimitiveKind)>;
    fn get_return_type(
        &self,
        args: &[PreExp],
        context: &TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> PrimitiveKind;
    fn get_function_name(&self) -> String;
    fn type_check(
        &self,
        args: &[PreExp],
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError>;
}
