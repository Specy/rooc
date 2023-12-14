use crate::{
    enum_with_variants_to_string,
    math_enums::{Comparison, Op, OptimizationType},
    primitives::{
        functions::function_traits::FunctionCall, parameter::Parameter, primitive::Primitive,
    },
    utils::{InputSpan, Spanned},
};

use super::{
    recursive_set_resolver::recursive_set_resolver,
    transformer::{Exp, TransformError, TransformerContext, VariableType},
};

enum_with_variants_to_string! {
    pub enum BlockScopedFunctionKind derives[Debug] {
        Sum,
        Prod,
    }
}
impl BlockScopedFunctionKind {
    pub fn to_string(&self) -> String {
        match self {
            Self::Sum => "sum".to_string(),
            Self::Prod => "prod".to_string(),
        }
    }
}
enum_with_variants_to_string! {
    pub enum BlockFunctionKind derives[Debug] {
        Min,
        Max,
        Avg,
    }
}

impl BlockFunctionKind {
    pub fn to_string(&self) -> String {
        match self {
            Self::Min => "min".to_string(),
            Self::Max => "max".to_string(),
            Self::Avg => "avg".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct BlockScopedFunction {
    kind: BlockScopedFunctionKind,
    iters: Vec<IterableSet>,
    exp: Box<PreExp>,
}
impl BlockScopedFunction {
    pub fn new(kind: BlockScopedFunctionKind, iters: Vec<IterableSet>, exp: Box<PreExp>) -> Self {
        Self { kind, iters, exp }
    }
    pub fn get_body_span(&self) -> InputSpan {
        self.exp.get_span().clone()
    }
}
#[derive(Debug)]
pub struct BlockFunction {
    kind: BlockFunctionKind,
    exps: Vec<PreExp>,
}
impl BlockFunction {
    pub fn new(kind: BlockFunctionKind, exps: Vec<PreExp>) -> Self {
        Self { kind, exps }
    }
}

#[derive(Debug)]
pub enum PreExp {
    Number(Spanned<f64>),
    Mod(Spanned<Box<PreExp>>),
    BlockFunction(Spanned<BlockFunction>),
    Variable(Spanned<String>),
    CompoundVariable(Spanned<CompoundVariable>),
    ArrayAccess(Spanned<ArrayAccess>),
    BlockScopedFunction(Spanned<BlockScopedFunction>),
    FunctionCall(Spanned<Box<dyn FunctionCall>>),

    BinaryOperation(Spanned<Op>, Box<PreExp>, Box<PreExp>),
    UnaryNegation(Spanned<Box<PreExp>>),
}

impl PreExp {
    pub fn to_boxed(self) -> Box<PreExp> {
        Box::new(self)
    }
    pub fn get_span(&self) -> &InputSpan {
        match self {
            Self::Number(n) => n.get_span(),
            Self::Mod(exp) => exp.get_span(),
            Self::BlockFunction(f) => f.get_span(),
            Self::Variable(name) => name.get_span(),
            Self::CompoundVariable(c) => c.get_span(),
            Self::BinaryOperation(op, _, _) => op.get_span(),
            Self::UnaryNegation(exp) => exp.get_span(),
            Self::ArrayAccess(array_access) => array_access.get_span(),
            Self::BlockScopedFunction(function) => function.get_span(),
            Self::FunctionCall(function_call) => function_call.get_span(),
        }
    }
    pub fn into_exp(&self, context: &mut TransformerContext) -> Result<Exp, TransformError> {
        let exp = match self {
            Self::BinaryOperation(op, lhs, rhs) => {
                let lhs = lhs
                    .into_exp(context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                let rhs = rhs
                    .into_exp(context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                match (op, lhs, rhs) {
                    (op, lhs, rhs) => Ok(Exp::BinOp(**op, lhs.to_box(), rhs.to_box())),
                }
            }
            Self::Number(n) => Ok(Exp::Number(**n)),
            Self::Mod(exp) => {
                let inner = exp
                    .into_exp(context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                Ok(Exp::Mod(inner.to_box()))
            }
            Self::BlockFunction(f) => {
                let mut parsed_exp = f
                    .exps
                    .iter()
                    .map(|exp| exp.into_exp(context))
                    .collect::<Result<Vec<Exp>, TransformError>>()
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                match f.kind {
                    BlockFunctionKind::Min => Ok(Exp::Min(parsed_exp)),
                    BlockFunctionKind::Max => Ok(Exp::Max(parsed_exp)),
                    BlockFunctionKind::Avg => {
                        let len = parsed_exp.len();
                        let mut sum = parsed_exp.pop().unwrap_or(Exp::Number(0.0));
                        for exp in parsed_exp.into_iter().rev() {
                            sum = Exp::BinOp(Op::Add, exp.to_box(), sum.to_box());
                        }
                        Ok(Exp::BinOp(
                            Op::Div,
                            sum.to_box(),
                            Exp::Number(len as f64).to_box(),
                        ))
                    }
                }
            }

            Self::UnaryNegation(exp) => {
                let inner = exp
                    .into_exp(context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                Ok(Exp::Neg(inner.to_box()))
            }
            Self::Variable(name) => {
                let value = context.get_value(&*name).map(|v| match v {
                    Primitive::Number(n) => Ok(Exp::Number(n.clone())),
                    _ => {
                        let err = TransformError::WrongArgument(format!(
                            "Expected \"Number\", got \"{}\"",
                            v.get_type().to_string()
                        ));
                        Err(err.to_spanned_error(self.get_span()))
                    }
                });
                match value {
                    Some(value) => Ok(value?),
                    None => Ok(Exp::Variable(name.get_span_value().clone())),
                }
            }
            Self::CompoundVariable(c) => {
                let parsed_indexes = context
                    .flatten_variable_name(&c.indexes)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                Ok(Exp::Variable(format!("{}_{}", c.name, parsed_indexes)))
            }
            Self::ArrayAccess(array_access) => {
                let value = context
                    .get_array_value(array_access)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                match value {
                    Primitive::Number(n) => Ok(Exp::Number(n.clone())),
                    _ => {
                        let err = TransformError::WrongArgument(format!(
                            "Expected \"Number\", got \"{}\"",
                            value.get_type().to_string()
                        ));
                        Err(err.to_spanned_error(self.get_span()))
                    }
                }
            }
            Self::BlockScopedFunction(f) => {
                let mut results = Vec::new();
                recursive_set_resolver(&f.iters, context, &mut results, 0, &|context| {
                    let inner = f
                        .exp
                        .into_exp(context)
                        .map_err(|e| e.to_spanned_error(self.get_span()))?;
                    Ok(inner)
                })
                .map_err(|e| e.to_spanned_error(self.get_span()))?;
                match f.kind {
                    BlockScopedFunctionKind::Sum => {
                        let mut sum = results.pop().unwrap_or(Exp::Number(0.0));
                        for result in results.into_iter().rev() {
                            sum = Exp::BinOp(Op::Add, result.to_box(), sum.to_box());
                        }
                        Ok(sum)
                    }
                    BlockScopedFunctionKind::Prod => {
                        let mut prod = results.pop().unwrap_or(Exp::Number(1.0));
                        for result in results.into_iter().rev() {
                            prod = Exp::BinOp(Op::Mul, result.to_box(), prod.to_box());
                        }
                        Ok(prod)
                    }
                }
            }
            Self::FunctionCall(function_call) => {
                //TODO improve this, what other types of functions can there be?
                let value = function_call
                    .call(&context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                match value {
                    Primitive::Number(n) => Ok(Exp::Number(n)),
                    _ => {
                        let err = TransformError::WrongArgument(format!(
                            "Expected \"Number\", got \"{}\"",
                            value.get_type().to_string()
                        ));
                        Err(err.to_spanned_error(self.get_span()))
                    }
                }
            }
        };
        exp.map(|e: Exp| e.flatten())
    }
}

#[derive(Debug)]
pub struct IterableSet {
    pub var: VariableType,
    pub iterator: Spanned<Parameter>,
    pub span: InputSpan,
}
impl IterableSet {
    pub fn new(var: VariableType, iterator: Spanned<Parameter>, span: InputSpan) -> Self {
        Self {
            var,
            iterator,
            span,
        }
    }
}

#[derive(Debug)]
pub struct ArrayAccess {
    pub name: String,
    pub accesses: Vec<Parameter>,
}
impl ArrayAccess {
    pub fn new(name: String, accesses: Vec<Parameter>) -> Self {
        Self { name, accesses }
    }
    pub fn to_string(&self) -> String {
        let rest = self
            .accesses
            .iter()
            .map(|a| format!("[{}]", a.to_string()))
            .collect::<Vec<String>>()
            .join("");
        format!("{}{}", self.name, rest)
    }
}

#[derive(Debug)]
pub struct CompoundVariable {
    pub name: String,
    pub indexes: Vec<String>,
}
impl CompoundVariable {
    pub fn new(name: String, indexes: Vec<String>) -> Self {
        Self { name, indexes }
    }
    pub fn to_string(&self) -> String {
        format!("{}_{}", self.name, self.indexes.join("_"))
    }
}

#[derive(Debug)]
pub struct PreObjective {
    pub objective_type: OptimizationType,
    pub rhs: PreExp,
}

impl PreObjective {
    pub fn new(objective_type: OptimizationType, rhs: PreExp) -> Self {
        Self {
            objective_type,
            rhs,
        }
    }
}

#[derive(Debug)]
pub struct PreCondition {
    pub lhs: PreExp,
    pub condition_type: Comparison,
    pub rhs: PreExp,
    pub iteration: Vec<IterableSet>,
    pub span: InputSpan,
}

impl PreCondition {
    pub fn new(
        lhs: PreExp,
        condition_type: Comparison,
        rhs: PreExp,
        iteration: Vec<IterableSet>,
        span: InputSpan,
    ) -> Self {
        Self {
            lhs,
            condition_type,
            rhs,
            iteration,
            span,
        }
    }
}
