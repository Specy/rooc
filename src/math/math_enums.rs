//TODO find a better name for this file

use core::fmt;
use std::str::FromStr;

use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::enum_with_variants_to_string;
use crate::traits::latex::ToLatex;

enum_with_variants_to_string! {
    pub enum Comparison derives[Debug, PartialEq, Clone, Copy] with_wasm {
        LowerOrEqual,
        UpperOrEqual,
        Equal,
    }
}

impl ToLatex for Comparison {
    fn to_latex(&self) -> String {
        match self {
            Comparison::LowerOrEqual => "\\leq".to_string(),
            Comparison::UpperOrEqual => "\\geq".to_string(),
            Comparison::Equal => "=".to_string(),
        }
    }
}

impl fmt::Display for Comparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Comparison::LowerOrEqual => "<=".to_string(),
            Comparison::UpperOrEqual => ">=".to_string(),
            Comparison::Equal => "=".to_string(),
        };

        f.write_str(&s)
    }
}

impl FromStr for Comparison {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "<=" => Ok(Comparison::LowerOrEqual),
            ">=" => Ok(Comparison::UpperOrEqual),
            "=" => Ok(Comparison::Equal),
            _ => Err(()),
        }
    }
}
enum_with_variants_to_string! {
    pub enum OptimizationType derives[Debug, PartialEq, Clone] with_wasm {
        Min,
        Max,
    }
}
impl ToLatex for OptimizationType {
    fn to_latex(&self) -> String {
        match self {
            OptimizationType::Min => "\\min".to_string(),
            OptimizationType::Max => "\\max".to_string(),
        }
    }
}

impl fmt::Display for OptimizationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            OptimizationType::Min => "min".to_string(),
            OptimizationType::Max => "max".to_string(),
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
            _ => Err(()),
        }
    }
}


enum_with_variants_to_string!{
    pub enum VariableType derives[Debug, PartialEq, Clone] with_wasm{
        Integer,
        Boolean,
        PositiveReal,
        Real,
    }
}

impl fmt::Display for VariableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            VariableType::Integer => "Integer".to_string(),
            VariableType::Boolean => "Boolean".to_string(),
            VariableType::PositiveReal => "PositiveReal".to_string(),
            VariableType::Real => "Real".to_string(),
        };

        f.write_str(&s)
    }
}
impl FromStr for VariableType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Integer" => Ok(VariableType::Integer),
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
            VariableType::Integer => "\\mathbb{Z}".to_string(),
            VariableType::Boolean => "\\{0,1\\}".to_string(),
            VariableType::PositiveReal => "\\mathbb{R}^+".to_string(),
            VariableType::Real => "\\mathbb{R}".to_string(),
        }
    }
}
