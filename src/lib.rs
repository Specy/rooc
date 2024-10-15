extern crate pest;
#[macro_use]
extern crate pest_derive;

use wasm_bindgen::prelude::*;

use parser::parser::{parse_problem_source, PreModel};
use utils::CompilationError;

use crate::parser::model_transformer::model::{transform_parsed_problem, Model};

pub mod macros;
pub mod math;
pub mod parser;
pub mod pipe;
pub mod primitives;
pub mod runtime_builtin;
pub mod solvers;
pub mod traits;
pub mod transformers;
pub mod type_checker;
pub mod utils;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct RoocParser {
    source: String,
}

impl RoocParser {
    pub fn new(source: String) -> Self {
        Self { source }
    }
    pub fn parse(&self) -> Result<PreModel, CompilationError> {
        parse_problem_source(&self.source)
    }
    pub fn format(&self) -> Result<String, CompilationError> {
        let parsed = self.parse()?;
        Ok(parsed.to_string())
    }
    pub fn parse_and_transform(&self) -> Result<Model, String> {
        let parsed = self
            .parse()
            .map_err(|e| e.to_string_from_source(&self.source))?;
        let transformed = transform_parsed_problem(parsed);
        match transformed {
            Ok(transformed) => Ok(transformed),
            Err(e) => Err(e
                .get_trace_from_source(&self.source)
                .unwrap_or(e.get_traced_error())),
        }
    }
    pub fn type_check(&self) -> Result<(), String> {
        let parsed = self
            .parse()
            .map_err(|e| e.to_string_from_source(&self.source))?;
        match parsed.create_type_checker() {
            Ok(_) => Ok(()),
            Err(e) => Err(e
                .get_trace_from_source(&self.source)
                .unwrap_or(e.get_traced_error())),
        }
    }
    pub fn hover_provider(&self, _line: usize, _column: usize, _offset: usize) {}
}

#[wasm_bindgen]
impl RoocParser {
    pub fn new_wasm(source: String) -> Self {
        Self::new(source)
    }
    pub fn format_wasm(&self) -> Result<String, CompilationError> {
        self.format()
    }
    pub fn parse_wasm(&self) -> Result<PreModel, CompilationError> {
        self.parse()
    }
    pub fn parse_and_transform_wasm(&self) -> Result<Model, String> {
        self.parse_and_transform()
    }

    pub fn wasm_get_source(&self) -> String {
        self.source.clone()
    }
}
