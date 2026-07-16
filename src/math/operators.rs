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
        And,
        Or,
        Xor,
        Implies,
        Iff,
        Not,
    }
}
impl Operator {
    /// Returns the precedence level of the operator.
    ///
    /// Higher precedence values indicate that the operator should be evaluated first.
    pub fn precedence(&self) -> u8 {
        match self {
            Operator::Implies | Operator::Iff => 1,
            Operator::Or => 2,
            Operator::Xor => 3,
            Operator::And => 4,
            Operator::Add | Operator::Sub => 5,
            Operator::Mul | Operator::Div => 6,
            Operator::Neg | Operator::Not => 7,
        }
    }

    /// Determines if the operator is left associative.
    ///
    /// Left associative operators are evaluated from left to right.
    /// For example, a - b - c is evaluated as (a - b) - c.
    pub fn is_left_associative(&self) -> bool {
        match self {
            Operator::Add | Operator::Sub | Operator::Mul | Operator::Div => true,
            Operator::And | Operator::Or | Operator::Xor | Operator::Iff => true,
            Operator::Implies => false,
            Operator::Neg | Operator::Not => false,
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
            Operator::And => "and".to_string(),
            Operator::Or => "or".to_string(),
            Operator::Xor => "xor".to_string(),
            Operator::Implies => "implies".to_string(),
            Operator::Iff => "iff".to_string(),
            Operator::Not => "not".to_string(),
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
        And,
        Or,
        Xor,
        Implies,
        Iff,
    }
}

impl BinOp {
    /// Returns the precedence level of the binary operator.
    pub fn precedence(&self) -> u8 {
        match self {
            BinOp::Implies | BinOp::Iff => 1,
            BinOp::Or => 2,
            BinOp::Xor => 3,
            BinOp::And => 4,
            BinOp::Add | BinOp::Sub => 5,
            BinOp::Mul | BinOp::Div => 6,
        }
    }

    /// Determines if the binary operator is left associative.
    pub fn is_left_associative(&self) -> bool {
        match self {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => true,
            BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Iff => true,
            BinOp::Implies => false,
        }
    }

    /// Returns true if the operator is a logic operator.
    pub fn is_logic(&self) -> bool {
        match self {
            BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff => true,
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => false,
        }
    }

    /// Converts a binary operator to the corresponding general Operator enum.
    pub fn to_operator(&self) -> Operator {
        match self {
            BinOp::Add => Operator::Add,
            BinOp::Sub => Operator::Sub,
            BinOp::Mul => Operator::Mul,
            BinOp::Div => Operator::Div,
            BinOp::And => Operator::And,
            BinOp::Or => Operator::Or,
            BinOp::Xor => Operator::Xor,
            BinOp::Implies => Operator::Implies,
            BinOp::Iff => Operator::Iff,
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
            BinOp::And => "\\land".to_string(),
            BinOp::Or => "\\lor".to_string(),
            BinOp::Xor => "\\oplus".to_string(),
            BinOp::Implies => "\\Rightarrow".to_string(),
            BinOp::Iff => "\\Leftrightarrow".to_string(),
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
            BinOp::And => "and".to_string(),
            BinOp::Or => "or".to_string(),
            BinOp::Xor => "xor".to_string(),
            BinOp::Implies => "implies".to_string(),
            BinOp::Iff => "iff".to_string(),
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
            "and" => Ok(BinOp::And),
            "or" => Ok(BinOp::Or),
            "xor" => Ok(BinOp::Xor),
            "implies" => Ok(BinOp::Implies),
            "iff" => Ok(BinOp::Iff),
            _ => Err(()),
        }
    }
}

enum_with_variants_to_string! {
    pub enum UnOp derives[Debug, PartialEq, Clone, Copy] with_wasm {
        Neg,
        Not,
    }
}

impl UnOp {
    /// Returns the precedence level of the unary operator.
    pub fn precedence(&self) -> u8 {
        match self {
            UnOp::Neg | UnOp::Not => 7,
        }
    }

    /// Determines if the unary operator is left associative.
    pub fn is_left_associative(&self) -> bool {
        match self {
            UnOp::Neg | UnOp::Not => false,
        }
    }

    /// Converts a unary operator to the corresponding general Operator enum.
    pub fn to_operator(&self) -> Operator {
        match self {
            UnOp::Neg => Operator::Neg,
            UnOp::Not => Operator::Not,
        }
    }
}

impl ToLatex for UnOp {
    fn to_latex(&self) -> String {
        match self {
            UnOp::Neg => "-".to_string(),
            UnOp::Not => "\\lnot ".to_string(),
        }
    }
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UnOp::Neg => "-".to_string(),
            UnOp::Not => "not ".to_string(),
        };

        f.write_str(&s)
    }
}

impl FromStr for UnOp {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" => Ok(UnOp::Neg),
            "not" => Ok(UnOp::Not),
            _ => Err(()),
        }
    }
}
