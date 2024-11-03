extern crate core;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use indexmap::IndexMap;
#[allow(unused_imports)]
use crate::prelude::*;

use parser::pre_model::{parse_problem_source, PreModel};
use utils::CompilationError;

use crate::parser::model_transformer::{transform_parsed_problem, Model};
use crate::runtime_builtin::{RoocFunction};


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


mod prelude {
    #[cfg(target_arch = "wasm32")]
    pub use {
        wasm_bindgen::prelude::*,
        wasm_bindgen::JsValue,
        serde_wasm_bindgen::*,
        crate::utils::serialize_json_compatible,
        js_sys::Function,
        crate::runtime_builtin::JsFunction,
        crate::parser::pre_model::js_value_to_fns_map,
        
    };
}



#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
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
    pub fn parse_and_transform(
        &self,
        fns: &IndexMap<String, Box<dyn RoocFunction>>,
    ) -> Result<Model, String> {
        let parsed = self
            .parse()
            .map_err(|e| e.to_string_from_source(&self.source))?;
        let transformed = transform_parsed_problem(parsed, fns);
        match transformed {
            Ok(transformed) => Ok(transformed),
            Err(e) => Err(e
                .trace_from_source(&self.source)
                .unwrap_or(e.traced_error())),
        }
    }
    pub fn type_check(&self, fns: &IndexMap<String, Box<dyn RoocFunction>>) -> Result<(), String> {
        let parsed = self
            .parse()
            .map_err(|e| e.to_string_from_source(&self.source))?;
        match parsed.create_type_checker(fns) {
            Ok(_) => Ok(()),
            Err(e) => Err(e
                .trace_from_source(&self.source)
                .unwrap_or(e.traced_error())),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
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
    pub fn parse_and_transform_wasm(&self, fns: Vec<JsFunction>) -> Result<Model, String> {
        let fns = js_value_to_fns_map(fns);
        self.parse_and_transform(&fns)
    }
    pub fn wasm_get_source(&self) -> String {
        self.source.clone()
    }
}
 