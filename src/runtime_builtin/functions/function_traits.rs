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

/// Represents a function call with its arguments, name and source span.
#[derive(Debug, Clone, Serialize)]
pub struct FunctionCall {
    pub args: Vec<PreExp>,
    pub name: String,
    span: InputSpan,
}

impl FunctionCall {
    /// Creates a new FunctionCall instance.
    ///
    /// # Arguments
    /// * `args` - Vector of PreExp arguments to the function
    /// * `name` - Name of the function being called
    /// * `span` - Source code span for error reporting
    pub fn new(args: Vec<PreExp>, name: String, span: Span) -> Self {
        Self {
            args,
            name,
            span: InputSpan::from_span(span),
        }
    }
}

/// The default type check implementation, it performs type checking of function arguments against expected types.
///
/// # Arguments
/// * `args` - The actual arguments passed to the function
/// * `expected` - Vector of expected argument names and their types
/// * `context` - Type checker context
/// * `fn_context` - Function context containing function definitions
///
/// # Returns
/// * `Ok(())` if type checking succeeds
/// * `Err(TransformError)` if there's a type mismatch
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
            }
            .add_span(arg.span()));
        }
    }

    Ok(())
}

/// Creates a type error for wrong argument types.
///
/// # Arguments
/// * `args` - The actual arguments passed
/// * `fun` - The function being called
/// * `context` - Type checker context
/// * `fn_context` - Function context
pub fn default_wrong_type(
    args: &[PreExp],
    fun: &dyn RoocFunction,
    context: &TypeCheckerContext,
    fn_context: &FunctionContext,
) -> TransformError {
    let type_signature = fun.type_signature(args, context, fn_context);
    let args = args.to_owned();
    TransformError::WrongFunctionSignature {
        signature: type_signature,
        got: args
            .iter()
            .map(|a| a.get_type(context, fn_context))
            .collect(),
    }
}

/// Creates an error for wrong number of arguments.
///
/// # Arguments
/// * `fun` - The function being called
/// * `args` - The arguments passed
/// * `context` - Type checker context
/// * `fn_context` - Function context
pub fn default_wrong_number_of_arguments(
    fun: &dyn RoocFunction,
    args: &[PreExp],
    fn_context: &FunctionContext,
) -> TransformError {
    TransformError::WrongNumberOfArguments {
        signature: fun.type_signature(args, &TypeCheckerContext::default(), fn_context),
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
            .function(&self.name)
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
        if let Some(f) = fn_context.function(&self.name) {
            let return_type = f.return_type(&self.args, context, fn_context);
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

/// Converts a function call to LaTeX format.
///
/// # Arguments
/// * `function` - The function call to convert
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

/// Converts a function call to a string representation.
///
/// # Arguments
/// * `function` - The function call to convert
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

/// Trait defining the interface for Rooc functions.
///
/// This trait must be implemented by all functions that can be called within the Rooc language.
pub trait RoocFunction: Debug {
    /// Executes the function with given arguments.
    ///
    /// # Arguments
    /// * `args` - Vector of arguments to the function
    /// * `context` - Transformer context
    /// * `fn_context` - Function context
    ///
    /// # Returns
    /// * `Ok(Primitive)` containing the function result
    /// * `Err(TransformError)` if execution fails
    fn call(
        &self,
        args: &[PreExp],
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError>;

    /// Returns the type signature of the function.
    fn type_signature(
        &self,
        args: &[PreExp],
        context: &TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Vec<(String, PrimitiveKind)>;

    /// Determines the return type of the function given its arguments.
    ///
    /// # Arguments
    /// * `args` - The arguments to the function
    /// * `context` - Type checker context
    /// * `fn_context` - Function context
    fn return_type(
        &self,
        args: &[PreExp],
        context: &TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> PrimitiveKind;

    /// Returns the name of the function.
    fn function_name(&self) -> String;

    /// Type checks the function call.
    ///
    /// # Arguments
    /// * `args` - The arguments to check
    /// * `context` - Type checker context
    /// * `fn_context` - Function context
    fn type_check(
        &self,
        args: &[PreExp],
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        default_type_check(
            args,
            &self.type_signature(args, context, fn_context),
            context,
            fn_context,
        )
    }
}
