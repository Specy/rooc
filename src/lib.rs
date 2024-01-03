extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod macros;
pub mod math;
pub mod parser;
pub mod primitives;
pub mod solvers;
pub mod transformers;
pub mod utils;

use parser::{
    parser::{parse_problem_source, PreProblem},
    transformer::{transform_parsed_problem, Problem},
};
use wasm_bindgen::prelude::*;
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct RoocParser {
    source: String,
}

impl RoocParser {
    pub fn new(source: String) -> Self {
        Self { source }
    }
    pub fn parse(&self) -> Result<PreProblem, String> {
        parse_problem_source(&self.source)
    }
    pub fn parse_and_transform(&self) -> Result<Problem, String> {
        let parsed = self.parse()?;
        let transformed = transform_parsed_problem(&parsed);
        match transformed {
            Ok(transformed) => Ok(transformed),
            Err(e) => Err(e
                .get_trace_from_source(&self.source)
                .unwrap_or(e.get_traced_error())),
        }
    }
}

#[wasm_bindgen]
impl RoocParser {
    pub fn new_wasm(source: String) -> Self {
        Self::new(source)
    }
    pub fn verify_wasm(&self) -> Result<(), String> {
        self.parse_and_transform_wasm().map(|_| ())
    }
    pub fn parse_and_transform_wasm(&self) -> Result<Problem, String> {
        self.parse_and_transform()
    }
}
