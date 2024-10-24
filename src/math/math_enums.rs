//TODO find a better name for this file

use core::fmt;
use num_traits::ToPrimitive;
use serde::Serialize;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

use crate::enum_with_variants_to_string;
use crate::parser::il::PreExp;
use crate::parser::model_transformer::TransformError;
use crate::parser::model_transformer::TransformerContext;
use crate::primitives::{Primitive, PrimitiveKind};
use crate::traits::ToLatex;
use crate::type_checker::type_checker_context::{
    FunctionContext, TypeCheckable, TypeCheckerContext, WithType,
};

enum_with_variants_to_string! {
    pub enum Comparison derives[Debug, PartialEq, Clone, Copy] with_wasm {
        LessOrEqual,
        GreaterOrEqual,
        Equal,
        Less,
        Greater
    }
}

impl ToLatex for Comparison {
    fn to_latex(&self) -> String {
        match self {
            Comparison::LessOrEqual => "\\leq".to_string(),
            Comparison::GreaterOrEqual => "\\geq".to_string(),
            Comparison::Equal => "=".to_string(),
            Comparison::Less => "<".to_string(),
            Comparison::Greater => ">".to_string(),
        }
    }
}

impl fmt::Display for Comparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Comparison::LessOrEqual => "<=".to_string(),
            Comparison::GreaterOrEqual => ">=".to_string(),
            Comparison::Equal => "=".to_string(),
            Comparison::Less => "<".to_string(),
            Comparison::Greater => ">".to_string(),
        };

        f.write_str(&s)
    }
}

impl FromStr for Comparison {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "<=" => Ok(Comparison::LessOrEqual),
            ">=" => Ok(Comparison::GreaterOrEqual),
            "=" => Ok(Comparison::Equal),
            "<" => Ok(Comparison::Less),
            ">" => Ok(Comparison::Greater),
            _ => Err(()),
        }
    }
}
enum_with_variants_to_string! {
    pub enum OptimizationType derives[Debug, PartialEq, Clone] with_wasm {
        Min,
        Max,
        Satisfy
    }
}
impl ToLatex for OptimizationType {
    fn to_latex(&self) -> String {
        match self {
            OptimizationType::Min => "\\min".to_string(),
            OptimizationType::Max => "\\max".to_string(),
            OptimizationType::Satisfy => "\\solve".to_string(),
        }
    }
}

impl fmt::Display for OptimizationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            OptimizationType::Min => "min".to_string(),
            OptimizationType::Max => "max".to_string(),
            OptimizationType::Satisfy => "solve".to_string(),
        };

        f.write_str(&s)
    }
}

impl FromStr for OptimizationType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "min" => Ok(OptimizationType::Min),
            "max" => Ok(OptimizationType::Max),
            "solve" => Ok(OptimizationType::Satisfy),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum PreVariableType {
    Boolean,
    PositiveReal,
    Real,
    IntegerRange(PreExp, PreExp),
}

impl PartialEq<Self> for PreVariableType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PreVariableType::Boolean, PreVariableType::Boolean) => true,
            (PreVariableType::PositiveReal, PreVariableType::PositiveReal) => true,
            (PreVariableType::Real, PreVariableType::Real) => true,
            (
                PreVariableType::IntegerRange(min1, max1),
                PreVariableType::IntegerRange(min2, max2),
            ) => {
                let first = match (min1, min2) {
                    (PreExp::Primitive(a), PreExp::Primitive(b)) => {
                        a.get_span_value() == b.get_span_value()
                    }
                    _ => false,
                };
                let second = match (max1, max2) {
                    (PreExp::Primitive(a), PreExp::Primitive(b)) => {
                        a.get_span_value() == b.get_span_value()
                    }
                    _ => false,
                };
                first && second
            }
            _ => false,
        }
    }
}

impl FromStr for PreVariableType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Boolean" => Ok(PreVariableType::Boolean),
            "PositiveReal" => Ok(PreVariableType::PositiveReal),
            "Real" => Ok(PreVariableType::Real),
            _ => Err(()),
        }
    }
}

impl PreVariableType {
    pub fn kinds_to_string() -> Vec<String> {
        vec![
            "Boolean".to_string(),
            "PositiveReal".to_string(),
            "Real".to_string(),
            "IntegerRange(min, max)".to_string(),
        ]
    }

