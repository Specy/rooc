use core::fmt;

use serde::{Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::math::{BinOp, UnOp};
use crate::parser::il::block_functions::{
    BlockFunction, BlockFunctionKind, BlockScopedFunction, BlockScopedFunctionKind,
};
use crate::parser::il::il_problem::{AddressableAccess, CompoundVariable};
use crate::parser::model_transformer::Exp;
use crate::parser::model_transformer::TransformError;
use crate::parser::model_transformer::TransformerContext;
use crate::parser::recursive_set_resolver::recursive_set_resolver;
use crate::primitives::ApplyOp;
use crate::primitives::IterableKind;
use crate::primitives::{Graph, GraphEdge, GraphNode};
use crate::primitives::{Primitive, PrimitiveKind};
use crate::runtime_builtin::FunctionCall;
use crate::traits::{escape_latex, ToLatex};
use crate::type_checker::type_checker_context::{
    FunctionContext, TypeCheckable, TypeCheckerContext, WithType,
};
use crate::utils::{InputSpan, Spanned};

#[derive(Debug, Clone, Serialize)]
pub enum PreExp {
    Primitive(Spanned<Primitive>),
    Abs(InputSpan, Box<PreExp>),
    BlockFunction(Spanned<BlockFunction>),
    Variable(Spanned<String>),
    CompoundVariable(Spanned<CompoundVariable>),
    ArrayAccess(Spanned<AddressableAccess>),
    BlockScopedFunction(Spanned<BlockScopedFunction>),
    FunctionCall(InputSpan, FunctionCall),
    BinaryOperation(Spanned<BinOp>, Box<PreExp>, Box<PreExp>),
    UnaryOperation(Spanned<UnOp>, Box<PreExp>),
}

#[wasm_bindgen(typescript_custom_section)]
const IPreExp: &'static str = r#"
export type SerializedFunctionCall = {
    args: SerializedPreExp[],
    name: string,
    span: InputSpan,
}
export type SerializedPreExp = {span: InputSpan} & (
    {type: "Primitive", value: SerializedPrimitive} |
    {type: "Abs", value: SerializedPreExp} |
    {type: "BlockFunction", value: SerializedBlockFunction} |
    {type: "Variable", value: string} |
    {type: "CompoundVariable", value: SerializedCompoundVariable} |
    {type: "ArrayAccess", value: SerializedAddressableAccess} |
    {type: "BlockScopedFunction", value: SerializedBlockScopedFunction} |
    {type: "FunctionCall", value: SerializedFunctionCall} | 
    {type: "BinaryOperation", value: {
        op: BinOp,
        lhs: SerializedPreExp,
        rhs: SerializedPreExp,
    }} |
    {type: "UnaryOperation", value: {
        op: UnOp,
        exp: SerializedPreExp,
    }}
)
"#;

impl TypeCheckable for PreExp {
    //TODO improve spans
    fn type_check(
        &self,
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        match self {
            Self::FunctionCall(span, fun) => fun
                .type_check(context, fn_context)
                .map_err(|e| e.add_span(span)),
            Self::BinaryOperation(op, lhs, rhs) => {
                lhs.type_check(context, fn_context)?;
                rhs.type_check(context, fn_context)?;
                let lhs_type = lhs.get_type(context, fn_context);
                let rhs_type = rhs.get_type(context, fn_context);
                if !lhs_type.can_apply_binary_op(**op, rhs_type.clone()) {
                    Err(TransformError::from_wrong_binop(
                        **op,
                        lhs_type,
                        rhs_type,
                        op.span().clone(),
                    ))
                } else {
                    Ok(())
                }
            }
            Self::UnaryOperation(op, exp) => {
                exp.type_check(context, fn_context)
                    .map_err(|e| e.add_span(exp.span()))?;
                let exp_type = exp.get_type(context, fn_context);
                if !exp_type.can_apply_unary_op(**op) {
                    Err(TransformError::from_wrong_unop(
                        **op,
                        exp_type,
                        op.span().clone(),
                    ))
                } else {
                    Ok(())
                }
            }
            Self::Primitive(_) => Ok(()),
            Self::Abs(_, exp) => {
                exp.type_check(context, fn_context)
                    .map_err(|e| e.add_span(exp.span()))?;
                let exp_type = exp.get_type(context, fn_context);
                if !exp_type.is_numeric() {
                    return Err(TransformError::from_wrong_type(
                        exp_type,
                        PrimitiveKind::Number,
                        exp.span().clone(),
                    ));
                }
                Ok(())
            }
            Self::Variable(name) => {
                //check if the variable is declared, if not, check if it will be declared at runtime
                //this is possible for named variables in the domain
                match context.value_of(name) {
                    Some(_) => Ok(()),
                    None => match context.static_domain_variable_of(name) {
                        Some(_) => Ok(()),
                        None => Err(TransformError::UndeclaredVariable(
                            name.value().clone(),
                        )),
                    }
                    .map_err(|e| e.add_span(name.span())),
                }
            }
            Self::CompoundVariable(c) => context
                .check_compound_variable(&c.indexes, fn_context)
                .map_err(|e| e.add_span(c.span())),
            Self::BlockFunction(f) => {
                for exp in &f.exps {
                    exp.type_check(context, fn_context)
                        .map_err(|e| e.add_span(f.span()))?;
                    let exp_type = exp.get_type(context, fn_context);
                    if !exp_type.is_numeric() {
                        return Err(TransformError::from_wrong_type(
                            PrimitiveKind::Number,
                            exp_type,
                            exp.span().clone(),
                        )
                        .add_span(f.span()));
                    }
                }
                Ok(())
            }
            Self::BlockScopedFunction(f) => {
                for iter in &f.iters {
                    iter.iterator
                        .type_check(context, fn_context)
                        .map_err(|e| e.add_span(f.span()))?;
                    context.add_scope();
                    let types = iter
                        .variable_types(context, fn_context)
                        .map_err(|e| e.add_span(f.span()))?;
                    for (name, t) in types {
                        context.add_token_type(
                            t,
                            name.span().clone(),
                            Some(name.value().clone()),
                        )?;
                    }
                }
                let res = f.exp.type_check(context, fn_context);
                let exp_type = f.exp.get_type(context, fn_context);
                for _ in &f.iters {
                    context.pop_scope().map_err(|e| e.add_span(f.span()))?;
                }
                if let Err(e) = res {
                    return Err(e.add_span(f.span()));
                }
                if !exp_type.is_numeric() {
                    let err = TransformError::from_wrong_type(
                        PrimitiveKind::Number,
                        exp_type,
                        f.exp.span().clone(),
                    )
                    .add_span(f.span());
                    return Err(err);
                }
                Ok(())
            }
            Self::ArrayAccess(array_access) => context
                .get_addressable_value(array_access, fn_context)
                .map(|_| ())
                .map_err(|e| e.add_span(array_access.span())),
        }
    }
    fn populate_token_type_map(
        &self,
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) {
        match self {
            Self::FunctionCall(_span, fun) => {
                fun.populate_token_type_map(context, fn_context);
            }
            Self::Abs(_, exp) => {
                exp.populate_token_type_map(context, fn_context);
            }
            Self::Primitive(p) => context.add_token_type_or_undefined(
                p.value().get_type(),
                p.span().clone(),
                None,
            ),
            Self::Variable(name) => match context.value_of(name) {
                Some(value) => context.add_token_type_or_undefined(
                    value.clone(),
                    name.span().clone(),
                    Some(name.value().clone()),
                ),
                None => {
                    match context.static_domain_variable_of(name) {
                        Some(_) => {
                            context.add_token_type_or_undefined(
                                PrimitiveKind::Number, //TODO we assume defined variables are numbers, this should be improved to specify this is a runtime variable
                                name.span().clone(),
                                Some(name.value().clone()),
                            )
                        }
                        None => context.add_token_type_or_undefined(
                            PrimitiveKind::Undefined,
                            name.span().clone(),
                            Some(name.value().clone()),
                        ),
                    }
                }
            },
            Self::CompoundVariable(c) => {
                context.add_token_type_or_undefined(
                    PrimitiveKind::Number, //every compound variable must be a number
                    c.span().clone(),
                    None,
                );
                for index in &c.indexes {
                    index.populate_token_type_map(context, fn_context);
                }
            }
            Self::BinaryOperation(_, lhs, rhs) => {
                lhs.populate_token_type_map(context, fn_context);
                rhs.populate_token_type_map(context, fn_context);
            }
            Self::UnaryOperation(_, exp) => {
                exp.populate_token_type_map(context, fn_context);
            }
            Self::ArrayAccess(array_access) => {
                context.add_token_type_or_undefined(
                    context
                        .value_of(&array_access.name)
                        .unwrap_or(&PrimitiveKind::Undefined)
                        .clone(),
                    array_access.span().clone(),
                    Some(array_access.name.to_string()),
                );
                for access in &array_access.accesses {
                    access.populate_token_type_map(context, fn_context);
                }
            }
            Self::BlockFunction(f) => {
                for exp in &f.exps {
                    exp.populate_token_type_map(context, fn_context);
                }
            }
            Self::BlockScopedFunction(f) => {
                for iter in &f.iters {
                    iter.populate_token_type_map(context, fn_context);
                }
                f.exp.populate_token_type_map(context, fn_context);
            }
        }
    }
}

impl WithType for PreExp {
    fn get_type(
        &self,
        context: &TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        match self {
            Self::Primitive(p) => p.value().get_type(),
            Self::FunctionCall(_, fun) => {
                let f = fn_context.function(&fun.name);
                if f.is_none() {
                    return PrimitiveKind::Undefined;
                }

                f.unwrap().return_type(&fun.args, context, fn_context)
            }
            Self::Variable(name) => {
                match context.value_of(name) {
                    Some(value) => value.clone(),
                    None => {
                        match context.static_domain_variable_of(name) {
                            Some(_) => PrimitiveKind::Number, //TODO we assume defined variables are numbers, this should be improved to specify this is a runtime variable, currently doesn't error out in type checking as function arguments
                            None => PrimitiveKind::Undefined,
                        }
                    }
                }
            }
            Self::BinaryOperation(_, lhs, _) => lhs.get_type(context, fn_context),
            Self::UnaryOperation(_, exp) => exp.get_type(context, fn_context),
            Self::Abs(_, exp) => exp.get_type(context, fn_context),
            Self::ArrayAccess(a) => context
                .get_addressable_value(a, fn_context)
                .unwrap_or(PrimitiveKind::Undefined),
            Self::BlockFunction(_) => PrimitiveKind::Number, //TODO check if this is true always
            Self::BlockScopedFunction(_) => PrimitiveKind::Number, //TODO check if this is true always
            Self::CompoundVariable(_) => PrimitiveKind::Number, //TODO check if this is true always
        }
    }
}

impl PreExp {
    pub fn to_boxed(self) -> Box<PreExp> {
        Box::new(self)
    }
    pub fn span(&self) -> &InputSpan {
        match self {
            Self::Primitive(n) => n.span(),
            Self::Abs(span, _) => span,
            Self::BlockFunction(f) => f.span(),
            Self::Variable(name) => name.span(),
            Self::CompoundVariable(c) => c.span(),
            Self::BinaryOperation(op, _, _) => op.span(),
            Self::UnaryOperation(op, _) => op.span(),
            Self::ArrayAccess(array_access) => array_access.span(),
            Self::BlockScopedFunction(function) => function.span(),
            Self::FunctionCall(span, _) => span,
        }
    }
    pub fn into_exp(
        &self,
        context: &mut TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Exp, TransformError> {
        match self {
            Self::BinaryOperation(op, lhs, rhs) => {
                let lhs = lhs
                    .into_exp(context, fn_context)
                    .map_err(|e| e.add_span(self.span()))?;
                let rhs = rhs
                    .into_exp(context, fn_context)
                    .map_err(|e| e.add_span(self.span()))?;
                Ok(Exp::BinOp(**op, lhs.to_box(), rhs.to_box()))
            }
            Self::Primitive(n) => match n.as_number_cast() {
                Ok(n) => Ok(Exp::Number(n)),
                Err(e) => Err(e.add_span(self.span())),
            },
            Self::Abs(span, exp) => {
                let inner = exp
                    .into_exp(context, fn_context)
                    .map_err(|e| e.add_span(span))?;
                Ok(Exp::Abs(inner.to_box()))
            }
            Self::BlockFunction(f) => {
                let mut parsed_exp = f
                    .exps
                    .iter()
                    .map(|exp| exp.into_exp(context, fn_context))
                    .collect::<Result<Vec<Exp>, TransformError>>()
                    .map_err(|e| e.add_span(self.span()))?;
                match f.kind {
                    BlockFunctionKind::Min => Ok(Exp::Min(parsed_exp)),
                    BlockFunctionKind::Max => Ok(Exp::Max(parsed_exp)),
                    BlockFunctionKind::Avg => {
                        let len = parsed_exp.len();
                        let mut sum = parsed_exp.pop().unwrap_or(Exp::Number(0.0));
                        for exp in parsed_exp.into_iter().rev() {
                            sum = Exp::BinOp(BinOp::Add, exp.to_box(), sum.to_box());
                        }
                        Ok(Exp::BinOp(
                            BinOp::Div,
                            sum.to_box(),
                            Exp::Number(len as f64).to_box(),
                        ))
                    }
                }
            }

            Self::UnaryOperation(op, exp) => {
                let inner = exp
                    .into_exp(context, fn_context)
                    .map_err(|e| e.add_span(self.span()))?;
                Ok(Exp::UnOp(**op, inner.to_box()))
            }
            Self::Variable(name) => {
                let value = context.value(name).map(|v| match v.as_number_cast() {
                    Ok(n) => Ok(Exp::Number(n)),
                    Err(e) => Err(e.add_span(self.span())),
                });
                match value {
                    Some(value) => Ok(value?),
                    None => {
                        context
                            .increment_domain_variable_usage(name)
                            .map_err(|e| e.add_span(self.span()))?;
                        Ok(Exp::Variable(name.value().clone()))
                    }
                }
            }
            Self::CompoundVariable(c) => {
                let indexes = &c
                    .indexes
                    .iter()
                    .map(|v| v.as_primitive(context, fn_context))
                    .collect::<Result<Vec<Primitive>, TransformError>>()
                    .map_err(|e| e.add_span(self.span()))?;
                let name = context
                    .flatten_compound_variable(&c.name, indexes)
                    .map_err(|e| e.add_span(self.span()))?;
                context
                    .increment_domain_variable_usage(&name)
                    .map_err(|e| e.add_span(self.span()))?;
                Ok(Exp::Variable(name))
            }
            Self::ArrayAccess(array_access) => {
                let value = context
                    .addressable_value(array_access, fn_context)
                    .map_err(|e| e.add_span(self.span()))?;
                match value.as_number_cast() {
                    Ok(n) => Ok(Exp::Number(n)),
                    Err(e) => Err(e.add_span(self.span())),
                }
            }
            Self::BlockScopedFunction(f) => {
                let mut results = Vec::new();
                recursive_set_resolver(
                    &f.iters,
                    context,
                    fn_context,
                    &mut results,
                    0,
                    &|context| {
                        let inner = f
                            .exp
                            .into_exp(context, fn_context)
                            .map_err(|e| e.add_span(self.span()))?;
                        Ok(inner)
                    },
                )
                .map_err(|e| e.add_span(self.span()))?;
                match f.kind {
                    BlockScopedFunctionKind::Sum => {
                        let mut sum = results.pop().unwrap_or(Exp::Number(0.0));
                        for result in results.into_iter().rev() {
                            sum = Exp::BinOp(BinOp::Add, result.to_box(), sum.to_box());
                        }
                        Ok(sum)
                    }
                    BlockScopedFunctionKind::Prod => {
                        let mut prod = results.pop().unwrap_or(Exp::Number(1.0));
                        for result in results.into_iter().rev() {
                            prod = Exp::BinOp(BinOp::Mul, result.to_box(), prod.to_box());
                        }
                        Ok(prod)
                    }
                    BlockScopedFunctionKind::Min => Ok(Exp::Min(results)),
                    BlockScopedFunctionKind::Max => Ok(Exp::Max(results)),
                    BlockScopedFunctionKind::Avg => {
                        let len = results.len();
                        let mut sum = results.pop().unwrap_or(Exp::Number(0.0));
                        for result in results.into_iter().rev() {
                            sum = Exp::BinOp(BinOp::Add, result.to_box(), sum.to_box());
                        }
                        Ok(Exp::BinOp(
                            BinOp::Div,
                            sum.to_box(),
                            Exp::Number(len as f64).to_box(),
                        ))
                    }
                }
            }
            Self::FunctionCall(span, function) => {
                let f = fn_context.function(&function.name);
                if f.is_none() {
                    return Err(
                        TransformError::NonExistentFunction(function.name.clone()).add_span(span)
                    );
                }
                let f = f.unwrap();
                let value = f
                    .call(&function.args, context, fn_context)
                    .map_err(|e| e.add_span(span))?;
                match value.as_number_cast() {
                    Ok(n) => Ok(Exp::Number(n)),
                    Err(e) => Err(e.add_span(self.span())),
                }
            }
        }
    }

    pub fn as_static_primitive(&self) -> Option<Primitive> {
        match self {
            Self::Primitive(p) => Some(p.value().clone()),
            _ => None,
        }
    }
    pub fn as_primitive(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        match self {
            PreExp::Primitive(p) => Ok(p.value().clone()),
            PreExp::Variable(s) => match context.value(s) {
                Some(value) => Ok(value.clone()),
                None => match context.variable_domain(s) {
                    None => Err(TransformError::UndeclaredVariable(
                        s.value().clone(),
                    )),
                    Some(_) => Err(
                        //TODO create a specific error for this
                        TransformError::Other(
                            format!("Variable \"{}\" is a domain variable and cannot be used inside expression valuation", s.value())
                        )
                    )
                },
            },
            PreExp::CompoundVariable(c) => {
                let indexes = &c.compute_indexes(context, fn_context)?;
                let name = context.flatten_compound_variable(&c.name, indexes)?;
                match context.value(&name) {
                    Some(value) => Ok(value.clone()),
                    None => match context.variable_domain(&name) {
                        None => Err(TransformError::UndeclaredVariable(
                            name.clone(),
                        )),
                        Some(_) => Err(
                            //TODO create a specific error for this
                            TransformError::Other(
                                format!("Variable \"{}\" is a domain variable and cannot be used inside expression valuation", name)
                            )
                        )
                    },
                }
            }
            PreExp::FunctionCall(_, fun) => {
                let f = fn_context.function(&fun.name).ok_or_else(|| {
                    TransformError::NonExistentFunction(fun.name.clone())
                })?;
                let value = f.call(&fun.args, context, fn_context)?;
                Ok(value)
            }
            PreExp::ArrayAccess(a) => {
                let value = context.addressable_value(a, fn_context)?;
                Ok(value)
            }
            PreExp::UnaryOperation(op, v) => {
                let value = v.as_primitive(context, fn_context)?;
                match value.apply_unary_op(**op) {
                    Ok(value) => Ok(value),
                    Err(_) => Err(TransformError::from_wrong_unop(
                        **op,
                        value.get_type(),
                        op.span().clone(),
                    )),
                }
            }
            PreExp::BinaryOperation(op, lhs, rhs) => {
                let lhs = lhs.as_primitive(context, fn_context)?;
                let rhs = rhs.as_primitive(context, fn_context)?;
                match lhs.apply_binary_op(**op, &rhs) {
                    Ok(value) => Ok(value),
                    Err(_) => Err(TransformError::from_wrong_binop(
                        **op,
                        lhs.get_type(),
                        rhs.get_type(),
                        op.span().clone(),
                    )),
                }
            }
            PreExp::Abs(_, _) | PreExp::BlockFunction(_) | PreExp::BlockScopedFunction(_) => {
                //TODO is this correct?
                Err(TransformError::WrongArgument {
                    got: PrimitiveKind::Undefined,
                    expected: PrimitiveKind::Any,
                })
            }
        }
    }
    //TODO make this a macro
    pub fn as_number(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<f64, TransformError> {
        self.as_primitive(context, fn_context)
            .map(|p| p.as_number())
            .map_err(|e| e.add_span(self.span()))?
    }
    pub fn as_number_cast(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<f64, TransformError> {
        self.as_primitive(context, fn_context)
            .map(|p| p.as_number_cast())
            .map_err(|e| e.add_span(self.span()))?
    }
    pub fn as_integer(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<i64, TransformError> {
        self.as_primitive(context, fn_context)
            .map(|p| p.as_integer())
            .map_err(|e| e.add_span(self.span()))?
    }
    pub fn as_integer_cast(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<i64, TransformError> {
        self.as_primitive(context, fn_context)
            .map(|p| p.as_integer_cast())
            .map_err(|e| e.add_span(self.span()))?
    }
    pub fn as_positive_integer(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<u64, TransformError> {
        self.as_primitive(context, fn_context)
            .map(|p| p.as_positive_integer())
            .map_err(|e| e.add_span(self.span()))?
    }
    pub fn as_usize(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<usize, TransformError> {
        self.as_primitive(context, fn_context)
            .map(|p| p.as_usize().map(|v| v.to_owned()))
            .map_err(|e| e.add_span(self.span()))?
    }
    pub fn as_usize_cast(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<usize, TransformError> {
        self.as_primitive(context, fn_context)
            .map(|p| p.as_usize_cast().map(|v| v.to_owned()))
            .map_err(|e| e.add_span(self.span()))?
    }
    pub fn as_string(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<String, TransformError> {
        self.as_primitive(context, fn_context)
            .map(|p| p.as_string().map(|v| v.to_owned()))
            .map_err(|e| e.add_span(self.span()))?
    }
    pub fn as_boolean(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<bool, TransformError> {
        self.as_primitive(context, fn_context)
            .map(|p| p.as_boolean())
            .map_err(|e| e.add_span(self.span()))?
    }
    pub fn as_graph(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Graph, TransformError> {
        self.as_primitive(context, fn_context)
            .map(|p| p.as_graph().map(|v| v.to_owned()))
            .map_err(|e| e.add_span(self.span()))?
    }
    pub fn as_node(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<GraphNode, TransformError> {
        self.as_primitive(context, fn_context)
            .map(|p| p.as_graph_node().map(|v| v.to_owned()))
            .map_err(|e| e.add_span(self.span()))?
    }
    pub fn as_edge(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<GraphEdge, TransformError> {
        self.as_primitive(context, fn_context)
            .map(|p| p.as_graph_edge().map(|v| v.to_owned()))
            .map_err(|e| e.add_span(self.span()))?
    }

    pub fn as_iterator(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<IterableKind, TransformError> {
        self.as_primitive(context, fn_context)
            .map(|p| p.as_iterator().map(|v| v.to_owned()))
            .map_err(|e| e.add_span(self.span()))?
    }

    pub(crate) fn is_leaf(&self) -> bool {
        !matches!(
            self,
            Self::BinaryOperation(_, _, _) | Self::UnaryOperation(_, _)
        )
    }
    fn to_string_with_precedence(&self, previous_precedence: u8) -> String {
        match self {
            Self::BinaryOperation(op, lhs, rhs) => {
                //TODO add implied multiplication like 2x 2(x + y) etc...
                /*
                   implicit_mul = {
                       (number | parenthesis | modulo){2,} ~ variable? |
                       (number | parenthesis | modulo) ~ variable
                   }
                */
                let lhs_str = lhs.to_string_with_precedence(op.precedence());
                let rhs_str = rhs.to_string_with_precedence(op.precedence());
                if op.precedence() < previous_precedence {
                    format!("({} {} {})", lhs_str, **op, rhs_str)
                } else {
                    format!("{} {} {}", lhs_str, **op, rhs_str)
                }
            }
            _ => self.to_string(),
        }
    }
    fn to_latex_with_precedence(&self, previous_precedence: u8) -> String {
        match self {
            Self::BinaryOperation(op, lhs, rhs) => {
                //TODO add implied multiplication like 2x 2(x + y) etc...
                /*
                   implicit_mul = {
                       (number | parenthesis | modulo){2,} ~ variable? |
                       (number | parenthesis | modulo) ~ variable
                   }
                */
                let lhs_str = lhs.to_latex_with_precedence(op.precedence());
                let rhs_str = rhs.to_latex_with_precedence(op.precedence());

                if op.precedence() < previous_precedence {
                    format!("({} {} {})", lhs_str, op.to_latex(), rhs_str)
                } else {
                    format!("{} {} {}", lhs_str, op.to_latex(), rhs_str)
                }
            }
            _ => self.to_latex(),
        }
    }
}

impl ToLatex for PreExp {
    fn to_latex(&self) -> String {
        match self {
            Self::ArrayAccess(a) => a.to_latex(),
            Self::BlockFunction(f) => f.to_latex(),
            Self::BlockScopedFunction(f) => f.to_latex(),
            Self::BinaryOperation(op, lhs, rhs) => {
                let rhs = rhs.to_latex_with_precedence(op.precedence());
                let lhs = lhs.to_latex_with_precedence(op.precedence());
                match op.value() {
                    BinOp::Div => format!("\\frac{{{}}}{{{}}}", lhs, rhs),
                    _ => format!("{} {} {}", lhs, op.to_latex(), rhs),
                }
            }
            Self::UnaryOperation(op, exp) => {
                if self.is_leaf() {
                    format!("{}{}", op.to_latex(), exp.to_latex())
                } else {
                    format!("{}({})", op.to_latex(), exp.to_latex())
                }
            }
            Self::Variable(name) => escape_latex(name.value()),
            Self::Primitive(p) => p.to_latex(),
            Self::Abs(_, exp) => format!("|{}|", exp.to_latex()),
            Self::CompoundVariable(c) => c.to_latex(),
            Self::FunctionCall(_, f) => f.to_latex(),
        }
    }
}

impl fmt::Display for PreExp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::ArrayAccess(a) => a.to_string(),
            Self::BlockFunction(f) => f.to_string(),
            Self::BlockScopedFunction(f) => f.to_string(),
            Self::BinaryOperation(op, lhs, rhs) => {
                let rhs = rhs.to_string_with_precedence(op.precedence());
                let lhs = lhs.to_string_with_precedence(op.precedence());
                format!("{} {} {}", lhs, **op, rhs)
            }
            Self::CompoundVariable(c) => c.to_string(),
            Self::FunctionCall(_, f) => f.to_string(),
            Self::Abs(_, exp) => format!("|{}|", **exp),
            Self::Primitive(p) => p.to_string(),
            Self::UnaryOperation(op, exp) => {
                if self.is_leaf() {
                    format!("{}{}", **op, **exp)
                } else {
                    format!("{}({})", **op, **exp)
                }
            }
            Self::Variable(name) => {
                if name.contains('_') {
                    //in case this is a escaped variable
                    format!("\\{}", **name)
                } else {
                    name.to_string()
                }
            }
        };
        f.write_str(&s)
    }
}

impl PreExp {
    pub fn get_span_wasm(&self) -> InputSpan {
        self.span().clone()
    }
}
