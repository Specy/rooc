use std::fmt::Display;

use wasm_bindgen::prelude::wasm_bindgen;

use crate::parser::model_transformer::model::Model;
use crate::parser::model_transformer::transform_error::TransformError;
use crate::parser::parser::PreModel;
use crate::solvers::simplex::{
    CanonicalTransformError, OptimalTableau, SimplexError, OptimalTableauWithSteps, Tableau,
};
use crate::transformers::linear_model::LinearModel;
use crate::transformers::linearizer::LinearizationError;
use crate::transformers::standard_linear_model::StandardLinearModel;
use crate::utils::CompilationError;
use crate::{match_pipe_data_to, RoocParser};
use crate::solvers::binary::{BinaryLpSolution, BinarySolverError};

#[derive(Debug, Clone)]
pub enum PipeableData {
    String(String),
    Parser(RoocParser),
    PreModel(PreModel),
    Model(Model),
    LinearModel(LinearModel),
    StandardLinearModel(StandardLinearModel),
    Tableau(Tableau),
    OptimalTableau(OptimalTableau),
    OptimalTableauWithSteps(OptimalTableauWithSteps),
    BinarySolution(BinaryLpSolution)
}

impl PipeableData {
    pub fn get_type(&self) -> PipeDataType {
        match self {
            PipeableData::String(_) => PipeDataType::String,
            PipeableData::Parser(_) => PipeDataType::Parser,
            PipeableData::Model(_) => PipeDataType::Model,
            PipeableData::LinearModel(_) => PipeDataType::LinearModel,
            PipeableData::StandardLinearModel(_) => PipeDataType::StandardLinearModel,
            PipeableData::Tableau(_) => PipeDataType::Tableau,
            PipeableData::PreModel(_) => PipeDataType::PreModel,
            PipeableData::OptimalTableau(_) => PipeDataType::OptimalTableau,
            PipeableData::OptimalTableauWithSteps(_) => PipeDataType::OptimalTableauWithSteps,
            PipeableData::BinarySolution(_) => PipeDataType::BinarySolution,
        }
    }
    //TODO make this macros
    pub fn to_string_data(self) -> Result<String, PipeError> {
        match_pipe_data_to!(self, String, String)
    }
    pub fn to_parser(self) -> Result<RoocParser, PipeError> {
        match_pipe_data_to!(self, Parser, Parser)
    }
    pub fn to_pre_model(self) -> Result<PreModel, PipeError> {
        match_pipe_data_to!(self, PreModel, PreModel)
    }
    pub fn to_model(self) -> Result<Model, PipeError> {
        match_pipe_data_to!(self, Model, Model)
    }
    pub fn to_linear_model(self) -> Result<LinearModel, PipeError> {
        match_pipe_data_to!(self, LinearModel, LinearModel)
    }
    pub fn to_standard_linear_model(self) -> Result<StandardLinearModel, PipeError> {
        match_pipe_data_to!(self, StandardLinearModel, StandardLinearModel)
    }
    pub fn to_tableau(self) -> Result<Tableau, PipeError> {
        match_pipe_data_to!(self, Tableau, Tableau)
    }
    pub fn to_optimal_tableau(self) -> Result<OptimalTableau, PipeError> {
        match_pipe_data_to!(self, OptimalTableau, OptimalTableau)
    }
    pub fn to_optimal_tableau_with_steps(self) -> Result<OptimalTableauWithSteps, PipeError> {
        match_pipe_data_to!(self, OptimalTableauWithSteps, OptimalTableauWithSteps)
    }
    pub fn to_binary_solution(self) -> Result<BinaryLpSolution, PipeError> {
        match_pipe_data_to!(self, BinarySolution, BinarySolution)
    }
    pub fn as_string_data(&self) -> Result<&String, PipeError> {
        match_pipe_data_to!(self, String, String)
    }
    pub fn as_binary_solution(&self) -> Result<&BinaryLpSolution, PipeError> {
        match_pipe_data_to!(self, BinarySolution, BinarySolution)
    }
    
