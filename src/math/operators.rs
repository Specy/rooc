use std::str::FromStr;

use crate::enum_with_variants_to_string;

enum_with_variants_to_string!{
    pub enum Op derives[Debug, PartialEq, Clone, Copy] {
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
impl FromStr for Op {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(Op::Add),
            "-" => Ok(Op::Sub),
            "*" => Ok(Op::Mul),
            "/" => Ok(Op::Div),
            _ => Err(()),
        }
    }
}


enum_with_variants_to_string!{
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
    fn apply_binary_op(
        &self,
        op: Op,
        to: &Self::Target,
    ) -> Result<Self::Target, Self::Error>;
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error>;
}

