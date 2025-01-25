use crate::{
    solve_binary_lp_problem, solve_milp_lp_problem, solve_real_lp_problem_clarabel, Assignment,
    IntOrBoolValue, LinearModel, LpSolution, MILPValue, SolverError, VariableType,
};
use indexmap::IndexMap;

/// Solves a any kind of linear programming problem by picking the right solver for the model.
///
/// Takes a linear model containing real, non-negative real, boolean, and integer variables and returns
/// an optimal solution or an error if the problem cannot be solved.
///
/// # Arguments
/// * `lp` - Any kind of linear programming model to solve
///
/// # Returns
/// * `Ok(LpSolution<MILPValue>)` - The optimal solution if found
/// * `Err(SolverError)` - Various error conditions that prevented finding a solution
///
/// # Example
/// ```rust
/// use rooc::{VariableType, Comparison, OptimizationType, auto_solver, LinearModel};
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
/// let solution = auto_solver(&model).unwrap();
/// ```
pub fn auto_solver(lp: &LinearModel) -> Result<LpSolution<MILPValue>, SolverError> {
    let domain = lp.domain();
    let has_binary = domain
        .values()
        .any(|v| *v.get_type() == VariableType::Boolean);
    let has_integer = domain
        .values()
        .any(|v| matches!(v.get_type(), VariableType::IntegerRange(_, _)));
    let has_real = domain.values().any(|v| {
        matches!(
            v.get_type(),
            VariableType::NonNegativeReal(_, _) | VariableType::Real(_, _)
        )
    });
    match (has_binary, has_integer, has_real) {
        (true, true, true) => solve_milp_lp_problem(lp),
        (true, true, false) => solve_milp_lp_problem(lp), //solve_integer_binary_lp_problem(lp).map(int_bool_to_milp),
        (true, false, true) => solve_milp_lp_problem(lp),
        (true, false, false) => solve_binary_lp_problem(lp).map(bool_to_milp),
        (false, true, true) => solve_milp_lp_problem(lp),
        (false, true, false) => solve_milp_lp_problem(lp), //solve_integer_binary_lp_problem(lp).map(int_bool_to_milp),
        (false, false, true) => solve_real_lp_problem_clarabel(lp).map(real_to_milp),
        (false, false, false) => Ok(LpSolution::new(vec![], 0.0, IndexMap::new())),
    }
}

fn bool_to_milp(val: LpSolution<bool>) -> LpSolution<MILPValue> {
    let values = val
        .assignment()
        .iter()
        .map(|v| Assignment {
            name: v.name.clone(),
            value: MILPValue::Bool(v.value),
        })
        .collect();
    LpSolution::new(values, val.value(), val.constraints().clone())
}

#[allow(unused)]
fn int_bool_to_milp(val: LpSolution<IntOrBoolValue>) -> LpSolution<MILPValue> {
    let values = val
        .assignment()
        .iter()
        .map(|v| {
            let value = match v.value {
                IntOrBoolValue::Int(v) => MILPValue::Int(v),
                IntOrBoolValue::Bool(v) => MILPValue::Bool(v),
            };
            Assignment {
                name: v.name.clone(),
                value,
            }
        })
        .collect();
    LpSolution::new(values, val.value(), val.constraints().clone())
}

fn real_to_milp(val: LpSolution<f64>) -> LpSolution<MILPValue> {
    let values = val
        .assignment()
        .iter()
        .map(|v| Assignment {
            value: MILPValue::Real(v.value),
            name: v.name.clone(),
        })
        .collect();
    LpSolution::new(values, val.value(), val.constraints().clone())
}