    pub fn as_parser(&self) -> Result<&RoocParser, PipeError> {
        match_pipe_data_to!(self, Parser, Parser)
    }
    pub fn as_pre_model(&self) -> Result<&PreModel, PipeError> {
        match_pipe_data_to!(self, PreModel, PreModel)
    }
    pub fn as_model(&self) -> Result<&Model, PipeError> {
        match_pipe_data_to!(self, Model, Model)
    }
    pub fn as_linear_model(&self) -> Result<&LinearModel, PipeError> {
        match_pipe_data_to!(self, LinearModel, LinearModel)
    }
    pub fn as_standard_linear_model(&self) -> Result<&StandardLinearModel, PipeError> {
        match_pipe_data_to!(self, StandardLinearModel, StandardLinearModel)
    }
    pub fn as_tableau(&self) -> Result<&Tableau, PipeError> {
        match_pipe_data_to!(self, Tableau, Tableau)
    }
    pub fn as_optimal_tableau(&self) -> Result<&OptimalTableau, PipeError> {
        match_pipe_data_to!(self, OptimalTableau, OptimalTableau)
    }
}

impl Display for PipeableData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipeableData::String(s) => write!(f, "{}", s),
            PipeableData::Parser(p) => write!(f, "{}", p.source),
            PipeableData::PreModel(p) => write!(f, "{}", p),
            PipeableData::Model(m) => write!(f, "{}", m),
            PipeableData::LinearModel(m) => write!(f, "{}", m),
            PipeableData::StandardLinearModel(m) => write!(f, "{}", m),
            PipeableData::Tableau(t) => write!(f, "{}", t),
            PipeableData::OptimalTableau(t) => write!(f, "{}", t),
            PipeableData::OptimalTableauWithSteps(t) => write!(f, "{:?}", t),
            PipeableData::BinarySolution(s) => write!(f, "{:?}", s),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub enum PipeDataType {
    String,
    Parser,
    PreModel,
    Model,
    LinearModel,
    StandardLinearModel,
    Tableau,
    OptimalTableau,
    OptimalTableauWithSteps,
    BinarySolution
}
impl Display for PipeDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PipeDataType::String => "String".to_string(),
            PipeDataType::Parser => "Parser".to_string(),
            PipeDataType::PreModel => "PreModel".to_string(),
            PipeDataType::Model => "Model".to_string(),
            PipeDataType::LinearModel => "LinearModel".to_string(),
            PipeDataType::StandardLinearModel => "StandardLinearModel".to_string(),
            PipeDataType::Tableau => "Tableau".to_string(),
            PipeDataType::OptimalTableau => "OptimalTableau".to_string(),
            PipeDataType::OptimalTableauWithSteps => "OptimalTableauWithSteps".to_string(),
            PipeDataType::BinarySolution => "BinarySolution".to_string(),
        };

        f.write_str(&s)
    }
}

#[derive(Debug)]
pub enum PipeError {
    EmptyPipeData,
    InvalidData {
        expected: PipeDataType,
        got: PipeDataType,
    },
    CompilationError {
        error: CompilationError,
        source: String,
    },
    TransformError {
        error: TransformError,
        source: String,
    },
    LinearizationError(LinearizationError),
    StandardizationError(()),
    CanonicalizationError(CanonicalTransformError),
    SimplexError(SimplexError, Tableau),
    BinarySolverError(BinarySolverError)
}
impl Display for PipeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipeError::EmptyPipeData => write!(f, "Pipe data is empty"),
            PipeError::InvalidData { expected, got } => {
                write!(
                    f,
                    "Expected data of type \"{}\" but got \"{}\"",
                    expected, got
                )
            }
            PipeError::CompilationError { error, source } => {
                write!(f, "{}", error.to_string_from_source(source))
            }
            PipeError::TransformError { error, source } => {
                match error.get_trace_from_source(source) {
                    Ok(trace) => write!(f, "{}", trace),
                    Err(_) => write!(f, "{}", error.get_traced_error()),
                }
            }
            PipeError::LinearizationError(e) => write!(f, "{}", e),
            PipeError::StandardizationError(_) => write!(f, "Standardization error"),
            PipeError::CanonicalizationError(e) => write!(f, "{}", e),
            PipeError::SimplexError(e, _) => write!(f, "{}", e),
            PipeError::BinarySolverError(e) => write!(f, "{}", e)
        }
    }
}

pub trait Pipeable {
    fn pipe(&self, data: &mut PipeableData) -> Result<PipeableData, PipeError>;
}
