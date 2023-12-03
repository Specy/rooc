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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}
impl Op {
    pub fn precedence(&self) -> u8 {
        match self {
            Op::Add => 1,
            Op::Sub => 1,
            Op::Mul => 2,
            Op::Div => 2,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Op::Add => "+".to_string(),
            Op::Sub => "-".to_string(),
            Op::Mul => "*".to_string(),
            Op::Div => "/".to_string(),
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
