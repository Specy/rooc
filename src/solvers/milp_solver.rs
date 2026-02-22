use crate::solvers::common::{LpSolution, SolverError};
use crate::transformers::LinearModel;
use crate::{
    Assignment, Comparison, OptimizationType, VariableType, make_constraints_map_from_assignment,
};
use microlp::{ComparisonOp, Error, OptimizationDirection, Problem};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Represents a variable value that can be either boolean or integer.
#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(tag = "type", content = "value")]
pub enum MILPValue {
    /// A boolean value (true/false)
    Bool(bool),
    /// An integer value
    Int(i32),
    /// A real value
    Real(f64),
}
impl Display for MILPValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MILPValue::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            MILPValue::Int(i) => write!(f, "{}", i),
            MILPValue::Real(r) => write!(f, "{}", r),
        }
    }
}
/// Solves a mixed-integer linear programming problem using the MicroLP solver.
///
/// Takes a linear model containing real, non-negative real, boolean, and integer variables and returns
/// an optimal solution or an error if the problem cannot be solved.
///
/// # Arguments
/// * `lp` - The mixed-integer linear programming model to solve
///
/// # Returns
/// * `Ok(LpSolution<MILPValue>)` - The optimal solution if found
/// * `Err(SolverError)` - Various error conditions that prevented finding a solution
///
/// # Example
/// ```rust
/// use rooc::{VariableType, Comparison, OptimizationType, solve_milp_lp_problem, LinearModel};
///
/// let mut model = LinearModel::new();
/// model.add_variable("x", VariableType::non_negative_real());
/// model.add_variable("y", VariableType::non_negative_real());
/// model.add_variable("z", VariableType::IntegerRange(0, 10));
///
/// // Machine time constraint: 3x + 2y + z <= 20
/// model.add_constraint(vec![3.0, 2.0, 1.0], Comparison::LessOrEqual, 20.0);
///
/// // Labor time constraint: 2x + y + 3z <= 15
/// model.add_constraint(vec![2.0, 1.0, 3.0], Comparison::LessOrEqual, 15.0);
///
/// // Minimum production constraint for x: x >= 2
/// model.add_constraint(vec![1.0, 0.0, 0.0], Comparison::GreaterOrEqual, 2.0);
///
/// // Maximum production constraint for y: y <= 7
/// model.add_constraint(vec![0.0, 1.0, 0.0], Comparison::LessOrEqual, 7.0);
///
/// // Set objective: maximize 50x + 40y + 45z
/// model.set_objective(vec![50.0, 40.0, 45.0], OptimizationType::Max);
///
/// let solution = solve_milp_lp_problem(&model).unwrap();
/// ```
pub fn solve_milp_lp_problem(lp: &LinearModel) -> Result<LpSolution<MILPValue>, SolverError> {
    let variables = lp.variables();
    let domain = lp.domain();
    let objective = lp.objective();
    let mut microlp_vars = Vec::with_capacity(variables.len());
    let opt_type = match lp.optimization_type() {
        OptimizationType::Max => OptimizationDirection::Maximize,
        OptimizationType::Min => OptimizationDirection::Minimize,
        OptimizationType::Satisfy => OptimizationDirection::Minimize,
    };
    let mut problem = Problem::new(opt_type);
    for (i, var) in variables.iter().enumerate() {
        let var_domain = domain.get(var).unwrap();
        let coeff = objective[i];
        let added_var = match var_domain.get_type() {
            VariableType::Real(min, max) => problem.add_var(coeff, (*min, *max)),
            VariableType::Boolean => problem.add_binary_var(coeff),
            VariableType::IntegerRange(min, max) => problem.add_integer_var(coeff, (*min, *max)),
            VariableType::NonNegativeReal(min, max) => problem.add_var(coeff, (*min, *max)),
        };
        microlp_vars.push(added_var);
    }

    for constraint in lp.constraints() {
        let coeffs = constraint.coefficients();
        let rhs = constraint.rhs();
        let comparison_type = constraint.constraint_type();
        let microlp_comparison_type = match comparison_type {
            Comparison::LessOrEqual => ComparisonOp::Le,
            Comparison::GreaterOrEqual => ComparisonOp::Ge,
            Comparison::Equal => ComparisonOp::Eq,
            c => {
                return Err(SolverError::UnavailableComparison {
                    got: *c,
                    expected: vec![
                        Comparison::LessOrEqual,
                        Comparison::GreaterOrEqual,
                        Comparison::Equal,
                    ],
                });
            }
        };
        let microlp_coeffs = microlp_vars
            .iter()
            .zip(coeffs.iter())
            .map(|(v, c)| (*v, *c))
            .collect::<Vec<_>>();
        problem.add_constraint(microlp_coeffs, microlp_comparison_type, rhs);
    }
    match problem.solve() {
        Ok(s) => {
            let assignment = microlp_vars
                .iter()
                .zip(variables)
                .map(|(v, name)| {
                    let value = s.var_value(*v);
                    let var_domain = domain.get(name).unwrap();
                    let value = match var_domain.get_type() {
                        VariableType::Real(_, _) | VariableType::NonNegativeReal(_, _) => {
                            MILPValue::Real(value)
                        }
                        VariableType::IntegerRange(_, _) => MILPValue::Int(value as i32),
                        VariableType::Boolean => MILPValue::Bool(value != 0.0),
                    };
                    Assignment {
                        name: name.clone(),
                        value,
                    }
                })
                .collect();
            let coeffs = microlp_vars
                .iter()
                .map(|v| s.var_value(*v))
                .collect();
            let constraints = make_constraints_map_from_assignment(lp, &coeffs);
            Ok(LpSolution::new(
                assignment,
                s.objective() + lp.objective_offset(),
                constraints,
            ))
        }
        Err(e) => Err(match e {
            Error::InternalError(s) => SolverError::Other(s),
            Error::Unbounded => SolverError::Unbounded,
            Error::Infeasible => SolverError::Infisible,
        }),
    }
}
