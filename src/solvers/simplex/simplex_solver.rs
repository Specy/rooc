use crate::math::{Comparison, OptimizationType, VariableType};
use crate::solvers::{find_invalid_variables, Assignment, LpSolution, SimplexError, SolverError};
use crate::transformers::LinearModel;
use microlp::{OptimizationDirection, Problem};

/// Solves a linear programming problem with real variables using a basic simplex algorithm.
///
/// This is a slower implementation that uses a custom tableau-based simplex method.
/// For better performance, prefer using `solve_real_lp_problem_micro_lp`.
///
/// # Arguments
/// * `lp` - The linear programming model to solve
/// * `limit` - Maximum number of iterations before giving up
///
/// # Returns
/// * `Ok(LpSolution<f64>)` - The optimal solution if found
/// * `Err(SolverError)` - Various error conditions that prevented finding a solution
///
/// # Example
/// ```rust
/// use rooc::{VariableType, Comparison, OptimizationType, solve_real_lp_problem_slow_simplex, LinearModel};
///
/// let mut model = LinearModel::new();
/// model.add_variable("x1", VariableType::non_negative_real());
/// model.add_variable("x2", VariableType::non_negative_real());
///
/// // Add constraint: x1 + x2 <= 5
/// model.add_constraint(vec![1.0, 1.0], Comparison::LessOrEqual, 5.0);
///
/// // Set objective: maximize x1 + 2*x2
/// model.set_objective(vec![1.0, 2.0], OptimizationType::Min);
///
/// let solution = solve_real_lp_problem_slow_simplex(&model, 1000).unwrap();
/// ```
#[allow(unused)]
pub fn solve_real_lp_problem_slow_simplex(
    lp: &LinearModel,
    limit: i64,
) -> Result<LpSolution<f64>, SolverError> {
    let standard = lp.clone().into_standard_form()?;
    let mut canonical_form = standard
        .into_tableau()
        .map_err(|e| SolverError::Other(e.to_string()))?;

    let solution = canonical_form.solve(limit);
    match solution {
        Ok(optimal_tableau) => Ok(optimal_tableau.as_lp_solution()),
        Err(e) => match e {
            SimplexError::IterationLimitReached => Err(SolverError::LimitReached),
            SimplexError::Unbounded => Err(SolverError::Unbounded),
            SimplexError::Other => Err(SolverError::Other("An error occoured".to_string())),
        },
    }
}

/// Solves a linear programming problem with real variables using the microlp solver.
///
/// This is the recommended solver for linear programming problems with real variables
/// as it provides better performance than the basic simplex implementation.
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
/// use rooc::{VariableType, Comparison, OptimizationType, solve_real_lp_problem_micro_lp, LinearModel};
///
/// let mut model = LinearModel::new();
/// model.add_variable("x1", VariableType::non_negative_real());
/// model.add_variable("x2", VariableType::real());
///
/// // Add constraint: x1 + x2 <= 5
/// model.add_constraint(vec![1.0, 1.0], Comparison::LessOrEqual, 5.0);
///
/// // Set objective: maximize x1 + 2*x2
/// model.set_objective(vec![1.0, 2.0], OptimizationType::Max);
///
/// let solution = solve_real_lp_problem_micro_lp(&model).unwrap();
/// ```
pub fn solve_real_lp_problem_micro_lp(lp: &LinearModel) -> Result<LpSolution<f64>, SolverError> {
    let domain = lp.domain();
    let invalid_variables = find_invalid_variables(domain, |var| {
        matches!(
            var,
            VariableType::Real(_, _) | VariableType::NonNegativeReal(_, _)
        )
    });
    if !invalid_variables.is_empty() {
        return Err(SolverError::InvalidDomain {
            expected: vec![
                VariableType::Real(f64::NEG_INFINITY, f64::INFINITY),
                VariableType::NonNegativeReal(0.0, f64::INFINITY),
            ],
            got: invalid_variables,
        });
    }
    let opt_type = match lp.optimization_type() {
        OptimizationType::Min => OptimizationDirection::Minimize,
        OptimizationType::Max => OptimizationDirection::Maximize,
        OptimizationType::Satisfy => {
            return Err(SolverError::UnimplementedOptimizationType {
                expected: vec![OptimizationType::Min, OptimizationType::Max],
                got: OptimizationType::Satisfy,
            })
        }
    };
    let mut problem = Problem::new(opt_type);
    let variables = lp.variables();

    let obj = lp.objective();
    let mut vars_microlp = Vec::with_capacity(obj.len());
    for (i, name) in variables.iter().enumerate() {
        let domain = if let Some(domain) = domain.get(name) {
            domain
        } else {
            return Err(SolverError::Other(format!(
                "Variable {} not found in domain",
                name
            )));
        };
        let var = match domain.get_type() {
            VariableType::NonNegativeReal(min, max) => problem.add_var(obj[i], (*min, *max)),
            VariableType::Real(min, max) => problem.add_var(obj[i], (*min, *max)),
            _ => {
                return Err(SolverError::InvalidDomain {
                    expected: vec![
                        VariableType::Real(f64::NEG_INFINITY, f64::INFINITY),
                        VariableType::NonNegativeReal(0.0, f64::INFINITY),
                    ],
                    got: vec![(name.clone(), domain.clone())],
                })
            }
        };
        vars_microlp.push(var);
    }

    for cons in lp.constraints() {
        let coeffs = cons
            .coefficients()
            .iter()
            .zip(vars_microlp.iter())
            .map(|(c, v)| (*v, *c))
            .collect::<Vec<_>>();
        let rhs = cons.rhs();
        let comparison = match cons.constraint_type() {
            Comparison::LessOrEqual => microlp::ComparisonOp::Le,
            Comparison::Equal => microlp::ComparisonOp::Eq,
            Comparison::GreaterOrEqual => microlp::ComparisonOp::Ge,
            Comparison::Less | Comparison::Greater => {
                return Err(SolverError::UnavailableComparison {
                    expected: vec![
                        Comparison::LessOrEqual,
                        Comparison::Equal,
                        Comparison::GreaterOrEqual,
                    ],
                    got: *cons.constraint_type(),
                })
            }
        };
        problem.add_constraint(&coeffs, comparison, rhs);
    }
    let solution = problem.solve();
    match solution {
        Ok(optimal_solution) => {
            match optimal_solution.objective() {
                f if f.is_infinite() => return Err(SolverError::Unbounded),
                f if f.is_nan() => return Err(SolverError::Infisible),
                _ => {}
            }
            let obj = optimal_solution.objective() + lp.objective_offset();
            let coeffs = variables
                .iter()
                .zip(vars_microlp.iter())
                .map(|(name, c)| Assignment {
                    name: name.clone(),
                    value: optimal_solution[*c],
                })
                .collect();
            Ok(LpSolution::new(coeffs, obj))
        }
        Err(e) => match e {
            microlp::Error::Unbounded => Err(SolverError::Unbounded),
            microlp::Error::Infeasible => Err(SolverError::Infisible),
            microlp::Error::InternalError(s) => Err(SolverError::Other(s)),
        },
    }
}
