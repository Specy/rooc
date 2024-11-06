use rooc::{solve_real_lp_problem_clarabel, Comparison, LinearModel, OptimizationType, VariableType};

fn main() {
    let mut model = LinearModel::new();
    model.add_variable("x1", VariableType::NonNegativeReal);
    model.add_variable("x2", VariableType::Real);

    // Add constraint: x1 + x2 <= 5
    model.add_constraint(vec![1.0, 1.0], Comparison::LessOrEqual, 5.0);

    // Set objective: maximize x1 + 2*x2
    model.set_objective(vec![1.0, 2.0], OptimizationType::Max);

    let solution = solve_real_lp_problem_clarabel(&model).unwrap();
    println!("{}", solution);
}
