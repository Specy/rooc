//TODO find a better name for this file

use core::fmt;
use std::str::FromStr;
use crate::enum_with_variants_to_string;
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;
enum_with_variants_to_string! {
    pub enum Comparison derives[Debug, PartialEq, Clone, Copy] with_wasm {
        LowerOrEqual,
        UpperOrEqual,
        Equal,
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
