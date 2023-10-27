use std::fmt::format;

#[derive(Debug, PartialEq, Clone)]
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
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}
impl Operator {
    pub fn precedence(&self) -> u8 {
        match self {
            Operator::Add => 1,
            Operator::Sub => 1,
            Operator::Mul => 2,
            Operator::Div => 2,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Operator::Add => "+".to_string(),
            Operator::Sub => "-".to_string(),
            Operator::Mul => "*".to_string(),
            Operator::Div => "/".to_string(),
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

#[derive(Debug, Clone)]
pub enum ConstantValue {
    Number(f64),
    OneDimArray(Vec<f64>),
    TwoDimArray(Vec<Vec<f64>>),
}
impl ConstantValue {
    pub fn to_string(&self) -> String {
        match self {
            Self::Number(n) => n.to_string(),
            Self::OneDimArray(v) => format!("{:?}", v),
            Self::TwoDimArray(v) => {
                let result = v.iter().map(|row| format!("{:?}", row)).collect::<Vec<String>>();
                format!("[{}]", result.join(",\n"))
            }
        }
    }
}

#[derive(Debug)]
pub struct Constant {
    pub name: String,
    pub value: ConstantValue,
}
impl Constant {
    pub fn new(name: String, value: ConstantValue) -> Self {
        Self { name, value }
    }
    pub fn to_string(&self) -> String {
        format!("{} = {}", self.name, self.value.to_string())
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String),
    MissingToken(String),
}
