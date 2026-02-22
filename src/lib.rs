//! # ROOC
//! ROOC is a modeling language for defining and solving optimization problems.
//!
//! Write everything as a single source file:
//! ```rust
//!use indexmap::IndexMap;
//!use rooc::pipe::{AutoSolverPipe, LinearModelPipe, ModelPipe, PipeContext, PipeRunner, PipeableData, PreModelPipe};
//!let source = "
//!max sum((value, i) in enumerate(values)) { value * x_i }
//!s.t.
//!    sum((weight, i) in enumerate(weights)) { weight * x_i } <= capacity
//!where
//!    let weights = [10, 60, 30, 40, 30, 20, 20, 2]
//!    let values = [1, 10, 15, 40, 60, 90, 100, 15]
//!    let capacity = 102
//!define
//!    x_i as Boolean for i in 0..len(weights)";
//!//use pipes to solve the problem
//!let pipe_runner = PipeRunner::new(vec![
//!    Box::new(rooc::pipe::CompilerPipe::new()),
//!    Box::new(PreModelPipe::new()),
//!    Box::new(ModelPipe::new()),
//!    Box::new(LinearModelPipe::new()),
//!    Box::new(AutoSolverPipe::new()),
//!]);
//!let result = pipe_runner
//!    .run(
//!        PipeableData::String(source.to_string()),
//!        &PipeContext::new(vec![], &IndexMap::new()),
//!    )
//!    .unwrap();
//!let last = result
//!    .into_iter()
//!    .last()
//!    .unwrap()
//!    .to_milp_solution()
//!        .unwrap();
//!
//!println!("{}", last)
//! ```
//! Or extend the language with your own functionality and data

extern crate core;
extern crate pest;
#[macro_use]
extern crate pest_derive;

#[allow(unused_imports)]
use crate::prelude::*;
use indexmap::IndexMap;
use std::fmt::{Display, Formatter};

use parser::pre_model::{PreModel, parse_problem_source};

use crate::parser::model_transformer::{Model, transform_parsed_problem};

#[macro_use]
mod macros;
mod math;
mod parser;
pub mod pipe;
mod primitives;
mod runtime_builtin;
mod solvers;
mod traits;
mod transformers;
pub mod type_checker;
mod utils;

use crate::model_transformer::TransformError;
pub use math::*;
pub use parser::*;
pub use primitives::*;
pub use runtime_builtin::*;
pub use solvers::*;
pub use transformers::*;
pub use utils::*;

mod prelude {
    #[cfg(target_arch = "wasm32")]
    pub use {
        crate::parser::pre_model::js_value_to_fns_map, crate::utils::serialize_json_compatible,
        js_sys::Function, serde_wasm_bindgen, wasm_bindgen::JsValue, wasm_bindgen::prelude::*,
    };
}

/// A parser for the Rooc optimization modeling language.
///
/// This struct provides functionality to parse, format, transform and type check
/// Rooc source code. It serves as the main entry point for processing Rooc programs.
///
/// # Example
/// ```rust
///    use indexmap::IndexMap;
///    use rooc::{Linearizer, RoocParser, solve_integer_binary_lp_problem};
///
///    let source = "
///    max sum((value, i) in enumerate(values)) { value * x_i }
///    s.t.
///        sum((weight, i) in enumerate(weights)) { weight * x_i } <= capacity
///    where
///        let weights = [10, 60, 30, 40, 30, 20, 20, 2]
///        let values = [1, 10, 15, 40, 60, 90, 100, 15]
///        let capacity = 102
///    define
///        x_i as Boolean for i in 0..len(weights)";
///    let rooc = RoocParser::new(source.to_string());
///    let parsed = rooc.parse().unwrap();
///    let model = parsed.transform(vec![], &IndexMap::new()).unwrap();
///    let linear = Linearizer::linearize(model).unwrap();
///    let solution = solve_integer_binary_lp_problem(&linear).unwrap();
///    println!("{}", solution)
/// ```
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct RoocParser {
    source: String,
}

impl RoocParser {
    /// Creates a new RoocParser instance with the given source code.
    ///
    /// # Arguments
    /// * `source` - The Rooc source code as a String
    pub fn new(source: String) -> Self {
        Self { source }
    }

    /// Parses the source code into a PreModel representation.
    ///
    /// # Returns
    /// * `Ok(PreModel)` - The parsed representation of the program
    /// * `Err(CompilationError)` - If parsing fails
    pub fn parse(&self) -> Result<PreModel, CompilationError> {
        parse_problem_source(&self.source)
    }

