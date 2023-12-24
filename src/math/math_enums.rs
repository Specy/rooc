//TODO find a better name for this file

use std::str::FromStr;

use crate::enum_with_variants_to_string;

enum_with_variants_to_string! {
    pub enum Comparison derives[Debug, PartialEq, Clone, Copy] {
        LowerOrEqual,
        UpperOrEqual,
        Equal,
    }
}

impl Comparison {
    pub fn to_string(&self) -> String {
        match self {
            Comparison::LowerOrEqual => "<=".to_string(),
            Comparison::UpperOrEqual => ">=".to_string(),
            Comparison::Equal => "=".to_string(),
        }
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
    pub enum OptimizationType derives[Debug, PartialEq, Clone] {
        Min,
        Max,
    }
}
impl OptimizationType {
    pub fn to_string(&self) -> String {
        match self {
            OptimizationType::Min => "min".to_string(),
            OptimizationType::Max => "max".to_string(),
        }
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
