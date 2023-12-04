use crate::{
    math_enums::Op,
    primitives::{functions::function_traits::FunctionCall, primitive::Primitive},
    utils::{InputSpan, Spanned},
};

use super::{
    parser::{recursive_set_resolver, ArrayAccess, CompoundVariable, PreSet},
    transformer::{Exp, TransformError, TransformerContext},
};

#[derive(Debug)]
pub enum PreExp {
    Number(Spanned<f64>),
    Mod(Spanned<Box<PreExp>>),
    Min(Spanned<Vec<PreExp>>),
    Max(Spanned<Vec<PreExp>>),
    Variable(Spanned<String>),
    CompoundVariable(Spanned<CompoundVariable>),
    BinaryOperation(Spanned<Op>, Box<PreExp>, Box<PreExp>),
    UnaryNegation(Spanned<Box<PreExp>>),
    ArrayAccess(Spanned<ArrayAccess>),
    Sum(Vec<PreSet>, Spanned<Box<PreExp>>),
    FunctionCall(Spanned<Box<dyn FunctionCall>>),
}

impl PreExp {
    pub fn to_boxed(self) -> Box<PreExp> {
        Box::new(self)
    }
    pub fn get_span(&self) -> &InputSpan {
        match self {
            Self::Number(n) => n.get_span(),
            Self::Mod(exp) => exp.get_span(),
            Self::Min(exps) => exps.get_span(),
            Self::Max(exps) => exps.get_span(),
            Self::Variable(name) => name.get_span(),
            Self::CompoundVariable(c) => c.get_span(),
            Self::BinaryOperation(op, _, _) => op.get_span(),
            Self::UnaryNegation(exp) => exp.get_span(),
            Self::ArrayAccess(array_access) => array_access.get_span(),
            Self::Sum(_, exp_body) => exp_body.get_span(),
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
            Self::Min(exps) => {
                let parsed_exp = exps
                    .iter()
                    .map(|exp| exp.into_exp(context))
                    .collect::<Result<Vec<Exp>, TransformError>>()
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                Ok(Exp::Min(parsed_exp))
            }
            Self::Max(exps) => {
                let parsed_exp = exps
                    .iter()
                    .map(|exp| exp.into_exp(context))
                    .collect::<Result<Vec<Exp>, TransformError>>()
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                Ok(Exp::Max(parsed_exp))
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
                            "Expected number, got {}",
                            v.get_argument_name()
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
                    .get_array_access_value(array_access)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                Ok(Exp::Number(value))
            }
            Self::Sum(sets, exp) => {
                let mut results = Vec::new();
                recursive_set_resolver(&sets, context, &mut results, 0, &|context| {
                    let inner = exp
                        .into_exp(context)
                        .map_err(|e| e.to_spanned_error(self.get_span()))?;
                    Ok(inner)
                })
                .map_err(|e| e.to_spanned_error(self.get_span()))?;
                let mut sum = results.pop().unwrap_or(Exp::Number(0.0));
                for result in results.into_iter().rev() {
                    sum = Exp::BinOp(Op::Add, result.to_box(), sum.to_box());
                }
                Ok(sum)
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
                            "Expected number, got {}",
                            value.get_argument_name()
                        ));
                        Err(err.to_spanned_error(self.get_span()))
                    }
                }
            }
        };
        exp.map(|e: Exp| e.flatten())
    }
}
