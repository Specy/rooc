use core::fmt;

#[allow(unused_imports)]
use crate::prelude::*;
use serde::Serialize;

use crate::math::{BinOp, OperandShape, OperandSide, UnOp};
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
use crate::traits::{ToLatex, escape_latex};
use crate::type_checker::type_checker_context::{
    CompoundFamilyKind, FunctionContext, TypeCheckable, TypeCheckerContext, WithType,
};
use crate::utils::{InputSpan, Spanned, fold_balanced};

/// Resolves the statically known kind of a compound variable from its
/// declared family, falling back to Number when nothing is known.
fn statically_flatten_compound_variable(compound: &CompoundVariable) -> Option<String> {
    let indexes = compound
        .indexes
        .iter()
        .map(|index| match index {
            PreExp::Primitive(value) => match value.value() {
                Primitive::Number(value) => Some(value.to_string()),
                Primitive::Integer(value) => Some(value.to_string()),
                Primitive::PositiveInteger(value) => Some(value.to_string()),
                Primitive::Boolean(value) => Some(if *value { "T" } else { "F" }.to_string()),
                Primitive::String(value) => Some(value.clone()),
                Primitive::GraphNode(value) => Some(value.name().clone()),
                Primitive::Iterable(_)
                | Primitive::Graph(_)
                | Primitive::GraphEdge(_)
                | Primitive::Tuple(_)
                | Primitive::Undefined => None,
            },
            PreExp::BlockFunction(_)
            | PreExp::Variable(_)
            | PreExp::CompoundVariable(_)
            | PreExp::ArrayAccess(_)
            | PreExp::BlockScopedFunction(_)
            | PreExp::FunctionCall(_, _)
            | PreExp::BinaryOperation(_, _, _)
            | PreExp::UnaryOperation(_, _) => None,
        })
        .collect::<Option<Vec<_>>>()?;
    Some(format!("{}_{}", compound.name, indexes.join("_")))
}

fn compound_variable_kind(
    context: &TypeCheckerContext,
    compound: &CompoundVariable,
) -> PrimitiveKind {
    match context.static_compound_variable_of(&compound.name, compound.indexes.len()) {
        Some(CompoundFamilyKind::Uniform(kind)) => kind.clone(),
        //the family has declarations with different types, the exact type of
        //a member is only known at runtime
        Some(CompoundFamilyKind::Mixed) => PrimitiveKind::Any,
        // No family: a fully literal reference may flatten onto one escaped
        // declaration, such as x_1 onto \x_1.
        None => statically_flatten_compound_variable(compound)
            .and_then(|name| context.static_domain_variable_of(&name))
            .map(|static_type| static_type.value.static_primitive_kind())
            .unwrap_or(PrimitiveKind::Number),
    }
}

/// Adds up a list of expressions, zero when the list is empty. The sum is
/// built as a balanced tree, so that a sum over many terms stays shallow.
fn fold_sum(exps: Vec<Exp>) -> Exp {
    fold_balanced(exps, |lhs, rhs| {
        Exp::BinOp(BinOp::Add, lhs.to_box(), rhs.to_box())
    })
    .unwrap_or(Exp::Number(0.0))
}

/// Multiplies a list of expressions, one when the list is empty. Like
/// [`fold_sum`], the product is built as a balanced tree.
fn fold_prod(exps: Vec<Exp>) -> Exp {
    fold_balanced(exps, |lhs, rhs| {
        Exp::BinOp(BinOp::Mul, lhs.to_box(), rhs.to_box())
    })
    .unwrap_or(Exp::Number(1.0))
}

