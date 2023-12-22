use crate::{
    math::math_enums::{Comparison, OptimizationType},
    solvers::linear_problem::{
        Constraint, EqualityConstraint, LinearProblem, StandardLinearProblem,
    },
};

pub fn to_standard_form(problem: &LinearProblem) -> Result<StandardLinearProblem, ()> {
    let objective = problem.get_objective();
    let mut constraints: Vec<EqualityConstraint> = Vec::new();
    let mut variables = problem.get_variables().clone();
    let mut slack_surplus_index = 0;
    let mut total_variables = variables.len();
    for constraint in problem.get_constraints() {
        let (equality_constraint, added_variable) =
            normalize_constraint(constraint, slack_surplus_index, total_variables);
        match added_variable {
            Some(variable) => {
                variables.push(variable);
                slack_surplus_index += 1;
                total_variables += 1;
            }
            None => {}
        }
        constraints.push(equality_constraint);
    }
    constraints
        .iter_mut()
        .for_each(|c| c.ensure_size(total_variables));
    let (objective_offset, objective) = match problem.get_optimization_type() {
        OptimizationType::Max => (
            problem.get_objective_offset(),
            //TODO the sign of the resulting min should be reverted, add a sign to the objective
            objective.iter().map(|c| c * -1.0).collect(),
        ),
        OptimizationType::Min => (problem.get_objective_offset(), objective.clone()),
    };
    Ok(StandardLinearProblem::new(
        objective,
        constraints,
        variables,
        objective_offset,
    ))
}

pub fn normalize_constraint(
    constraint: &Constraint,
    last_slack_surplus_index: usize,
    total_variables: usize,
) -> (EqualityConstraint, Option<String>) {
    match constraint.get_constraint_type() {
        Comparison::Equal => {
            let equality_constraint = EqualityConstraint::new(
                constraint.get_coefficients().clone(),
                constraint.get_rhs(),
            );
            (equality_constraint, None)
        }
        Comparison::LowerOrEqual => {
            let mut coefficients = constraint.get_coefficients().clone();
            coefficients.resize(total_variables, 0.0);
            coefficients.push(1.0);
            let equality_constraint = EqualityConstraint::new(coefficients, constraint.get_rhs());
            (
                equality_constraint,
                Some(format!("_s{}", last_slack_surplus_index + 1)),
            )
        }
        Comparison::UpperOrEqual => {
            let mut coefficients = constraint.get_coefficients().clone();
            coefficients.resize(total_variables, 0.0);
            coefficients.push(-1.0);
            let equality_constraint = EqualityConstraint::new(coefficients, constraint.get_rhs());
            (
                equality_constraint,
                Some(format!("_s{}", last_slack_surplus_index + 1)),
            )
        }
    }
}
