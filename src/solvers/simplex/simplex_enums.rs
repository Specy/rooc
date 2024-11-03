#[allow(unused_imports)]
use crate::prelude::*;
use core::fmt;
use serde::Serialize;
use std::fmt::Display;

#[derive(Serialize)]
pub enum StepAction {
    Pivot {
        entering: usize,
        leaving: usize,
        ratio: f64,
    },
    Finished,
}

#[derive(Debug)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub enum SimplexError {
    Unbounded,
    IterationLimitReached,
    Other,
}
impl Display for SimplexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SimplexError::Unbounded => "Unbounded Problem",
            SimplexError::IterationLimitReached => "Iteration Limit Reached",
            SimplexError::Other => "Other",
        };
        f.write_str(s)
    }
}

#[derive(Debug)]
pub enum CanonicalTransformError {
    Raw(String),
    InvalidBasis(String),
    Infesible(String),
    SimplexError(String),
}

impl Display for CanonicalTransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Raw(s) => s.clone(),
            Self::InvalidBasis(s) => format!("Invalid Basis: {}", s),
            Self::Infesible(s) => format!("Infesible: {}", s),
            Self::SimplexError(s) => format!("Simplex Error: {}", s),
        };
        f.write_str(&s)
    }
}