    pub fn to_variable_type_without_context(&self) -> VariableType {
        match self {
            PreVariableType::Boolean => VariableType::Boolean,
            PreVariableType::PositiveReal => VariableType::PositiveReal,
            PreVariableType::Real => VariableType::Real,
            PreVariableType::IntegerRange(min, max) => {
                let min = match min {
                    PreExp::Primitive(p) => match **p {
                        Primitive::Integer(v) => v.to_i32().unwrap_or(-16384),
                        Primitive::PositiveInteger(v) => v.to_i32().unwrap_or(-16384),
                        _ => -16384,
                    },
                    _ => 16384,
                };
                let max = match max {
                    PreExp::Primitive(p) => match **p {
                        Primitive::Integer(v) => v.to_i32().unwrap_or(16384),
                        Primitive::PositiveInteger(v) => v.to_i32().unwrap_or(16384),
                        _ => 16384,
                    },
                    _ => 16384,
                };
                VariableType::IntegerRange(min, max)
            }
        }
    }
    pub fn to_variable_type(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<VariableType, TransformError> {
        match self {
            PreVariableType::Boolean => Ok(VariableType::Boolean),
            PreVariableType::PositiveReal => Ok(VariableType::PositiveReal),
            PreVariableType::Real => Ok(VariableType::Real),
            PreVariableType::IntegerRange(min, max) => {
                let min_i64 = min.as_integer_cast(context, fn_context)?;
                let max_i64 = max.as_integer_cast(context, fn_context)?;
                let min_i32 = min_i64.to_i32();
                let max_i32 = max_i64.to_i32();
                if min_i32.is_none() {
                    return Err(TransformError::TooLarge {
                        got: min_i64,
                        max: i32::MAX as i64,
                        message: format!(
                            "Maximum value of an IntegerRange must be less than or equal to {}",
                            i32::MAX
                        ),
                    }
                    .add_span(min.get_span()));
                }
                if max_i32.is_none() {
                    return Err(TransformError::TooLarge {
                        got: max_i64,
                        max: i32::MAX as i64,
                        message: format!(
                            "Maximum value of an IntegerRange must be less than or equal to {}",
                            i32::MAX
                        ),
                    }
                    .add_span(max.get_span()));
                }
                let min_i32 = min_i32.unwrap();
                let max_i32 = max_i32.unwrap();
                if min_i32 > max_i32 {
                    return Err(TransformError::Other(
                        format!("Minimum value of an IntegerRange must be less than or equal to the maximum value. Got {} > {}", min_i32, max_i32)
                    ));
                }
                Ok(VariableType::IntegerRange(min_i32, max_i32))
            }
        }
    }
}

impl ToLatex for PreVariableType {
    fn to_latex(&self) -> String {
        match self {
            PreVariableType::Boolean => "\\{0,1\\}".to_string(),
            PreVariableType::PositiveReal => "\\mathbb{R}^+_0".to_string(),
            PreVariableType::Real => "\\mathbb{R}".to_string(),
            PreVariableType::IntegerRange(min, max) => format!(
                "\\{{x \\in \\mathbb{{Z}} | {} \\leq x \\leq {}\\}}",
                min.to_latex(),
                max.to_latex()
            ),
        }
    }
}

impl TypeCheckable for PreVariableType {
    fn type_check(
        &self,
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        match self {
            PreVariableType::Boolean => Ok(()),
            PreVariableType::PositiveReal => Ok(()),
            PreVariableType::Real => Ok(()),
            PreVariableType::IntegerRange(min, max) => {
                min.type_check(context, fn_context)?;
                max.type_check(context, fn_context)?;
                let lhs = min.get_type(context, fn_context);
                let rhs = max.get_type(context, fn_context);
                if !matches!(lhs, PrimitiveKind::Integer | PrimitiveKind::PositiveInteger) {
                    return Err(TransformError::from_wrong_type(
                        PrimitiveKind::Integer,
                        lhs,
                        min.get_span().clone(),
                    ));
                }
                if !matches!(rhs, PrimitiveKind::Integer | PrimitiveKind::PositiveInteger) {
                    return Err(TransformError::from_wrong_type(
                        PrimitiveKind::Integer,
                        rhs,
                        max.get_span().clone(),
                    ));
                }
                Ok(())
            }
        }
    }

    fn populate_token_type_map(
        &self,
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) {
        match self {
            PreVariableType::Boolean => {}
            PreVariableType::PositiveReal => {}
            PreVariableType::Real => {}
            PreVariableType::IntegerRange(min, max) => {
                min.populate_token_type_map(context, fn_context);
                max.populate_token_type_map(context, fn_context);
            }
        }
    }
}

impl fmt::Display for PreVariableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PreVariableType::Boolean => "Boolean".to_string(),
            PreVariableType::PositiveReal => "PositiveReal".to_string(),
            PreVariableType::Real => "Real".to_string(),
            PreVariableType::IntegerRange(min, max) => format!("IntegerRange({}, {})", min, max),
        };

        f.write_str(&s)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum VariableType {
    Boolean,
    PositiveReal,
    Real,
    IntegerRange(i32, i32),
}
#[wasm_bindgen(typescript_custom_section)]
const IVariablesDomainDeclaration: &'static str = r#"
export type VariableType = {
    type: "Boolean" | "PositiveReal" | "Real"
} | {
    type: "IntegerRange"
    value: [number, number]
}
"#;

impl VariableType {
    pub fn kinds_to_string() -> Vec<String> {
        vec![
            "Boolean".to_string(),
            "PositiveReal".to_string(),
            "Real".to_string(),
            "IntegerRange(min, max)".to_string(),
        ]
    }
}

impl fmt::Display for VariableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            VariableType::Boolean => "Boolean".to_string(),
            VariableType::PositiveReal => "PositiveReal".to_string(),
            VariableType::Real => "Real".to_string(),
            VariableType::IntegerRange(min, max) => format!("IntegerRange({}, {})", min, max),
        };

        f.write_str(&s)
    }
}

impl FromStr for VariableType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Boolean" => Ok(VariableType::Boolean),
            "PositiveReal" => Ok(VariableType::PositiveReal),
            "Real" => Ok(VariableType::Real),
            _ => Err(()),
        }
    }
}

impl ToLatex for VariableType {
    fn to_latex(&self) -> String {
        match self {
            VariableType::Boolean => "\\{0,1\\}".to_string(),
            VariableType::PositiveReal => "\\mathbb{R}^+_0".to_string(),
            VariableType::Real => "\\mathbb{R}".to_string(),
            VariableType::IntegerRange(min, max) => format!(
                "\\{{{} \\in \\mathbb{{Z}} | {} \\leq {} \\leq {}\\}}",
                min, min, "x", max
            ),
        }
    }
}
