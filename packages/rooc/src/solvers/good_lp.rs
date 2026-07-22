//! Shared scaffolding for solver backends provided by `good_lp`.

#[cfg(any(
    feature = "coin_cbc",
    feature = "highs",
    feature = "lpsolve",
    feature = "scip",
    feature = "scip_bundled",
    feature = "cplex-rs"
))]
use std::time::Duration;

use crate::make_constraints_map_from_assignment;
use crate::math::{Comparison, OptimizationType, VariableType};
use crate::solvers::{Assignment, LpSolution, SolutionStatus, SolverError};
use crate::transformers::LinearModel;
#[cfg(any(feature = "clarabel", feature = "highs"))]
use ::good_lp::DualValues as GoodLpDualValues;
#[cfg(any(
    feature = "coin_cbc",
    feature = "highs",
    feature = "lpsolve",
    feature = "scip",
    feature = "scip_bundled",
    feature = "cplex-rs"
))]
use ::good_lp::WithTimeLimit as GoodLpWithTimeLimit;
use ::good_lp::constraint::ConstraintReference;
use ::good_lp::{
    Expression, ProblemVariables, ResolutionError, Solution as GoodLpSolution,
    SolutionStatus as GoodLpSolutionStatus, Solver as GoodLpSolver,
    SolverModel as GoodLpSolverModel, Variable, VariableDefinition,
};
#[cfg(any(
    feature = "coin_cbc",
    feature = "highs",
    feature = "scip",
    feature = "scip_bundled"
))]
use ::good_lp::{WithInitialSolution as GoodLpWithInitialSolution, WithMipGap as GoodLpWithMipGap};
use indexmap::IndexMap;

#[cfg(any(
    feature = "coin_cbc",
    feature = "highs",
    feature = "lpsolve",
    feature = "scip",
    feature = "scip_bundled",
    feature = "cplex-rs"
))]
#[derive(Debug, Clone, Default)]
pub(crate) struct GoodLpOptions {
    pub(crate) time_limit: Option<Duration>,
    pub(crate) mip_gap: Option<f64>,
    pub(crate) initial_solution: Vec<(String, f64)>,
}

#[cfg(any(
    feature = "coin_cbc",
    feature = "highs",
    feature = "lpsolve",
    feature = "scip",
    feature = "scip_bundled",
    feature = "cplex-rs"
))]
impl GoodLpOptions {
    pub(crate) fn with_time_limit(mut self, time_limit: Duration) -> Self {
        self.time_limit = Some(time_limit);
        self
    }

    pub(crate) fn with_mip_gap(mut self, mip_gap: f64) -> Self {
        self.mip_gap = Some(mip_gap);
        self
    }

    pub(crate) fn with_initial_solution<I, N>(mut self, solution: I) -> Self
    where
        I: IntoIterator<Item = (N, f64)>,
        N: Into<String>,
    {
        self.initial_solution = solution
            .into_iter()
            .map(|(name, value)| (name.into(), value))
            .collect();
        self
    }
}

/// Solves a ROOC linear model through any compatible `good_lp` backend.
pub(crate) fn solve_with_good_lp<S, M, R, C, V, D>(
    lp: &LinearModel,
    solver: S,
    configure_model: C,
    validate_solution: V,
    extract_duals: D,
) -> Result<LpSolution<f64>, SolverError>
where
    S: GoodLpSolver<Model = M>,
    M: GoodLpSolverModel<Solution = R, Error = ResolutionError>,
    R: GoodLpSolution,
    C: FnOnce(M, &[(String, Variable)]) -> Result<M, SolverError>,
    V: FnOnce(&R) -> Result<(), SolverError>,
    D: FnOnce(&mut R, &[(String, ConstraintReference)]) -> IndexMap<String, f64>,
{
    let variables = lp.variables();
    if lp.objective().len() != variables.len() {
        return Err(SolverError::Other(format!(
            "objective length {} does not match variable count {}",
            lp.objective().len(),
            variables.len()
        )));
    }

    let mut problem_variables = ProblemVariables::new();
    let mut created_variables = Vec::with_capacity(variables.len());
    for name in variables {
        let domain = lp.domain().get(name).ok_or_else(|| {
            SolverError::Other(format!("variable {name} is missing from the model domain"))
        })?;
        let definition = variable_definition(name, domain.get_type());
        let variable = problem_variables.add(definition);
        created_variables.push((name.clone(), variable));
    }

    let objective_direction = match lp.optimization_type() {
        OptimizationType::Min | OptimizationType::Satisfy => {
            ::good_lp::solvers::ObjectiveDirection::Minimisation
        }
        OptimizationType::Max => ::good_lp::solvers::ObjectiveDirection::Maximisation,
    };
    let objective = match lp.optimization_type() {
        OptimizationType::Satisfy => Expression::from(0.0),
        OptimizationType::Min | OptimizationType::Max => {
            created_variables.iter().zip(lp.objective()).fold(
                Expression::from(lp.objective_offset()),
                |expression, ((_, variable), coefficient)| expression + *coefficient * *variable,
            )
        }
    };

    let mut model = problem_variables
        .optimise(objective_direction, objective)
        .using(solver);
    let mut constraint_references = Vec::with_capacity(lp.constraints().len());
    for constraint in lp.constraints() {
        if constraint.coefficients().len() != variables.len() {
            return Err(SolverError::Other(format!(
                "constraint has {} coefficients but variable count is {}",
                constraint.coefficients().len(),
                variables.len()
            )));
        }
        let expression = constraint.coefficients().iter().enumerate().fold(
            Expression::with_capacity(variables.len()),
            |expression, (index, coefficient)| {
                expression + *coefficient * created_variables[index].1
            },
        );
        let constraint_name = constraint.name();
        let constraint = match constraint.constraint_type() {
            Comparison::LessOrEqual => expression.leq(constraint.rhs()),
            Comparison::GreaterOrEqual => expression.geq(constraint.rhs()),
            Comparison::Equal => expression.eq(constraint.rhs()),
            comparison => {
                return Err(SolverError::UnavailableComparison {
                    got: *comparison,
                    expected: vec![
                        Comparison::LessOrEqual,
                        Comparison::GreaterOrEqual,
                        Comparison::Equal,
                    ],
                });
            }
        };
        let reference = model.add_constraint(constraint);
        constraint_references.push((constraint_name, reference));
    }

    let model = configure_model(model, &created_variables)?;
    let mut solution = model.solve().map_err(map_resolution_error)?;
    validate_solution(&solution)?;
    let shadow_prices = extract_duals(&mut solution, &constraint_references);

    let assignment = created_variables
        .iter()
        .map(|(name, variable)| Assignment {
            name: name.clone(),
            value: solution.value(*variable),
        })
        .collect::<Vec<_>>();
    let values = assignment
        .iter()
        .map(|assignment| assignment.value)
        .collect();
    let status = match solution.status() {
        GoodLpSolutionStatus::Optimal => SolutionStatus::Optimal,
        GoodLpSolutionStatus::TimeLimit | GoodLpSolutionStatus::GapLimit => {
            SolutionStatus::Feasible
        }
    };
    let value = lp.calc_objective(&values);
    let constraints = make_constraints_map_from_assignment(lp, &values);

    Ok(LpSolution::new(assignment, value, constraints)
        .with_status(status)
        .with_shadow_prices(shadow_prices))
}