/// Folds a list of expressions into a balanced tree of binary exclusive
/// disjunctions, which computes the parity (odd number of true values) of the
/// whole list.
fn fold_xor(exps: Vec<Exp>) -> Exp {
    fold_balanced(exps, |lhs, rhs| Exp::Xor(lhs.to_box(), rhs.to_box())).unwrap_or(Exp::Number(0.0))
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
/// Represents an expression in the intermediate language before final transformation.
///
/// This enum captures different types of expressions that can appear in the source code,
/// including primitives, variables, function calls, and mathematical operations.
pub enum PreExp {
    /// A primitive value like numbers, strings, booleans
    Primitive(Spanned<Primitive>),
    /// A block function like average, min, max, etc.
    BlockFunction(Spanned<BlockFunction>),
    /// A simple variable reference like 'x'
    Variable(Spanned<String>),
    /// A compound variable like 'x_y'
    CompoundVariable(Spanned<CompoundVariable>),
    /// Array access expression like `arr[i]`
    ArrayAccess(Spanned<AddressableAccess>),
    /// A block function with a scope, like a summation or a product
    BlockScopedFunction(Spanned<BlockScopedFunction>),
    /// A function call expression
    FunctionCall(InputSpan, FunctionCall),
    /// A binary operation like addition, multiplication
    BinaryOperation(Spanned<BinOp>, Box<PreExp>, Box<PreExp>),
    /// A unary operation like negation
    UnaryOperation(Spanned<UnOp>, Box<PreExp>),
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
const IPreExp: &'static str = r#"
export type SerializedFunctionCall = {
    args: SerializedPreExp[],
    name: string,
    span: InputSpan,
}
export type SerializedPreExp =
    {type: "Primitive", value: SerializedSpanned<SerializedPrimitive>} |
    {type: "BlockFunction", value: SerializedSpanned<SerializedBlockFunction>} |
    {type: "Variable", value: SerializedSpanned<string>} |
    {type: "CompoundVariable", value: SerializedSpanned<SerializedCompoundVariable>} |
    {type: "ArrayAccess", value: SerializedSpanned<SerializedAddressableAccess>} |
    {type: "BlockScopedFunction", value: SerializedSpanned<SerializedBlockScopedFunction>} |
    {type: "FunctionCall", value: [InputSpan, SerializedFunctionCall]} |
    {type: "BinaryOperation", value: [SerializedSpanned<SerializedBinOp>, SerializedPreExp, SerializedPreExp]} |
    {type: "UnaryOperation", value: [SerializedSpanned<SerializedUnOp>, SerializedPreExp]}
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
            Self::Variable(name) => {
                //check if the variable is declared, if not, check if it will be declared at runtime
                //this is possible for named variables in the domain
                match context.value_of(name) {
                    Some(_) => Ok(()),
                    None => match context.static_domain_variable_of(name) {
                        Some(_) => Ok(()),
                        None => Err(TransformError::UndeclaredVariable(name.value().clone())),
                    }
                    .map_err(|e| e.add_span(name.span())),
                }
            }
            Self::CompoundVariable(c) => {
                //a compound variable with no matching family declaration is
                //a guaranteed error at transform time, report it early, unless
                //it can flatten onto a literal (escaped) declaration like \x_1
                if context
                    .static_compound_variable_of(&c.name, c.indexes.len())
                    .is_none()
                    && statically_flatten_compound_variable(c)
                        .and_then(|name| context.static_domain_variable_of(&name))
                        .is_none()
                {
                    return Err(
                        TransformError::UndeclaredVariable(c.to_string()).add_span(c.span())
                    );
                }
                context
                    .check_compound_variable(&c.indexes, fn_context)
                    .map_err(|e| e.add_span(c.span()))
            }
            Self::BlockFunction(f) => {
                if let Some(expected) = f.kind.exact_arity()
                    && f.exps.len() != expected
                {
                    return Err(TransformError::Other(format!(
                        "The {} block takes exactly {} expression, got {}",
                        f.kind,
                        expected,
                        f.exps.len()
                    ))
                    .add_span(f.span()));
                }
                let requires_boolean = f.kind.is_logic();
                for exp in &f.exps {
                    exp.type_check(context, fn_context)
                        .map_err(|e| e.add_span(f.span()))?;
                    let exp_type = exp.get_type(context, fn_context);
                    if requires_boolean {
                        if exp_type != PrimitiveKind::Boolean && !exp_type.is_any() {
                            return Err(TransformError::from_wrong_type(
                                PrimitiveKind::Boolean,
                                exp_type,
                                exp.span().clone(),
                            )
                            .add_span(f.span()));
                        }
                    } else if !exp_type.is_numeric() {
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
                if f.kind.is_logic() {
                    if exp_type != PrimitiveKind::Boolean && !exp_type.is_any() {
                        let err = TransformError::from_wrong_type(
                            PrimitiveKind::Boolean,
                            exp_type,
                            f.exp.span().clone(),
                        )
                        .add_span(f.span());
                        return Err(err);
                    }
                } else if !exp_type.is_numeric() {
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
            Self::ArrayAccess(array_access) => {
                // Type-check each index expression too, so ill-typed indices (wrong
                // arity / non-numeric function calls) are rejected here rather than
                // slipping through to the transform phase.
                for access in array_access.accesses.iter() {
                    access.type_check(context, fn_context)?;
                }
                context
                    .get_addressable_value(array_access, fn_context)
                    .map(|_| ())
                    .map_err(|e| e.add_span(array_access.span()))
            }
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
            Self::Primitive(p) => {
                context.add_token_type_or_undefined(p.value().get_type(), p.span().clone(), None)
            }
            Self::Variable(name) => match context.value_of(name) {
                Some(value) => context.add_token_type_or_undefined(
                    value.clone(),
                    name.span().clone(),
                    Some(name.value().clone()),
                ),
                None => match context.static_domain_variable_of(name) {
                    Some(v) => context.add_token_type_or_undefined(
                        v.value.static_primitive_kind(),
                        name.span().clone(),
                        Some(name.value().clone()),
                    ),
                    None => context.add_token_type_or_undefined(
                        PrimitiveKind::Undefined,
                        name.span().clone(),
                        Some(name.value().clone()),
                    ),
                },
            },
            Self::CompoundVariable(c) => {
                context.add_token_type_or_undefined(
                    compound_variable_kind(context, c),
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
            Self::Variable(name) => match context.value_of(name) {
                Some(value) => value.clone(),
                None => match context.static_domain_variable_of(name) {
                    Some(v) => v.value.static_primitive_kind(),
                    None => PrimitiveKind::Undefined,
                },
            },
            Self::BinaryOperation(op, lhs, rhs) => match **op {
                BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff => {
                    PrimitiveKind::Boolean
                }
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                    let lhs_type = lhs.get_type(context, fn_context);
                    let rhs_type = rhs.get_type(context, fn_context);
                    if lhs_type.is_numeric() && rhs_type.is_numeric() {
                        use PrimitiveKind::*;
                        // Primitive arithmetic dispatches on the left operand:
                        // a Boolean lhs is first cast to f64, while a Boolean
                        // rhs keeps the lhs integer kind except for division.
                        let is_number_result =
                            lhs_type == Number || rhs_type == Number || lhs_type == Boolean;
                        match **op {
                            BinOp::Add | BinOp::Mul => {
                                if is_number_result {
                                    Number
                                } else if lhs_type == Integer || rhs_type == Integer {
                                    Integer
                                } else {
                                    PositiveInteger
                                }
                            }
                            BinOp::Sub => {
                                if is_number_result {
                                    Number
                                } else {
                                    Integer
                                }
                            }
                            BinOp::Div => Number,
                            BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff => {
                                unreachable!("logic operator in arithmetic branch")
                            }
                        }
                    } else {
                        //non numeric operations keep the previous behavior of
                        //using the left hand side type, like string concatenation
                        lhs_type
                    }
                }
            },
            Self::UnaryOperation(op, exp) => match **op {
                UnOp::Not => PrimitiveKind::Boolean,
                UnOp::Neg => match exp.get_type(context, fn_context) {
                    //negating a boolean yields its 0/1 value negated, a number
                    PrimitiveKind::Boolean => PrimitiveKind::Number,
                    other => other,
                },
            },
            Self::ArrayAccess(a) => context
                .get_addressable_value(a, fn_context)
                .unwrap_or(PrimitiveKind::Undefined),
            Self::BlockFunction(f) => match f.kind {
                BlockFunctionKind::All | BlockFunctionKind::Any | BlockFunctionKind::Xor => {
                    PrimitiveKind::Boolean
                }
                BlockFunctionKind::Abs => f
                    .exps
                    .first()
                    .map(|exp| exp.get_type(context, fn_context))
                    .unwrap_or(PrimitiveKind::Undefined),
                BlockFunctionKind::Min | BlockFunctionKind::Max | BlockFunctionKind::Avg => {
                    PrimitiveKind::Number
                }
            },
            Self::BlockScopedFunction(f) => {
                if f.kind.is_logic() {
                    PrimitiveKind::Boolean
                } else {
                    PrimitiveKind::Number
                }
            }
            Self::CompoundVariable(c) => compound_variable_kind(context, c),
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
                Ok(match **op {
                    BinOp::And => Exp::And(vec![lhs, rhs]),
                    BinOp::Or => Exp::Or(vec![lhs, rhs]),
                    BinOp::Xor => Exp::Xor(lhs.to_box(), rhs.to_box()),
                    BinOp::Implies => Exp::Implies(lhs.to_box(), rhs.to_box()),
                    BinOp::Iff => Exp::Iff(lhs.to_box(), rhs.to_box()),
                    op @ (BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div) => {
                        Exp::BinOp(op, lhs.to_box(), rhs.to_box())
                    }
                })
            }
            Self::Primitive(n) => match n.as_number_cast() {
                Ok(n) => Ok(Exp::Number(n)),
                Err(e) => Err(e.add_span(self.span())),
            },
            Self::BlockFunction(f) => {
                let mut parsed_exp = f
                    .exps
                    .iter()
                    .map(|exp| exp.into_exp(context, fn_context))
                    .collect::<Result<Vec<Exp>, TransformError>>()
                    .map_err(|e| e.add_span(self.span()))?;
                match f.kind {
                    BlockFunctionKind::Abs => {
                        let inner = parsed_exp.pop().ok_or_else(|| {
                            TransformError::Other(
                                "The abs block takes exactly one expression, got 0".to_string(),
                            )
                            .add_span(self.span())
                        })?;
                        if !parsed_exp.is_empty() {
                            return Err(TransformError::Other(format!(
                                "The abs block takes exactly one expression, got {}",
                                parsed_exp.len() + 1
                            ))
                            .add_span(self.span()));
                        }
                        Ok(Exp::Abs(inner.to_box()))
                    }
                    BlockFunctionKind::Min => Ok(Exp::Min(parsed_exp)),
                    BlockFunctionKind::Max => Ok(Exp::Max(parsed_exp)),
                    BlockFunctionKind::Avg => {
                        let len = parsed_exp.len();
                        Ok(Exp::BinOp(
                            BinOp::Div,
                            fold_sum(parsed_exp).to_box(),
                            Exp::Number(len as f64).to_box(),
                        ))
                    }
                    BlockFunctionKind::All => Ok(Exp::And(parsed_exp)),
                    BlockFunctionKind::Any => Ok(Exp::Or(parsed_exp)),
                    BlockFunctionKind::Xor => Ok(fold_xor(parsed_exp)),
                }
            }

            Self::UnaryOperation(op, exp) => {
                let inner = exp
                    .into_exp(context, fn_context)
                    .map_err(|e| e.add_span(self.span()))?;
                Ok(match **op {
                    UnOp::Not => Exp::Not(inner.to_box()),
                    UnOp::Neg => Exp::UnOp(UnOp::Neg, inner.to_box()),
                })
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
                    .compute_indexes(context, fn_context)
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
                    BlockScopedFunctionKind::Sum => Ok(fold_sum(results)),
                    BlockScopedFunctionKind::Prod => Ok(fold_prod(results)),
                    BlockScopedFunctionKind::Min => Ok(Exp::Min(results)),
                    BlockScopedFunctionKind::Max => Ok(Exp::Max(results)),
                    BlockScopedFunctionKind::All => Ok(Exp::And(results)),
                    BlockScopedFunctionKind::Any => Ok(Exp::Or(results)),
                    BlockScopedFunctionKind::Xor => Ok(fold_xor(results)),
                    BlockScopedFunctionKind::Avg => {
                        let len = results.len();
                        Ok(Exp::BinOp(
                            BinOp::Div,
                            fold_sum(results).to_box(),
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
                    None => Err(TransformError::UndeclaredVariable(s.value().clone())),
                    Some(_) => Err(
                        //TODO create a specific error for this
                        TransformError::Other(format!(
                            "Variable \"{}\" is a domain variable and cannot be used inside expression valuation",
                            s.value()
                        )),
                    ),
                },
            },
            PreExp::CompoundVariable(c) => {
                let indexes = &c.compute_indexes(context, fn_context)?;
                let name = context.flatten_compound_variable(&c.name, indexes)?;
                match context.value(&name) {
                    Some(value) => Ok(value.clone()),
                    None => match context.variable_domain(&name) {
                        None => Err(TransformError::UndeclaredVariable(name.clone())),
                        Some(_) => Err(
                            //TODO create a specific error for this
                            TransformError::Other(format!(
                                "Variable \"{}\" is a domain variable and cannot be used inside expression valuation",
                                name
                            )),
                        ),
                    },
                }
            }
            PreExp::FunctionCall(_, fun) => {
                let f = fn_context
                    .function(&fun.name)
                    .ok_or_else(|| TransformError::NonExistentFunction(fun.name.clone()))?;
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
            PreExp::BlockFunction(_) | PreExp::BlockScopedFunction(_) => {
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

    /// How the expression looks to an operator containing it.
    fn shape(&self) -> OperandShape {
        match self {
            Self::BinaryOperation(op, _, _) => OperandShape::Binary(**op),
            Self::UnaryOperation(_, _) => OperandShape::Prefix,
            Self::Primitive(p) => match p.value() {
                //a negative literal is written with a leading minus, like a negation
                Primitive::Number(n) if n.is_sign_negative() => OperandShape::Prefix,
                Primitive::Integer(n) if *n < 0 => OperandShape::Prefix,
                _ => OperandShape::Atom,
            },
            Self::BlockFunction(_)
            | Self::Variable(_)
            | Self::CompoundVariable(_)
            | Self::ArrayAccess(_)
            | Self::BlockScopedFunction(_)
            | Self::FunctionCall(_, _) => OperandShape::Atom,
        }
    }

    /// Like [`Self::shape`], but a division is written as a fraction, whose
    /// bar already groups it.
    fn latex_shape(&self) -> OperandShape {
        match self {
            Self::BinaryOperation(op, _, _) if **op == BinOp::Div => OperandShape::Atom,
            _ => self.shape(),
        }
    }

    /// Formats an operand of `op`, in parentheses when the precedence requires it.
    fn operand_to_string(&self, op: BinOp, side: OperandSide) -> String {
        if op.operand_needs_parens(self.shape(), side) {
            format!("({})", self)
        } else {
            self.to_string()
        }
    }

    /// LaTeX counterpart of [`Self::operand_to_string`].
    fn operand_to_latex(&self, op: BinOp, side: OperandSide, before_product: bool) -> String {
        if op.operand_needs_parens(self.latex_shape(), side) {
            format!("\\left({}\\right)", self.to_latex())
        } else {
            self.latex_before(before_product)
        }
    }

    /// The LaTeX of the expression, `before_product` tells whether a product
    /// continues right after it, which a sum or a product written as a big
    /// operator would otherwise seem to extend over.
    fn latex_before(&self, before_product: bool) -> String {
        match self {
            Self::ArrayAccess(a) => a.to_latex(),
            Self::BlockFunction(f) => f.to_latex(),
            Self::BlockScopedFunction(f) => match f.kind {
                BlockScopedFunctionKind::Sum | BlockScopedFunctionKind::Prod if before_product => {
                    format!("\\left({}\\right)", f.to_latex())
                }
                _ => f.to_latex(),
            },
            Self::BinaryOperation(op, lhs, rhs) => match **op {
                BinOp::Div => format!("\\frac{{{}}}{{{}}}", lhs.to_latex(), rhs.to_latex()),
                _ => format!(
                    "{} {} {}",
                    lhs.operand_to_latex(**op, OperandSide::Left, **op == BinOp::Mul),
                    op.to_latex(),
                    rhs.operand_to_latex(**op, OperandSide::Right, before_product)
                ),
            },
            Self::UnaryOperation(op, exp) => {
                if op.operand_needs_parens(exp.latex_shape()) {
                    format!("{}\\left({}\\right)", op.to_latex(), exp.to_latex())
                } else {
                    format!("{}{}", op.to_latex(), exp.latex_before(before_product))
                }
            }
            Self::Variable(name) => escape_latex(name.value()),
            Self::Primitive(p) => p.to_latex(),
            Self::CompoundVariable(c) => c.to_latex(),
            Self::FunctionCall(_, f) => f.to_latex(),
        }
    }
}

impl ToLatex for PreExp {
    fn to_latex(&self) -> String {
        self.latex_before(false)
    }
}

impl fmt::Display for PreExp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::ArrayAccess(a) => a.to_string(),
            Self::BlockFunction(f) => f.to_string(),
            Self::BlockScopedFunction(f) => f.to_string(),
            Self::BinaryOperation(op, lhs, rhs) => format!(
                "{} {} {}",
                lhs.operand_to_string(**op, OperandSide::Left),
                **op,
                rhs.operand_to_string(**op, OperandSide::Right)
            ),
            Self::CompoundVariable(c) => c.to_string(),
            Self::FunctionCall(_, f) => f.to_string(),
            Self::Primitive(p) => match p.value() {
                //keeps the decimal point, so that the literal is parsed back
                //as a number rather than as an integer
                Primitive::Number(n) if n.fract() == 0.0 => format!("{:.1}", n),
                _ => p.to_string(),
            },
            Self::UnaryOperation(op, exp) => {
                if op.operand_needs_parens(exp.shape()) {
                    format!("{}({})", **op, **exp)
                } else {
                    format!("{}{}", **op, **exp)
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

#[cfg(test)]
mod tests {
    use super::PreExp;
    use crate::math::BinOp;
    use crate::primitives::{Primitive, PrimitiveKind};
    use crate::runtime_builtin::RoocFunction;
    use crate::type_checker::type_checker_context::{
        FunctionContext, TypeCheckerContext, WithType,
    };
    use crate::utils::{InputSpan, Spanned};
    use indexmap::IndexMap;

    fn primitive(value: Primitive) -> PreExp {
        PreExp::Primitive(Spanned::new(value, InputSpan::default()))
    }

    fn binary_type(op: BinOp, lhs: Primitive, rhs: Primitive) -> PrimitiveKind {
        let exp = PreExp::BinaryOperation(
            Spanned::new(op, InputSpan::default()),
            primitive(lhs).to_boxed(),
            primitive(rhs).to_boxed(),
        );
        let functions = IndexMap::<String, Box<dyn RoocFunction>>::new();
        let builtins = IndexMap::<String, Box<dyn RoocFunction>>::new();
        exp.get_type(
            &TypeCheckerContext::default(),
            &FunctionContext::new(&functions, &builtins),
        )
    }

    #[test]
    fn arithmetic_types_follow_boolean_numeric_runtime_results() {
        assert_eq!(
            binary_type(BinOp::Div, Primitive::Integer(7), Primitive::Boolean(true),),
            PrimitiveKind::Number
        );
        assert_eq!(
            binary_type(
                BinOp::Sub,
                Primitive::PositiveInteger(7),
                Primitive::Boolean(true),
            ),
            PrimitiveKind::Integer
        );
        assert_eq!(
            binary_type(
                BinOp::Mul,
                Primitive::PositiveInteger(7),
                Primitive::Boolean(false),
            ),
            PrimitiveKind::PositiveInteger
        );
        assert_eq!(
            binary_type(
                BinOp::Mul,
                Primitive::Boolean(false),
                Primitive::PositiveInteger(7),
            ),
            PrimitiveKind::Number
        );
    }

    fn objective_latex(source: &str) -> String {
        use crate::traits::ToLatex;
        crate::parser::pre_model::parse_problem_source(source)
            .expect("failed to parse")
            .objective()
            .to_latex()
    }

    #[test]
    fn latex_keeps_the_meaning_of_the_expression() {
        for (exp, expected) in [
            (r"x - (y - z)", r"x - \left(y - z\right)"),
            (r"x + (y - z)", r"x + y - z"),
            (r"x / (y / z)", r"\frac{x}{\frac{y}{z}}"),
            (r"(x + y) / 2 * z", r"\frac{x + y}{2} \cdot z"),
            (r"-(x + y)", r"-\left(x + y\right)"),
            (r"not (a and b)", r"\lnot \left(a \land b\right)"),
            (r"a and b or c", r"\left(a \land b\right) \lor c"),
        ] {
            let source = format!("min {exp}\ns.t.\n    1 >= 1");
            assert_eq!(objective_latex(&source), format!(r"\min \ {expected}"));
        }
    }

    #[test]
    fn latex_big_operators_cover_exactly_their_body() {
        let sum = r"\sum_{i \in \left\{0,\dots,3\right\}}";
        for (exp, expected) in [
            (
                "sum(i in 0..3) { x_i - y_i }",
                format!(r"{sum}\left(x_{{i}} - y_{{i}}\right)"),
            ),
            ("sum(i in 0..3) { x_i } + 1", format!(r"{sum}x_{{i}} + 1")),
            (
                "2 * sum(i in 0..3) { x_i }",
                format!(r"2 \cdot {sum}x_{{i}}"),
            ),
            (
                "sum(i in 0..3) { x_i } * 2",
                format!(r"\left({sum}x_{{i}}\right) \cdot 2"),
            ),
            (
                "2 * sum(i in 0..3) { x_i } * 3",
                format!(r"2 \cdot \left({sum}x_{{i}}\right) \cdot 3"),
            ),
        ] {
            let source = format!("min {exp}\ns.t.\n    1 >= 1");
            assert_eq!(objective_latex(&source), format!(r"\min \ {expected}"));
        }
    }

    #[test]
    fn latex_of_a_satisfaction_problem_has_no_objective() {
        let latex = objective_latex("solve\ns.t.\n    1 >= 1");
        assert_eq!(latex, r"\text{solve}");
    }
}
