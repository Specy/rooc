#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
#[cfg(target_arch = "wasm32")]
use crate::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::pipe::run_pipe;
#[cfg(target_arch = "wasm32")]
use crate::runtime_builtin::JsFunction;
#[cfg(target_arch = "wasm32")]
use {
    crate::RoocParser,
    crate::parser::model_transformer::Model,
    crate::parser::pre_model::PreModel,
    crate::pipe::PipeContext,
    crate::pipe::pipe_definitions::{PipeDataType, PipeError, Pipeable, PipeableData},
    crate::pipe::pipe_executors::{
        AutoSolverPipe, BinarySolverPipe, CompilerPipe, IntegerBinarySolverPipe, LinearModelPipe,
        MILPSolverPipe, ModelPipe, Pipes, PreModelPipe, RealSolver, StandardLinearModelPipe,
        StepByStepSimplexPipe, TableauPipe,
    },
    crate::solvers::{OptimalTableau, OptimalTableauWithSteps, Tableau},
    crate::transformers::StandardLinearModel,
    crate::{Constant, Primitive},
};
#[cfg(target_arch = "wasm32")]
use crate::{IntOrBoolValue, LinearModel, LpSolution, MILPValue};

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct WasmPipeRunner {
    pipes: Vec<Box<dyn Pipeable>>,
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct JsPipable {
    function: Function,
}
#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl JsPipable {
    pub fn new_wasm(function: Function) -> JsPipable {
        JsPipable { function }
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
enum JsPipableData {
    String(String),
    LinearModel(LinearModel),
    BinarySolution(LpSolution<bool>),
    IntegerBinarySolution(LpSolution<IntOrBoolValue>),
    RealSolution(LpSolution<f64>),
    MILPSolution(LpSolution<MILPValue>),
}

#[cfg(target_arch = "wasm32")]
impl Pipeable for JsPipable {
    fn pipe(
        &self,
        data: &mut PipeableData,
        _pipe_context: &PipeContext,
    ) -> Result<PipeableData, PipeError> {
        let js_pipable = match data {
            PipeableData::String(s) => JsPipableData::String(s.clone()),
            PipeableData::LinearModel(lm) => JsPipableData::LinearModel(lm.clone()),
            PipeableData::BinarySolution(sol) => JsPipableData::BinarySolution(sol.clone()),
            PipeableData::IntegerBinarySolution(sol) => {
                JsPipableData::IntegerBinarySolution(sol.clone())
            }
            PipeableData::RealSolution(sol) => JsPipableData::RealSolution(sol.clone()),
            PipeableData::MILPSolution(sol) => JsPipableData::MILPSolution(sol.clone()),
            _ => return Err(PipeError::Other("JsPipable data must be one of: String, LinearModel, BinarySolution, IntegerBinarySolution, RealSolution, MILPSolution".to_string()))
        };
        let js_pipable =
            serialize_json_compatible(&js_pipable).map_err(|e| PipeError::Other(e.to_string()))?;
        let js_args = js_sys::Array::new();
        js_args.push(&js_pipable);
        let result = self.function.apply(&JsValue::NULL, &js_args).map_err(|e| {
            PipeError::Other(
                e.as_string()
                    .unwrap_or("Error thrown in function".to_string()),
            )
        })?;
        let result: JsPipableData =
            serde_wasm_bindgen::from_value(result).map_err(|e| PipeError::Other(e.to_string()))?;
        let pipe_result = match result {
            JsPipableData::String(s) => PipeableData::String(s),
            JsPipableData::LinearModel(lm) => PipeableData::LinearModel(lm),
            JsPipableData::BinarySolution(sol) => PipeableData::BinarySolution(sol),
            JsPipableData::IntegerBinarySolution(sol) => PipeableData::IntegerBinarySolution(sol),
            JsPipableData::RealSolution(sol) => PipeableData::RealSolution(sol),
            JsPipableData::MILPSolution(sol) => PipeableData::MILPSolution(sol),
        };
        Ok(pipe_result)
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl WasmPipeRunner {
    pub fn new_wasm() -> WasmPipeRunner {
        WasmPipeRunner { pipes: vec![] }
    }

    pub fn add_step_by_name(&mut self, step: Pipes) {
        let step: Box<dyn Pipeable> = match step {
            Pipes::CompilerPipe => Box::new(CompilerPipe::new()),
            Pipes::PreModelPipe => Box::new(PreModelPipe::new()),
            Pipes::ModelPipe => Box::new(ModelPipe::new()),
            Pipes::LinearModelPipe => Box::new(LinearModelPipe::new()),
            Pipes::StandardLinearModelPipe => Box::new(StandardLinearModelPipe::new()),
            Pipes::TableauPipe => Box::new(TableauPipe::new()),
            Pipes::RealPipe => Box::new(RealSolver::new()),
            Pipes::StepByStepSimplexPipe => Box::new(StepByStepSimplexPipe::new()),
            Pipes::BinarySolverPipe => Box::new(BinarySolverPipe::new()),
            Pipes::IntegerBinarySolverPipe => Box::new(IntegerBinarySolverPipe::new()),
            Pipes::MILPSolverPipe => Box::new(MILPSolverPipe::new()),
            Pipes::AutoSolverPipe => Box::new(AutoSolverPipe::new()),
        };
        self.pipes.push(step);
    }

    pub fn add_step_with_fn(&mut self, function: JsPipable) {
        self.pipes.push(Box::new(function));
    }

    pub fn wasm_run_from_string(
        &self,
        data: String,
        constants: JsValue,
        fns: Vec<JsFunction>,
    ) -> Result<Vec<WasmPipableData>, WasmPipeError> {
        let data = PipeableData::String(data);
        let constants: Vec<(String, Primitive)> = serde_wasm_bindgen::from_value(constants)
            .map_err(|e| WasmPipeError::new(PipeError::Other(e.to_string()), vec![]))?;
        let constants = constants
            .into_iter()
            .map(|v| Constant::from_primitive(&v.0, v.1))
            .collect();
        let fns = js_value_to_fns_map(fns);
        match run_pipe(&self.pipes, data, &PipeContext::new(constants, &fns)) {
            Ok(results) => Ok(results.into_iter().map(WasmPipableData::new).collect()),
            Err((e, results)) => {
                let results: Vec<WasmPipableData> =
                    results.into_iter().map(WasmPipableData::new).collect();
                Err(WasmPipeError::new(e, results))
            }
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
pub struct WasmPipeError {
    error: PipeError,
    context: Vec<WasmPipableData>,
}

#[cfg(target_arch = "wasm32")]
impl WasmPipeError {
    pub fn new(error: PipeError, context: Vec<WasmPipableData>) -> WasmPipeError {
        WasmPipeError { error, context }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct WasmPipableData {
    data: PipeableData,
}

#[cfg(target_arch = "wasm32")]
impl WasmPipableData {
    pub fn new(data: PipeableData) -> WasmPipableData {
        WasmPipableData { data }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<WasmPipableData> for PipeableData {
    fn from(data: WasmPipableData) -> Self {
        data.data
    }
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[allow(clippy::wrong_self_convention)]
#[cfg(target_arch = "wasm32")]
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

    pub fn to_real_solution(self) -> Result<JsValue, JsValue> {
        self.data
            .to_real_solution()
            .map(|s| serde_wasm_bindgen::to_value(&s).unwrap())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn to_milp_solution(self) -> Result<JsValue, JsValue> {
        self.data
            .to_milp_solution()
            .map(|s| serde_wasm_bindgen::to_value(&s).unwrap())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
