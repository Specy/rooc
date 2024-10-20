//TODO find a better name for this file

use core::fmt;
use std::str::FromStr;

use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::enum_with_variants_to_string;
use crate::traits::latex::ToLatex;

enum_with_variants_to_string! {
    pub enum Comparison derives[Debug, PartialEq, Clone, Copy] with_wasm {
        LessOrEqual,
        GreaterOrEqual,
        Equal,
    }
}

impl ToLatex for Comparison {
    fn to_latex(&self) -> String {
        match self {
            Comparison::LessOrEqual => "\\leq".to_string(),
            Comparison::GreaterOrEqual => "\\geq".to_string(),
            Comparison::Equal => "=".to_string(),
        }
    }
}

impl fmt::Display for Comparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Comparison::LessOrEqual => "<=".to_string(),
            Comparison::GreaterOrEqual => ">=".to_string(),
            Comparison::Equal => "=".to_string(),
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
