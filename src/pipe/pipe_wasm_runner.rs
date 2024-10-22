use crate::parser::model_transformer::model::Model;
use crate::parser::pre_model::PreModel;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use crate::pipe::pipe_definitions::{PipeDataType, PipeError, Pipeable, PipeableData};
use crate::pipe::pipe_executors::{
    BinarySolverPipe, CompilerPipe, IntegerBinarySolverPipe, LinearModelPipe, ModelPipe, Pipes,
    PreModelPipe, SimplexPipe, StandardLinearModelPipe, StepByStepSimplexPipe, TableauPipe,
};
use crate::pipe::pipe_runner::PipeRunner;
use crate::solvers::simplex::{OptimalTableau, OptimalTableauWithSteps, Tableau};
use crate::transformers::linear_model::LinearModel;
use crate::transformers::standard_linear_model::StandardLinearModel;
use crate::RoocParser;

#[wasm_bindgen]
#[allow(unused)]
struct WasmPipeRunner {
    pipe: PipeRunner,
}

#[wasm_bindgen]
#[allow(unused)]
impl WasmPipeRunner {
    pub fn new_wasm(steps: Vec<Pipes>) -> Result<WasmPipeRunner, String> {
        let runners = steps
            .into_iter()
            .map(|step| {
                let item: Box<dyn Pipeable> = match step {
                    Pipes::CompilerPipe => Box::new(CompilerPipe::new()),
                    Pipes::PreModelPipe => Box::new(PreModelPipe::new()),
                    Pipes::ModelPipe => Box::new(ModelPipe::new()),
                    Pipes::LinearModelPipe => Box::new(LinearModelPipe::new()),
                    Pipes::StandardLinearModelPipe => Box::new(StandardLinearModelPipe::new()),
                    Pipes::TableauPipe => Box::new(TableauPipe::new()),
                    Pipes::SimplexPipe => Box::new(SimplexPipe::new()),
                    Pipes::StepByStepSimplexPipe => Box::new(StepByStepSimplexPipe::new()),
                    Pipes::BinarySolverPipe => Box::new(BinarySolverPipe::new()),
                    Pipes::IntegerBinarySolverPipe => Box::new(IntegerBinarySolverPipe::new()),
                };
                item
            })
            .collect();
        Ok(WasmPipeRunner {
            pipe: PipeRunner::new(runners),
        })
    }

    pub fn wasm_run_from_string(
        &self,
        data: String,
    ) -> Result<Vec<WasmPipableData>, WasmPipeError> {
        let data = PipeableData::String(data);
        match self.pipe.run(data) {
            Ok(results) => Ok(results.into_iter().map(WasmPipableData::new).collect()),
            Err((e, results)) => {
                let results: Vec<WasmPipableData> =
                    results.into_iter().map(WasmPipableData::new).collect();
                Err(WasmPipeError::new(e, results))
            }
        }
    }
}

#[wasm_bindgen]
pub struct WasmPipeError {
    error: PipeError,
    context: Vec<WasmPipableData>,
}

impl WasmPipeError {
    pub fn new(error: PipeError, context: Vec<WasmPipableData>) -> WasmPipeError {
        WasmPipeError { error, context }
    }
}

#[wasm_bindgen]
impl WasmPipeError {
    pub fn wasm_get_error(&self) -> String {
        self.error.to_string()
    }
    pub fn wasm_get_context(&self) -> Vec<WasmPipableData> {
        self.context.clone()
    }
    pub fn wasm_to_context(self) -> Vec<WasmPipableData> {
        self.context
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmPipableData {
    data: PipeableData,
}
impl WasmPipableData {
    pub fn new(data: PipeableData) -> WasmPipableData {
        WasmPipableData { data }
    }
}
impl From<WasmPipableData> for PipeableData {
    fn from(data: WasmPipableData) -> Self {
        data.data
    }
}
#[wasm_bindgen]
#[allow(clippy::wrong_self_convention)]
impl WasmPipableData {
    pub fn wasm_get_type(&self) -> PipeDataType {
        self.data.get_type()
    }

    //TODO is there a better way to do this instead of making singular functions for each type?
    pub fn to_string_data(self) -> Result<String, JsValue> {
        self.data
            .to_string_data()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn to_parser(self) -> Result<RoocParser, JsValue> {
        self.data
            .to_parser()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn to_pre_model(self) -> Result<PreModel, JsValue> {
        self.data
            .to_pre_model()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn to_model(self) -> Result<Model, JsValue> {
        self.data
            .to_model()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn to_linear_model(self) -> Result<LinearModel, JsValue> {
        self.data
            .to_linear_model()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn to_standard_linear_model(self) -> Result<StandardLinearModel, JsValue> {
        self.data
            .to_standard_linear_model()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn to_tableau(self) -> Result<Tableau, JsValue> {
        self.data
            .to_tableau()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn to_optimal_tableau(self) -> Result<OptimalTableau, JsValue> {
        self.data
            .to_optimal_tableau()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn to_optimal_tableau_with_steps(self) -> Result<OptimalTableauWithSteps, JsValue> {
        self.data
            .to_optimal_tableau_with_steps()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn to_binary_solution(self) -> Result<JsValue, JsValue> {
        self.data
            .to_binary_solution()
            .map(|s| serde_wasm_bindgen::to_value(&s).unwrap())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn to_integer_binary_solution(self) -> Result<JsValue, JsValue> {
        self.data
            .to_integer_binary_solution()
            .map(|s| serde_wasm_bindgen::to_value(&s).unwrap())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
