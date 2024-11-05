use core::fmt;
use std::str::FromStr;

#[allow(unused_imports)]
use crate::prelude::*;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

use crate::enum_with_variants_to_string;
use crate::traits::ToLatex;

enum_with_variants_to_string! {
    pub enum Operator derives[Debug, PartialEq, Clone, Copy] with_wasm {
        Add,
        Sub,
        Mul,
        Div,
        Neg,
    }
}
impl Operator {
    /// Returns the precedence level of the operator.
    ///
    /// Higher precedence values indicate that the operator should be evaluated first.
    pub fn precedence(&self) -> u8 {
        match self {
            Operator::Add | Operator::Sub => 1,
            Operator::Mul | Operator::Div => 2,
            Operator::Neg => 3,
        }
    }

    /// Determines if the operator is left associative.
    ///
    /// Left associative operators are evaluated from left to right.
    /// For example, a - b - c is evaluated as (a - b) - c.
    pub fn is_left_associative(&self) -> bool {
        match self {
            Operator::Add | Operator::Sub | Operator::Mul | Operator::Div => true,
            Operator::Neg => false,
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Operator::Add => "+".to_string(),
            Operator::Sub => "-".to_string(),
            Operator::Mul => "*".to_string(),
            Operator::Div => "/".to_string(),
            Operator::Neg => "-".to_string(),
        };

        f.write_str(&s)
    }
}

enum_with_variants_to_string! {
    pub enum BinOp derives[Debug, PartialEq, Clone, Copy] with_wasm {
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
    /// Returns the precedence level of the binary operator.
    pub fn precedence(&self) -> u8 {
        match self {
            BinOp::Add | BinOp::Sub => 1,
            BinOp::Mul | BinOp::Div => 2,
        }
    }

    /// Determines if the binary operator is left associative.
    pub fn is_left_associative(&self) -> bool {
        match self {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => true,
        }
    }

    /// Converts a binary operator to the corresponding general Operator enum.
    pub fn to_operator(&self) -> Operator {
        match self {
            BinOp::Add => Operator::Add,
            BinOp::Sub => Operator::Sub,
            BinOp::Mul => Operator::Mul,
            BinOp::Div => Operator::Div,
        }
    }
}

impl ToLatex for BinOp {
    fn to_latex(&self) -> String {
        match self {
            BinOp::Add => "+".to_string(),
            BinOp::Sub => "-".to_string(),
            BinOp::Mul => "\\cdot".to_string(),
            BinOp::Div => "\\div".to_string(),
        }
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinOp::Add => "+".to_string(),
            BinOp::Sub => "-".to_string(),
            BinOp::Mul => "*".to_string(),
            BinOp::Div => "/".to_string(),
        };

        f.write_str(&s)
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
    pub enum UnOp derives[Debug, PartialEq, Clone, Copy] with_wasm {
        Neg,
    }
}

impl UnOp {
    /// Returns the precedence level of the unary operator.
    pub fn precedence(&self) -> u8 {
        match self {
            UnOp::Neg => 3,
        }
    }

    /// Determines if the unary operator is left associative.
    pub fn is_left_associative(&self) -> bool {
        match self {
            UnOp::Neg => false,
        }
    }

    /// Converts a unary operator to the corresponding general Operator enum.
    pub fn to_operator(&self) -> Operator {
        match self {
            UnOp::Neg => Operator::Neg,
        }
    }
}

impl ToLatex for UnOp {
    fn to_latex(&self) -> String {
        match self {
            UnOp::Neg => "-".to_string(),
        }
    }
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UnOp::Neg => "-".to_string(),
        };

        f.write_str(&s)
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
