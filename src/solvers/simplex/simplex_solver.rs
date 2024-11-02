use crate::math::{Comparison, OptimizationType, VariableType};
use crate::solvers::{find_invalid_variables, Assignment, LpSolution, SimplexError, SolverError};
use crate::transformers::LinearModel;
use microlp::{OptimizationDirection, Problem};

#[allow(unused)]
pub fn solve_real_lp_problem_slow_simplex(lp: &LinearModel, limit: i64) -> Result<LpSolution<f64>, SolverError> {
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


pub fn solve_real_lp_problem_micro_lp(lp: &LinearModel) -> Result<LpSolution<f64>, SolverError> {
    let domain = lp.get_domain();
    let invalid_variables = find_invalid_variables(domain, |var| {
        matches!(var, VariableType::Real | VariableType::NonNegativeReal)
    });
    if !invalid_variables.is_empty() {
        return Err(SolverError::InvalidDomain {
            expected: vec![VariableType::Real, VariableType::NonNegativeReal],
            got: invalid_variables,
        });
    }
    let opt_type = match lp.get_optimization_type() {
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
    let variables = lp.get_variables();

    
    let obj = lp.get_objective();
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
            VariableType::NonNegativeReal => problem.add_var(obj[i], (0.0, f64::INFINITY)),
            VariableType::Real => problem.add_var(obj[i], (f64::NEG_INFINITY, f64::INFINITY)),
            _ => {
                return Err(SolverError::InvalidDomain {
                    expected: vec![VariableType::Real, VariableType::NonNegativeReal],
                    got: vec![(name.clone(), domain.clone())],
                })
            }
        };
        vars_microlp.push(var);
    }
    
    for cons in lp.get_constraints() {
        let coeffs = cons
            .get_coefficients()
            .iter()
            .zip(vars_microlp.iter())
            .map(|(c, v)| (*v, *c))
            .collect::<Vec<_>>();
        let rhs = cons.get_rhs();
        let comparison = match cons.get_constraint_type() {
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
                    got: *cons.get_constraint_type(),
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
            let obj = optimal_solution.objective() + lp.get_objective_offset();
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
