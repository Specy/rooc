use serde::de::value::Error;

use crate::primitives::{
    graph::{Graph, GraphEdge, GraphNode},
    iterable::IterableKind,
    primitive::{Primitive, Tuple},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    //And
    //Or
    //Not
    //Xor
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


pub trait ApplyOp {
    type Target;
    type Error;
    fn apply_op(
        &self,
        op: Op,
        to: &Self::Target,
    ) -> Result<Self::Target, Self::Error>;
}

