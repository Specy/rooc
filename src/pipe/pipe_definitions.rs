use crate::parser::model_transformer::Model;
use crate::parser::model_transformer::TransformError;
use crate::parser::pre_model::PreModel;
#[allow(unused_imports)]
use crate::prelude::*;
use crate::primitives::Constant;
use crate::runtime_builtin::RoocFunction;
use crate::solvers::IntOrBoolValue;
use crate::solvers::{
    CanonicalTransformError, OptimalTableau, OptimalTableauWithSteps, SimplexError, Tableau,
};
use crate::solvers::{LpSolution, SolverError};
use crate::transformers::LinearModel;
use crate::transformers::LinearizationError;
use crate::transformers::StandardLinearModel;
use crate::utils::CompilationError;
use crate::{match_pipe_data_to, MILPValue, RoocParser};
use indexmap::IndexMap;
use std::fmt::Display;

/// The data that can be passed between pipes
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
    BinarySolution(LpSolution<bool>),
    IntegerBinarySolution(LpSolution<IntOrBoolValue>),
    RealSolution(LpSolution<f64>),
    MILPSolution(LpSolution<MILPValue>),
}

#[allow(clippy::result_large_err)]
#[allow(clippy::wrong_self_convention)]
impl PipeableData {
    /// Gets the type of the data
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
            PipeableData::IntegerBinarySolution(_) => PipeDataType::IntegerBinarySolution,
            PipeableData::RealSolution(_) => PipeDataType::RealSolution,
            PipeableData::MILPSolution(_) => PipeDataType::MILPSolution,
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
    pub fn to_binary_solution(self) -> Result<LpSolution<bool>, PipeError> {
        match_pipe_data_to!(self, BinarySolution, BinarySolution)
    }
    pub fn to_real_solution(self) -> Result<LpSolution<f64>, PipeError> {
        match_pipe_data_to!(self, RealSolution, RealSolution)
    }
    pub fn to_milp_solution(self) -> Result<LpSolution<MILPValue>, PipeError> {
        match_pipe_data_to!(self, MILPSolution, MILPSolution)
    }

    pub fn to_integer_binary_solution(self) -> Result<LpSolution<IntOrBoolValue>, PipeError> {
        match_pipe_data_to!(self, IntegerBinarySolution, IntegerBinarySolution)
    }
    pub fn as_string_data(&self) -> Result<&String, PipeError> {
        match_pipe_data_to!(self, String, String)
    }
    pub fn as_binary_solution(&self) -> Result<&LpSolution<bool>, PipeError> {
        match_pipe_data_to!(self, BinarySolution, BinarySolution)
    }
    pub fn as_integer_binary_solution(&self) -> Result<&LpSolution<IntOrBoolValue>, PipeError> {
        match_pipe_data_to!(self, IntegerBinarySolution, IntegerBinarySolution)
    }
    pub fn as_real_solution(&self) -> Result<&LpSolution<f64>, PipeError> {
        match_pipe_data_to!(self, RealSolution, RealSolution)
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
            PipeableData::OptimalTableauWithSteps(t) => write!(f, "{}", t),
            PipeableData::BinarySolution(s) => write!(f, "{}", s),
            PipeableData::IntegerBinarySolution(s) => write!(f, "{}", s),
            PipeableData::RealSolution(s) => write!(f, "{}", s),
            PipeableData::MILPSolution(s) => write!(f, "{}", s),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug)]
/// The type of data that can be passed between pipes
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
    BinarySolution,
    IntegerBinarySolution,
    RealSolution,
    MILPSolution,
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
            PipeDataType::IntegerBinarySolution => "IntegerBinarySolution".to_string(),
            PipeDataType::RealSolution => "RealSolution".to_string(),
            PipeDataType::MILPSolution => "MILPSolution".to_string(),
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
    StandardizationError(SolverError),
    CanonicalizationError(CanonicalTransformError),
    StepByStepSimplexError(SimplexError, Tableau),
    SolverError(SolverError),
    Other(String),
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
            PipeError::Other(s) => write!(f, "{}", s),
            PipeError::CompilationError { error, source } => {
                write!(f, "{}", error.to_string_from_source(source))
            }
            PipeError::TransformError { error, source } => match error.trace_from_source(source) {
                Ok(trace) => write!(f, "{}", trace),
                Err(_) => write!(f, "{}", error.traced_error()),
            },
            PipeError::LinearizationError(e) => write!(f, "{}", e),
            PipeError::StandardizationError(e) => write!(f, "{}", e),
            PipeError::CanonicalizationError(e) => write!(f, "{}", e),
            PipeError::StepByStepSimplexError(e, _) => write!(f, "{}", e),
            PipeError::SolverError(e) => write!(f, "{}", e),
        }
    }
}

/// The context in which a pipe is executed
///
/// It contains the functions and constants that can be used in the pipe
pub struct PipeContext<'a> {
    functions: &'a IndexMap<String, Box<dyn RoocFunction>>,
    constants: Vec<Constant>,
}
impl PipeContext<'_> {
    pub fn new(
        constants: Vec<Constant>,
        fns: &IndexMap<String, Box<dyn RoocFunction>>,
    ) -> PipeContext {
        PipeContext {
            constants,
            functions: fns,
        }
    }
    pub fn constants(&self) -> &Vec<Constant> {
        &self.constants
    }
    pub fn functions(&self) -> &IndexMap<String, Box<dyn RoocFunction>> {
        self.functions
    }
}

pub trait Pipeable {
    /// The function that is called when the pipe is executed
    #[allow(clippy::result_large_err)]
    fn pipe(
        &self,
        data: &mut PipeableData,
        pipe_context: &PipeContext,
    ) -> Result<PipeableData, PipeError>;
}