    /// Formats the source code according to Rooc's formatting rules.
    ///
    /// # Returns
    /// * `Ok(String)` - The formatted source code
    /// * `Err(CompilationError)` - If parsing fails during formatting
    pub fn format(&self) -> Result<String, CompilationError> {
        let parsed = self.parse()?;
        Ok(parsed.to_string())
    }

    /// Parses and transforms the source code into a Model representation.
    ///
    /// # Arguments
    /// * `constants` - Vector of constant definitions to be used during transformation
    /// * `fns` - Map of function names to their implementations
    ///
    /// # Returns
    /// * `Ok(Model)` - The transformed model
    /// * `Err(String)` - Error message if parsing or transformation fails
    pub fn parse_and_transform(
        &self,
        constants: Vec<Constant>,
        fns: &IndexMap<String, Box<dyn RoocFunction>>,
    ) -> Result<Model, String> {
        let parsed = self
            .parse()
            .map_err(|e| e.to_string_from_source(&self.source))?;
        let transformed = transform_parsed_problem(parsed, constants, fns);
        match transformed {
            Ok(transformed) => Ok(transformed),
            Err(e) => Err(e
                .trace_from_source(&self.source)
                .unwrap_or(e.traced_error())),
        }
    }

    /// Type checks the source code against provided constants and functions.
    ///
    /// # Arguments
    /// * `constants` - Vector of constants to check against
    /// * `fns` - Map of function names to their implementations
    ///
    /// # Returns
    /// * `Ok(())` - If type checking succeeds
    /// * `Err(String)` - Error message if type checking fails
    pub fn type_check(
        &self,
        constants: &Vec<Constant>,
        fns: &FunctionContextMap,
    ) -> Result<(), String> {
        let parsed = self
            .parse()
            .map_err(|e| e.to_string_from_source(&self.source))?;
        match parsed.create_type_checker(constants, fns) {
            Ok(_) => Ok(()),
            Err(e) => Err(e
                .trace_from_source(&self.source)
                .unwrap_or(e.traced_error())),
        }
    }
}
pub type FunctionContextMap = IndexMap<String, Box<dyn RoocFunction>>;
pub struct RoocSolver {
    model: PreModel,
}

#[derive(Debug)]
pub enum RoocSolverError<T> {
    Transform(TransformError),
    Linearization(LinearizationError),
    Solver(T),
}
impl<T: Display> Display for RoocSolverError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transform(t) => write!(f, "{}", t),
            Self::Linearization(l) => write!(f, "{}", l),
            Self::Solver(s) => write!(f, "{}", s),
        }
    }
}
impl RoocSolver {
    pub fn try_new(source: String) -> Result<Self, CompilationError> {
        let parser = RoocParser::new(source.clone());
        let model = parser.parse()?;
        Ok(RoocSolver { model })
    }

    pub fn solve_with_data_using<T, E, F>(
        self,
        func: F,
        constants: Vec<Constant>,
        fns: &FunctionContextMap,
    ) -> Result<T, RoocSolverError<E>>
    where
        F: Fn(&LinearModel) -> Result<T, E>,
    {
        self.model
            .create_type_checker(&constants, fns)
            .map_err(|e| RoocSolverError::Transform(e))?;
        let compiled = self
            .model
            .transform(constants, fns)
            .map_err(|e| RoocSolverError::Transform(e))?;
        let linearized =
            Linearizer::linearize(compiled).map_err(|e| RoocSolverError::Linearization(e))?;
        let result = func(&linearized).map_err(|e| RoocSolverError::Solver(e))?;
        Ok(result)
    }

    pub fn solve_using<T, E, F>(self, func: F) -> Result<T, RoocSolverError<E>>
    where
        F: Fn(&LinearModel) -> Result<T, E>,
    {
        self.solve_with_data_using(func, vec![], &IndexMap::new())
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
    pub fn parse_and_transform_wasm(
        &self,
        constants: JsValue,
        fns: Vec<JsFunction>,
    ) -> Result<Model, String> {
        let constants: Vec<(String, Primitive)> =
            serde_wasm_bindgen::from_value(constants).map_err(|e| e.to_string())?;
        let constants = constants
            .into_iter()
            .map(|v| Constant::from_primitive(&v.0, v.1))
            .collect();
        let fns = js_value_to_fns_map(fns);
        self.parse_and_transform(constants, &fns)
    }
    pub fn wasm_get_source(&self) -> String {
        self.source.clone()
    }
}