#[cfg(any(
    feature = "coin_cbc",
    feature = "highs",
    feature = "lpsolve",
    feature = "scip",
    feature = "scip_bundled",
    feature = "cplex-rs"
))]
pub(crate) fn apply_time_limit<M: GoodLpWithTimeLimit>(mut model: M, options: &GoodLpOptions) -> M {
    if let Some(time_limit) = options.time_limit {
        model = model.with_time_limit(time_limit.as_secs_f64());
    }
    model
}

#[cfg(any(
    feature = "coin_cbc",
    feature = "highs",
    feature = "scip",
    feature = "scip_bundled"
))]
pub(crate) fn apply_mip_options<M>(
    mut model: M,
    options: &GoodLpOptions,
    variables: &[(String, Variable)],
) -> Result<M, SolverError>
where
    M: GoodLpWithInitialSolution + GoodLpWithTimeLimit + GoodLpWithMipGap,
{
    if !options.initial_solution.is_empty() {
        let initial_solution = options
            .initial_solution
            .iter()
            .map(|(name, value)| {
                if !value.is_finite() {
                    return Err(SolverError::Other(format!(
                        "initial solution value for variable {name} must be finite"
                    )));
                }
                let variable = variables
                    .iter()
                    .find(|(variable_name, _)| variable_name == name)
                    .map(|(_, variable)| *variable)
                    .ok_or_else(|| {
                        SolverError::Other(format!(
                            "initial solution refers to unknown variable {name}"
                        ))
                    })?;
                Ok((variable, *value))
            })
            .collect::<Result<Vec<_>, _>>()?;
        model = model.with_initial_solution(initial_solution);
    }

    model = apply_time_limit(model, options);

    if let Some(mip_gap) = options.mip_gap {
        let mip_gap = mip_gap_as_f32(mip_gap)?;
        model = model
            .with_mip_gap(mip_gap)
            .map_err(|error| SolverError::Other(error.to_string()))?;
    }

    Ok(model)
}

#[cfg(any(feature = "clarabel", feature = "highs"))]
pub(crate) fn collect_good_lp_duals<D>(
    dual: D,
    constraint_references: &[(String, ConstraintReference)],
) -> IndexMap<String, f64>
where
    D: GoodLpDualValues,
{
    constraint_references
        .iter()
        .filter_map(|(name, reference)| {
            if name.is_empty() {
                None
            } else {
                Some((name.clone(), dual.dual(reference.clone())))
            }
        })
        .collect()
}

#[cfg(any(
    feature = "coin_cbc",
    feature = "highs",
    feature = "scip",
    feature = "scip_bundled"
))]
fn mip_gap_as_f32(mip_gap: f64) -> Result<f32, SolverError> {
    if !mip_gap.is_finite() || mip_gap < 0.0 {
        return Err(SolverError::Other(format!(
            "MIP gap must be finite and non-negative, got {mip_gap}"
        )));
    }
    let mip_gap = mip_gap as f32;
    if !mip_gap.is_finite() {
        return Err(SolverError::Other(
            "MIP gap is too large for good_lp's f32 interface".to_string(),
        ));
    }
    Ok(mip_gap)
}

fn variable_definition(name: &str, variable_type: &VariableType) -> VariableDefinition {
    match variable_type {
        VariableType::Boolean => VariableDefinition::new().name(name).binary(),
        VariableType::IntegerRange(min, max) => VariableDefinition::new()
            .name(name)
            .integer()
            .min(*min as f64)
            .max(*max as f64),
        VariableType::Real(min, max) | VariableType::NonNegativeReal(min, max) => {
            VariableDefinition::new().name(name).min(*min).max(*max)
        }
    }
}

fn map_resolution_error(error: ResolutionError) -> SolverError {
    match error {
        ResolutionError::Unbounded => SolverError::Unbounded,
        ResolutionError::Infeasible => SolverError::Infeasible,
        ResolutionError::Other(message) => SolverError::Other(message.to_string()),
        ResolutionError::Str(message) => SolverError::Other(message),
    }
}
