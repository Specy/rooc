use crate::math::{Comparison, OptimizationType, VariableType};
use crate::solvers::{find_invalid_variables, Assignment, LpSolution, SolverError};
use crate::transformers::LinearModel;
use good_lp::clarabel;
use good_lp::solvers::ObjectiveDirection;
use good_lp::{
    Expression, ProblemVariables, ResolutionError, Solution, SolverModel, Variable,
    VariableDefinition,
};
use indexmap::IndexMap;

/// Solves a linear programming problem with real variables using the Clarabel solver.
///
/// Takes a linear model containing real or non-negative real variables and returns an optimal solution
/// or an error if the problem cannot be solved.
///
/// # Arguments
/// * `lp` - The linear programming model to solve, must contain only real or non-negative real variables
///
/// # Returns
/// * `Ok(LpSolution<f64>)` - The optimal solution if found
/// * `Err(SolverError)` - Various error conditions that prevented finding a solution
///
/// # Example
/// ```rust
/// use rooc::{VariableType, Comparison, OptimizationType, solve_real_lp_problem_clarabel, LinearModel};
///
/// let mut model = LinearModel::new();
/// model.add_variable("x1", VariableType::NonNegativeReal);
/// model.add_variable("x2", VariableType::Real);
///
/// // Add constraint: x1 + x2 <= 5
/// model.add_constraint(vec![1.0, 1.0], Comparison::LessOrEqual, 5.0);
///
/// // Set objective: maximize x1 + 2*x2
/// model.set_objective(vec![1.0, 2.0], OptimizationType::Max);
///
/// let solution = solve_real_lp_problem_clarabel(&model).unwrap();
/// ```
pub fn solve_real_lp_problem_clarabel(lp: &LinearModel) -> Result<LpSolution<f64>, SolverError> {
    let domain = lp.domain();
    let invalid_variables = find_invalid_variables(domain, |var| {
        matches!(var, VariableType::Real | VariableType::NonNegativeReal)
    });
    if !invalid_variables.is_empty() {
        return Err(SolverError::InvalidDomain {
            expected: vec![VariableType::Real, VariableType::NonNegativeReal],
            got: invalid_variables,
        });
    }
    let opt_type = match lp.optimization_type() {
        OptimizationType::Min => ObjectiveDirection::Minimisation,
        OptimizationType::Max => ObjectiveDirection::Maximisation,
        OptimizationType::Satisfy => ObjectiveDirection::Minimisation,
    };
    let mut variables = ProblemVariables::new();
    let mut created_vars: IndexMap<String, Variable> = IndexMap::new();
    for (name, var) in domain.iter() {
        let def = VariableDefinition::new().name(name);
        let def = match var.get_type() {
            VariableType::Real => def.min(f32::MIN).max(f32::MAX),
            VariableType::NonNegativeReal => def.min(0.0).max(f32::MAX),
            _ => panic!(),
        };
        let var = variables.add(def);
        created_vars.insert(name.clone(), var);
    }
    let vars = lp.variables();
    let obj_exp = match lp.optimization_type() {
        OptimizationType::Satisfy => 0.into(),
        OptimizationType::Max | OptimizationType::Min => vars.iter().zip(lp.objective()).fold(
            Expression::from(lp.objective_offset()),
            |acc, (name, coeff)| {
                let var = created_vars.get(name).unwrap();
                acc + (*coeff) * (*var)
            },
        ),
    };
    let objective = variables.optimise(opt_type, obj_exp.clone());
    let mut model = objective.using(clarabel);
    for constraint in lp.constraints() {
        let mut good_lp_constraint = Expression::with_capacity(vars.len());
        for (i, c) in constraint.coefficients().iter().enumerate() {
            let name = &vars[i];
            let existing = created_vars.get(name).unwrap().clone();
            good_lp_constraint += (*c) * existing;
        }
        let constraint = match constraint.constraint_type() {
            Comparison::LessOrEqual => good_lp_constraint.leq(constraint.rhs()),
            Comparison::GreaterOrEqual => good_lp_constraint.geq(constraint.rhs()),
            Comparison::Equal => good_lp_constraint.eq(constraint.rhs()),
            c => {
                return Err(SolverError::UnavailableComparison {
                    got: *c,
                    expected: vec![
                        Comparison::LessOrEqual,
                        Comparison::GreaterOrEqual,
                        Comparison::Equal,
                    ],
                })
            }
        };
        model = model.with(constraint);
    }
    let solution = model.solve();
    match solution {
        Ok(sol) => {
            let vars = vars
                .iter()
                .map(|name| {
                    let var = created_vars.get(name).unwrap();
                    Assignment {
                        name: name.clone(),
                        value: sol.value(*var),
                    }
                })
                .collect::<Vec<Assignment<f64>>>();
            let coeffs = lp.objective();
            //good_lp does not provide a way to get the objective value
            let value = vars
                .iter()
                .enumerate()
                .fold(lp.objective_offset(), |acc, (i, a)| {
                    acc + a.value * coeffs[i]
                });
            Ok(LpSolution::new(vars, value + lp.objective_offset()))
        }
        Err(e) => match e {
            ResolutionError::Unbounded => Err(SolverError::Unbounded),
            ResolutionError::Infeasible => Err(SolverError::Infisible),
            ResolutionError::Other(s) => Err(SolverError::Other(s.to_string())),
            ResolutionError::Str(s) => Err(SolverError::Other(s)),
        },
    }
}
