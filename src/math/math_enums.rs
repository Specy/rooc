//TODO find a better name for this file

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Comparison {
    LowerOrEqual,
    UpperOrEqual,
    Equal,
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


#[derive(Debug, PartialEq, Clone)]
pub enum OptimizationType {
    Min,
    Max,
}
impl OptimizationType {
    pub fn to_string(&self) -> String {
        match self {
            OptimizationType::Min => "min".to_string(),
            OptimizationType::Max => "max".to_string(),
        }
    }
}
