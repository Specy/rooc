//TODO find a better name for this file

use crate::parser::il::PreExp;
use crate::parser::model_transformer::TransformError;
use crate::parser::model_transformer::TransformerContext;
#[allow(unused_imports)]
use crate::prelude::*;
use crate::primitives::{Primitive, PrimitiveKind};
use crate::traits::ToLatex;
use crate::type_checker::type_checker_context::{
    FunctionContext, TypeCheckable, TypeCheckerContext, WithType,
};
use crate::{enum_with_variants_to_string, InputSpan, Spanned};
use core::f64;
use core::fmt;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

enum_with_variants_to_string! {
    pub enum Comparison derives[Debug, PartialEq, Clone, Copy, Deserialize] with_wasm {
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

    pub enum OptimizationType derives[Debug, PartialEq, Clone, Deserialize] with_wasm {
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

/// Represents the type of a variable before type checking and transformation
#[derive(Debug, Clone, Serialize)]
pub enum PreVariableType {
    /// Boolean variable (0 or 1)
    Boolean,
    /// Real number greater than or equal to zero within a [min, max] range (optional)
    NonNegativeReal(Option<PreExp>, Option<PreExp>),
    /// Any real number within a [min, max] range (optional)
    Real(Option<PreExp>, Option<PreExp>),
    /// Integer within a specified range [min, max]
    IntegerRange(PreExp, PreExp),
}

fn default_bound(negative: bool, zero: bool) -> PreExp {
    let val = if negative {
        f64::NEG_INFINITY
    } else if zero {
        0.0
    } else {
        f64::INFINITY
    };
    let spanned = Spanned::new(Primitive::Number(val), InputSpan::default());
    PreExp::Primitive(spanned)
}

impl PartialEq<Self> for PreVariableType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PreVariableType::Boolean, PreVariableType::Boolean) => true,
            (
                PreVariableType::NonNegativeReal(min1, max1),
                PreVariableType::NonNegativeReal(min2, max2),
            ) => {
                let min1 = min1.clone().unwrap_or_else(|| default_bound(false, true));
                let max1 = max1.clone().unwrap_or_else(|| default_bound(false, false));
                let min2 = min2.clone().unwrap_or_else(|| default_bound(false, true));
                let max2 = max2.clone().unwrap_or_else(|| default_bound(false, false));
                let min_eq = match (min1, min2) {
                    (PreExp::Primitive(a), PreExp::Primitive(b)) => a.value() == b.value(),
                    _ => false,
                };
                let max_eq = match (max1, max2) {
                    (PreExp::Primitive(a), PreExp::Primitive(b)) => a.value() == b.value(),
                    _ => false,
                };
                min_eq && max_eq
            }
            (PreVariableType::Real(min1, max1), PreVariableType::Real(min2, max2)) => {
                let min1 = min1.clone().unwrap_or_else(|| default_bound(true, false));
                let max1 = max1.clone().unwrap_or_else(|| default_bound(false, false));
                let min2 = min2.clone().unwrap_or_else(|| default_bound(true, false));
                let max2 = max2.clone().unwrap_or_else(|| default_bound(false, false));
                let min_eq = match (min1, min2) {
                    (PreExp::Primitive(a), PreExp::Primitive(b)) => a.value() == b.value(),
                    _ => false,
                };
                let max_eq = match (max1, max2) {
                    (PreExp::Primitive(a), PreExp::Primitive(b)) => a.value() == b.value(),
                    _ => false,
                };
                min_eq && max_eq
            }
            (
                PreVariableType::IntegerRange(min1, max1),
                PreVariableType::IntegerRange(min2, max2),
            ) => {
                let first = match (min1, min2) {
                    (PreExp::Primitive(a), PreExp::Primitive(b)) => a.value() == b.value(),
                    _ => false,
                };
                let second = match (max1, max2) {
                    (PreExp::Primitive(a), PreExp::Primitive(b)) => a.value() == b.value(),
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
            "NonNegativeReal" => Ok(PreVariableType::NonNegativeReal(None, None)),
            "Real" => Ok(PreVariableType::Real(None, None)),
            _ => Err(()),
        }
    }
}

impl PreVariableType {
    /// Returns a list of all available variable type names as strings
    pub fn kinds_to_string() -> Vec<String> {
        vec![
            "Boolean".to_string(),
            "NonNegativeReal".to_string(),
            "Real".to_string(),
            "IntegerRange(min, max)".to_string(),
        ]
    }

    /// Converts PreVariableType to VariableType without using context
    /// This is a simplified conversion that uses default values for ranges
    pub fn to_variable_type_without_context(&self) -> VariableType {
        match self {
            PreVariableType::Boolean => VariableType::Boolean,
            PreVariableType::NonNegativeReal(min, max) => {
                let min = match min {
                    Some(PreExp::Primitive(p)) => match **p {
                        Primitive::Integer(v) => v.to_f64().unwrap_or(0.0),
                        Primitive::PositiveInteger(v) => v.to_f64().unwrap_or(0.0),
                        Primitive::Number(v) => v,
                        _ => 0.0,
                    },
                    _ => 0.0,
                };
                let max = match max {
                    Some(PreExp::Primitive(p)) => match **p {
                        Primitive::Integer(v) => v.to_f64().unwrap_or(f64::INFINITY),
                        Primitive::PositiveInteger(v) => v.to_f64().unwrap_or(f64::INFINITY),
                        Primitive::Number(v) => v,
                        _ => f64::INFINITY,
                    },
                    _ => f64::INFINITY,
                };
                VariableType::NonNegativeReal(min, max)
            }
            PreVariableType::Real(min, max) => {
                let min = match min {
                    Some(PreExp::Primitive(p)) => match **p {
                        Primitive::Integer(v) => v.to_f64().unwrap_or(f64::NEG_INFINITY),
                        Primitive::PositiveInteger(v) => v.to_f64().unwrap_or(f64::NEG_INFINITY),
                        Primitive::Number(v) => v,
                        _ => f64::NEG_INFINITY,
                    },
                    _ => f64::NEG_INFINITY,
                };
                let max = match max {
                    Some(PreExp::Primitive(p)) => match **p {
                        Primitive::Integer(v) => v.to_f64().unwrap_or(f64::INFINITY),
                        Primitive::PositiveInteger(v) => v.to_f64().unwrap_or(f64::INFINITY),
                        Primitive::Number(v) => v,
                        _ => f64::INFINITY,
                    },
                    _ => f64::INFINITY,
                };
                VariableType::Real(min, max)
            }
            PreVariableType::IntegerRange(min, max) => {
                let min = match min {
                    PreExp::Primitive(p) => match **p {
                        Primitive::Integer(v) => v.to_i32().unwrap_or(-16384),
                        Primitive::PositiveInteger(v) => v.to_i32().unwrap_or(-16384),
                        _ => -16384,
                    },
                    _ => -16384,
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

    /// Converts PreVariableType to VariableType using the provided context
    /// for evaluating range expressions
    pub fn to_variable_type(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<VariableType, TransformError> {
        match self {
            PreVariableType::Boolean => Ok(VariableType::Boolean),
            PreVariableType::NonNegativeReal(min, max) => {
                let min = min.clone().unwrap_or_else(|| default_bound(false, true));
                let max = max.clone().unwrap_or_else(|| default_bound(false, false));
                let min_f64 = min.as_number_cast(context, fn_context)?;
                let max_f64 = max.as_number_cast(context, fn_context)?;
                if min_f64 < 0.0 {
                    return Err(TransformError::Other(
                        format!("Minimum value of a NonNegativeReal must be greater than or equal to 0. Got {}", min_f64)
                    ).add_span(min.span()));
                }
                if min_f64 > max_f64 {
                    return Err(TransformError::Other(
                        format!("Minimum value must be less than or equal to the maximum value. Got {} > {}", min_f64, max_f64)
                    ).add_span(min.span()));
                }
                Ok(VariableType::NonNegativeReal(min_f64, max_f64))
            }
            PreVariableType::Real(min, max) => {
                let min = min.clone().unwrap_or_else(|| default_bound(true, false));
                let max = max.clone().unwrap_or_else(|| default_bound(false, false));
                let min_f64 = min.as_number_cast(context, fn_context)?;
                let max_f64 = max.as_number_cast(context, fn_context)?;
                if min_f64 > max_f64 {
                    return Err(TransformError::Other(
                        format!("Minimum value must be less than or equal to the maximum value. Got {} > {}", min_f64, max_f64)
                    ));
                }
                Ok(VariableType::Real(min_f64, max_f64))
            }
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
                    .add_span(min.span()));
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
                    .add_span(max.span()));
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
            PreVariableType::NonNegativeReal(min, max) => match (min, max) {
                (None, None) => "\\mathbb{R}^+_0".to_string(),
                (min, max) => format!(
                    "\\{{x \\in \\mathbb{{R}}^+_0 | {} \\leq x \\leq {}\\}}",
                    min.clone()
                        .map(|m| m.to_latex())
                        .unwrap_or_else(|| "0".to_string()),
                    max.clone()
                        .map(|m| m.to_latex())
                        .unwrap_or_else(|| "\\infty".to_string())
                ),
            },
            PreVariableType::Real(min, max) => match (min, max) {
                (None, None) => "\\mathbb{R}".to_string(),
                (min, max) => format!(
                    "\\{{x \\in \\mathbb{{R}} | {} \\leq x \\leq {}\\}}",
                    min.clone()
                        .map(|m| m.to_latex())
                        .unwrap_or_else(|| "-\\infty".to_string()),
                    max.clone()
                        .map(|m| m.to_latex())
                        .unwrap_or_else(|| "\\infty".to_string())
                ),
            },
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
            PreVariableType::NonNegativeReal(min, max) => {
                if let Some(min) = min {
                    min.type_check(context, fn_context)?;
                    let min_type = min.get_type(context, fn_context);
                    if !min_type.is_numeric() {
                        return Err(TransformError::from_wrong_type(
                            PrimitiveKind::Number,
                            min_type,
                            min.span().clone(),
                        ));
                    }
                }
                if let Some(max) = max {
                    max.type_check(context, fn_context)?;
                    let max_type = max.get_type(context, fn_context);
                    if !max_type.is_numeric() {
                        return Err(TransformError::from_wrong_type(
                            PrimitiveKind::Number,
                            max_type,
                            max.span().clone(),
                        ));
                    }
                }
                Ok(())
            }
            PreVariableType::Real(min, max) => {
                if let Some(min) = min {
                    min.type_check(context, fn_context)?;
                    let min_type = min.get_type(context, fn_context);
                    if !min_type.is_numeric() {
                        return Err(TransformError::from_wrong_type(
                            PrimitiveKind::Number,
                            min_type,
                            min.span().clone(),
                        ));
                    }
                }
                if let Some(max) = max {
                    max.type_check(context, fn_context)?;
                    let max_type = max.get_type(context, fn_context);
                    if !max_type.is_numeric() {
                        return Err(TransformError::from_wrong_type(
                            PrimitiveKind::Number,
                            max_type,
                            max.span().clone(),
                        ));
                    }
                }
                Ok(())
            }
            PreVariableType::IntegerRange(min, max) => {
                min.type_check(context, fn_context)?;
                max.type_check(context, fn_context)?;
                let lhs = min.get_type(context, fn_context);
                let rhs = max.get_type(context, fn_context);
                if !matches!(lhs, PrimitiveKind::Integer | PrimitiveKind::PositiveInteger) {
                    return Err(TransformError::from_wrong_type(
                        PrimitiveKind::Integer,
                        lhs,
                        min.span().clone(),
                    ));
                }
                if !matches!(rhs, PrimitiveKind::Integer | PrimitiveKind::PositiveInteger) {
                    return Err(TransformError::from_wrong_type(
                        PrimitiveKind::Integer,
                        rhs,
                        max.span().clone(),
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
            PreVariableType::NonNegativeReal(min, max) | PreVariableType::Real(min, max) => {
                if let Some(min) = min {
                    min.populate_token_type_map(context, fn_context);
                }
                if let Some(max) = max {
                    max.populate_token_type_map(context, fn_context);
                }
            }
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
            PreVariableType::NonNegativeReal(min, max) => match (min, max) {
                (None, None) => "NonNegativeReal".to_string(),
                (min, max) => format!(
                    "NonNegativeReal({}, {})",
                    min.clone().map_or("0".to_string(), |m| m.to_string()),
                    max.clone()
                        .map_or("Infinity".to_string(), |m| m.to_string())
                ),
            },
            PreVariableType::Real(min, max) => match (min, max) {
                (None, None) => "Real".to_string(),
                (min, max) => format!(
                    "Real({}, {})",
                    min.clone()
                        .map_or("MinusInfinity".to_string(), |m| m.to_string()),
                    max.clone()
                        .map_or("Infinity".to_string(), |m| m.to_string())
                ),
            },
            PreVariableType::IntegerRange(min, max) => format!("IntegerRange({}, {})", min, max),
        };

        f.write_str(&s)
    }
}

/// Represents the final, resolved type of a variable after being compiled
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum VariableType {
    /// Boolean variable (0 or 1)
    Boolean,
    /// Real number greater than or equal to zero
    NonNegativeReal(f64, f64),
    /// Any real number
    Real(f64, f64),
    /// Integer within a specified range [min, max]
    IntegerRange(i32, i32),
}

//TODO change this
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
const IVariablesDomainDeclaration: &'static str = r#"
export type VariableType = {
    type: "Boolean" 
} | {
    type: "IntegerRange" | "NonNegativeReal" | "Real"
    value: [number, number]
}
"#;

impl VariableType {
    pub fn non_negative_real() -> VariableType {
        VariableType::NonNegativeReal(0.0, f64::INFINITY)
    }
    pub fn real() -> VariableType {
        VariableType::Real(f64::NEG_INFINITY, f64::INFINITY)
    }
    pub fn bool() -> VariableType {
        VariableType::Boolean
    }
    pub fn integer_range(min: i32, max: i32) -> VariableType {
        VariableType::IntegerRange(min, max)
    }
    /// Returns a list of all available variable type names as strings
    pub fn kinds_to_string() -> Vec<String> {
        vec![
            "Boolean".to_string(),
            "NonNegativeReal".to_string(),
            "Real".to_string(),
            "IntegerRange(min, max)".to_string(),
        ]
    }
}

impl fmt::Display for VariableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            VariableType::Boolean => "Boolean".to_string(),
            VariableType::NonNegativeReal(min, max) => match (*min, *max) {
                (0.0, f64::INFINITY) => "NonNegativeReal".to_string(),
                _ => format!(
                    "NonNegativeReal({}, {})",
                    min,
                    if *max == f64::INFINITY {
                        "Infinity".to_string()
                    } else {
                        max.to_string()
                    }
                ),
            },
            VariableType::Real(min, max) => match (*min, *max) {
                (f64::NEG_INFINITY, f64::INFINITY) => "Real".to_string(),
                _ => format!(
                    "Real({}, {})",
                    if *min == f64::NEG_INFINITY {
                        "MinusInfinity".to_string()
                    } else {
                        min.to_string()
                    },
                    if *max == f64::INFINITY {
                        "Infinity".to_string()
                    } else {
                        max.to_string()
                    }
                ),
            },
            VariableType::IntegerRange(min, max) => format!("IntegerRange({}, {})", min, max),
        };

        f.write_str(&s)
    }
}

impl ToLatex for VariableType {
    fn to_latex(&self) -> String {
        match self {
            VariableType::Boolean => "\\{0,1\\}".to_string(),
            VariableType::NonNegativeReal(min, max) => match (*min, *max) {
                (0.0, f64::INFINITY) => "\\mathbb{R}^+_0".to_string(),
                _ => format!(
                    "\\{{{} \\in \\mathbb{{R}}^+_0 | {} \\leq {} \\leq {}\\}}",
                    "x",
                    if *min == 0.0 {
                        "0".to_string()
                    } else {
                        min.to_string()
                    },
                    "x",
                    if *max == f64::INFINITY {
                        "\\infty".to_string()
                    } else {
                        max.to_string()
                    }
                ),
            },
            VariableType::Real(min, max) => match (*min, *max) {
                (f64::NEG_INFINITY, f64::INFINITY) => "\\mathbb{R}".to_string(),
                _ => format!(
                    "\\{{{} \\in \\mathbb{{R}} | {} \\leq {} \\leq {}\\}}",
                    "x",
                    if *min == f64::NEG_INFINITY {
                        "-\\infty".to_string()
                    } else {
                        min.to_string()
                    },
                    "x",
                    if *max == f64::INFINITY {
                        "\\infty".to_string()
                    } else {
                        max.to_string()
                    }
                ),
            },
            VariableType::IntegerRange(min, max) => format!(
                "\\{{{} \\in \\mathbb{{Z}} | {} \\leq {} \\leq {}\\}}",
                min, min, "x", max
            ),
        }
    }
}
