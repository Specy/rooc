use crate::pipe::pipe_definitions::{PipeError, Pipeable, PipeableData};
use crate::pipe::PipeContext;
#[allow(unused_imports)]
use crate::prelude::*;
use crate::solvers::{
    solve_binary_lp_problem, solve_integer_binary_lp_problem, solve_real_lp_problem_clarabel,
};
use crate::transformers::Linearizer;
use crate::{auto_solver, solve_milp_lp_problem, RoocParser};

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
/// Enum that represents the different pipes that can be executed
pub enum Pipes {
    CompilerPipe,
    PreModelPipe,
    ModelPipe,
    LinearModelPipe,
    StandardLinearModelPipe,
    TableauPipe,
    RealPipe,
    StepByStepSimplexPipe,
    BinarySolverPipe,
    IntegerBinarySolverPipe,
    MILPSolverPipe,
    AutoSolverPipe,
}

//-------------------- Source Compiler --------------------
/// Pipe that compiles the source code into a parser
pub struct CompilerPipe {}
impl Default for CompilerPipe {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilerPipe {
    pub fn new() -> CompilerPipe {
        CompilerPipe {}
    }
}

impl Pipeable for CompilerPipe {
    fn pipe(&self, data: &mut PipeableData, _: &PipeContext) -> Result<PipeableData, PipeError> {
        let str = data.as_string_data()?;
        let parser = RoocParser::new(str.clone());
        Ok(PipeableData::Parser(parser))
    }
}
//-------------------- Pre Model --------------------
/// Pipe that transforms the parsed code into a pre model
pub struct PreModelPipe {}
impl Default for PreModelPipe {
    fn default() -> Self {
        Self::new()
    }
}

impl PreModelPipe {
    pub fn new() -> PreModelPipe {
        PreModelPipe {}
    }
}
impl Pipeable for PreModelPipe {
    fn pipe(&self, data: &mut PipeableData, _: &PipeContext) -> Result<PipeableData, PipeError> {
        let parser = data.as_parser()?;
        match parser.parse() {
            Ok(model) => Ok(PipeableData::PreModel(model)),
            Err(e) => Err(PipeError::CompilationError {
                error: e,
                source: parser.source.clone(),
            }),
        }
    }
}
//-------------------- Model --------------------
/// Pipe that transforms the pre model into a model
pub struct ModelPipe {}
impl Default for ModelPipe {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelPipe {
    pub fn new() -> ModelPipe {
        ModelPipe {}
    }
}
impl Pipeable for ModelPipe {
    fn pipe(
        &self,
        data: &mut PipeableData,
        pipe_context: &PipeContext,
    ) -> Result<PipeableData, PipeError> {
        let pre_model = data.as_pre_model()?;
        if let Err(e) =
            pre_model.create_type_checker(pipe_context.constants(), pipe_context.functions())
        {
            return Err(PipeError::TransformError {
                error: e,
                source: pre_model.source().unwrap_or("".to_string()),
            });
        }
        match pre_model
            .clone()
            .transform(pipe_context.constants().clone(), pipe_context.functions())
        {
            Ok(model) => Ok(PipeableData::Model(model)),
            Err(e) => Err(PipeError::TransformError {
                error: e,
                source: pre_model.source().unwrap_or("".to_string()),
            }),
        }
    }
}
//-------------------- Linear Model --------------------
/// Pipe that transforms the model into a linear model
pub struct LinearModelPipe {}
impl Default for LinearModelPipe {
    fn default() -> Self {
        Self::new()
    }
}

impl LinearModelPipe {
    pub fn new() -> LinearModelPipe {
        LinearModelPipe {}
    }
}
impl Pipeable for LinearModelPipe {
    fn pipe(&self, data: &mut PipeableData, _: &PipeContext) -> Result<PipeableData, PipeError> {
        let model = data.as_model()?;
        let model = model.clone();
        let linearizer = Linearizer::linearize(model);
        match linearizer {
            Ok(linear) => Ok(PipeableData::LinearModel(linear)),
            Err(e) => Err(PipeError::LinearizationError(e)),
        }
    }
}
//-------------------- Standard Linear Model --------------------
/// Pipe that transforms the linear model into a standard linear model
pub struct StandardLinearModelPipe {}
impl Default for StandardLinearModelPipe {
    fn default() -> Self {
        Self::new()
    }
}

impl StandardLinearModelPipe {
    pub fn new() -> StandardLinearModelPipe {
        StandardLinearModelPipe {}
    }
}
impl Pipeable for StandardLinearModelPipe {
    fn pipe(&self, data: &mut PipeableData, _: &PipeContext) -> Result<PipeableData, PipeError> {
        let linear_model = data.as_linear_model()?;
        let standard = linear_model.clone().into_standard_form();
        match standard {
            Ok(standard) => Ok(PipeableData::StandardLinearModel(standard)),
            Err(e) => Err(PipeError::StandardizationError(e)),
        }
    }
}
//-------------------- Tableau --------------------
/// Pipe that transforms the standard linear model into a tableau for the simplex algorithm
pub struct TableauPipe {}
impl Default for TableauPipe {
    fn default() -> Self {
        Self::new()
    }
}

impl TableauPipe {
    pub fn new() -> TableauPipe {
        TableauPipe {}
    }
}
impl Pipeable for TableauPipe {
    fn pipe(&self, data: &mut PipeableData, _: &PipeContext) -> Result<PipeableData, PipeError> {
        let standard_linear_model = data.as_standard_linear_model()?.clone();
        let tableau = standard_linear_model.into_tableau();
        match tableau {
            Ok(tableau) => Ok(PipeableData::Tableau(tableau)),
            Err(e) => Err(PipeError::CanonicalizationError(e)),
        }
    }
}
//-------------------- Simplex --------------------
/// Pipe that solves the linear model using a real solver
pub struct RealSolver {}
impl Default for RealSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl RealSolver {
    pub fn new() -> RealSolver {
        RealSolver {}
    }
}

impl Pipeable for RealSolver {
    fn pipe(&self, data: &mut PipeableData, _: &PipeContext) -> Result<PipeableData, PipeError> {
        let model = data.as_linear_model()?.clone();
        //solve_real_lp_problem
        let assignment = solve_real_lp_problem_clarabel(&model);
        match assignment {
            Ok(optimal) => Ok(PipeableData::RealSolution(optimal)),
            Err(e) => Err(PipeError::SolverError(e)),
        }
    }
}
//-------------------- Step by step Simplex  --------------------
/// Pipe that solves the linear model using a step by step simplex algorithm
pub struct StepByStepSimplexPipe {}
impl Default for StepByStepSimplexPipe {
    fn default() -> Self {
        Self::new()
    }
}

impl StepByStepSimplexPipe {
    pub fn new() -> StepByStepSimplexPipe {
        StepByStepSimplexPipe {}
    }
}
impl Pipeable for StepByStepSimplexPipe {
    fn pipe(&self, data: &mut PipeableData, _: &PipeContext) -> Result<PipeableData, PipeError> {
        let mut tableau = data.as_tableau()?.clone();
        let optimal_tableau = tableau.solve_step_by_step(1000);
        match optimal_tableau {
            Ok(optimal_tableau) => Ok(PipeableData::OptimalTableauWithSteps(optimal_tableau)),
            Err(e) => Err(PipeError::StepByStepSimplexError(e, tableau)),
        }
    }
}

//-------------------- Dual --------------------

struct DualPipe {}
impl Default for DualPipe {
    fn default() -> Self {
        Self::new()
    }
}

impl DualPipe {
    pub fn new() -> DualPipe {
        DualPipe {}
    }
}
impl Pipeable for DualPipe {
    fn pipe(&self, data: &mut PipeableData, _: &PipeContext) -> Result<PipeableData, PipeError> {
        let model = data.as_linear_model()?.clone();
        //TODO: Implement dual
        let dual = model;
        Ok(PipeableData::LinearModel(dual))
    }
}

//-------------------- Binary solver --------------------

/// Pipe that solves the linear model using a binary solver
pub struct BinarySolverPipe {}
impl Default for BinarySolverPipe {
    fn default() -> Self {
        Self::new()
    }
}

impl BinarySolverPipe {
    pub fn new() -> BinarySolverPipe {
        BinarySolverPipe {}
    }
}
impl Pipeable for BinarySolverPipe {
    fn pipe(&self, data: &mut PipeableData, _: &PipeContext) -> Result<PipeableData, PipeError> {
        let linear_model = data.as_linear_model()?;
        let binary_solution = solve_binary_lp_problem(linear_model);
        match binary_solution {
            Ok(solution) => Ok(PipeableData::BinarySolution(solution)),
            Err(e) => Err(PipeError::SolverError(e)),
        }
    }
}
//-------------------- Integer Binary solver --------------------
/// Pipe that solves the linear model using an integer binary solver
pub struct IntegerBinarySolverPipe {}
impl Default for IntegerBinarySolverPipe {
    fn default() -> Self {
        Self::new()
    }
}

impl IntegerBinarySolverPipe {
    pub fn new() -> IntegerBinarySolverPipe {
        IntegerBinarySolverPipe {}
    }
}
impl Pipeable for IntegerBinarySolverPipe {
    fn pipe(&self, data: &mut PipeableData, _: &PipeContext) -> Result<PipeableData, PipeError> {
        let linear_model = data.as_linear_model()?;
        let integer_binary_solution = solve_integer_binary_lp_problem(linear_model);
        match integer_binary_solution {
            Ok(solution) => Ok(PipeableData::IntegerBinarySolution(solution)),
            Err(e) => Err(PipeError::SolverError(e)),
        }
    }
}

//-------------------- MILP solver --------------------
/// Pipe that solves linear models using a MILP solver
pub struct MILPSolverPipe {}
impl Default for MILPSolverPipe {
    fn default() -> Self {
        Self::new()
    }
}

impl MILPSolverPipe {
    pub fn new() -> MILPSolverPipe {
        MILPSolverPipe {}
    }
}
impl Pipeable for MILPSolverPipe {
    fn pipe(&self, data: &mut PipeableData, _: &PipeContext) -> Result<PipeableData, PipeError> {
        let linear_model = data.as_linear_model()?;
        let integer_binary_solution = solve_milp_lp_problem(linear_model);
        match integer_binary_solution {
            Ok(solution) => Ok(PipeableData::MILPSolution(solution)),
            Err(e) => Err(PipeError::SolverError(e)),
        }
    }
}

//-------------------- Auto solver --------------------
/// Pipe that solves linear models by automatically picking the right solver
pub struct AutoSolverPipe {}
impl Default for AutoSolverPipe {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoSolverPipe {
    pub fn new() -> AutoSolverPipe {
        AutoSolverPipe {}
    }
}
impl Pipeable for AutoSolverPipe {
    fn pipe(&self, data: &mut PipeableData, _: &PipeContext) -> Result<PipeableData, PipeError> {
        let linear_model = data.as_linear_model()?;
        let solution = auto_solver(linear_model);
        match solution {
            Ok(solution) => Ok(PipeableData::MILPSolution(solution)),
            Err(e) => Err(PipeError::SolverError(e)),
        }
    }
}
