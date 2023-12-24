use std::str::FromStr;

use crate::enum_with_variants_to_string;

enum_with_variants_to_string! {
    pub enum Operator derives[Debug, PartialEq, Clone, Copy]  {
        Add,
        Sub,
        Mul,
        Div,
        Neg,
    }
}
impl Operator {
    pub fn to_string(&self) -> String {
        match self {
            Operator::Add => "+".to_string(),
            Operator::Sub => "-".to_string(),
            Operator::Mul => "*".to_string(),
            Operator::Div => "/".to_string(),
            Operator::Neg => "-".to_string(),
        }
    }
    pub fn precedence(&self) -> u8 {
        match self {
            Operator::Add | Operator::Sub => 1,
            Operator::Mul | Operator::Div => 2,
            Operator::Neg => 3,
        }
    }
    pub fn is_left_associative(&self) -> bool {
        match self {
            Operator::Add | Operator::Sub | Operator::Mul | Operator::Div => true,
            Operator::Neg => false,
        }
    }
}


enum_with_variants_to_string! {
    pub enum BinOp derives[Debug, PartialEq, Clone, Copy] {
        Add,
        Sub,
        Mul,
        Div,
        //And
        //Or
        //Not
        //Xor
    }
}

impl BinOp {
    pub fn to_string(&self) -> String {
        match self {
            BinOp::Add => "+".to_string(),
            BinOp::Sub => "-".to_string(),
            BinOp::Mul => "*".to_string(),
            BinOp::Div => "/".to_string(),
        }
    }
    pub fn precedence(&self) -> u8 {
        match self {
            BinOp::Add | BinOp::Sub => 1,
            BinOp::Mul | BinOp::Div => 2,
        }
    }
    pub fn is_left_associative(&self) -> bool {
        match self {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => true,
        }
    }
    pub fn to_operator(&self) -> Operator {
        match self {
            BinOp::Add => Operator::Add,
            BinOp::Sub => Operator::Sub,
            BinOp::Mul => Operator::Mul,
            BinOp::Div => Operator::Div,
        }
    }
}

impl FromStr for BinOp {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(BinOp::Add),
            "-" => Ok(BinOp::Sub),
            "*" => Ok(BinOp::Mul),
            "/" => Ok(BinOp::Div),
            _ => Err(()),
        }
    }
}

enum_with_variants_to_string! {
    pub enum UnOp derives[Debug, PartialEq, Clone, Copy] {
        Neg,
    }
}
impl UnOp {
    pub fn to_string(&self) -> String {
        match self {
            UnOp::Neg => "-".to_string(),
        }
    }
    pub fn precedence(&self) -> u8 {
        match self {
            UnOp::Neg => 3,
        }
    }
    pub fn is_left_associative(&self) -> bool {
        match self {
            UnOp::Neg => false,
        }
    }
    pub fn to_operator(&self) -> Operator {
        match self {
            UnOp::Neg => Operator::Neg,
        }
    }
}

impl FromStr for UnOp {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" => Ok(UnOp::Neg),
            _ => Err(()),
        }
    }
}

pub trait ApplyOp {
    type Target;
    type Error;
    fn apply_binary_op(&self, op: BinOp, to: &Self::Target) -> Result<Self::Target, Self::Error>;
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error>;
}
