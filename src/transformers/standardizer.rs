use crate::{
    math::math_enums::{Comparison, OptimizationType},
};
use crate::transformers::linear_model::{LinearConstraint, LinearModel};
use crate::transformers::standard_linear_model::{EqualityConstraint, StandardLinearModel};

pub fn to_standard_form(problem: &LinearModel) -> Result<StandardLinearModel, ()> {
    let objective = problem.get_objective();
    let mut constraints: Vec<EqualityConstraint> = Vec::new();
    let mut variables = problem.get_variables().clone();
    let mut context = NormalizationContext {
        surplus_index: 0,
        slack_index: 0,
        total_variables: variables.len()
    };
    for constraint in problem.get_constraints() {
        let (equality_constraint, added_variable) =
            normalize_constraint(constraint, &mut context);
        if let Some(variable) = added_variable {
            variables.push(variable);
        };
        constraints.push(equality_constraint);
    }
    constraints
        .iter_mut()
        .for_each(|c| c.ensure_size(context.total_variables));
    let (objective_offset, objective, flip_objective) = match problem.get_optimization_type() {
        OptimizationType::Max => (
            problem.get_objective_offset(),
            //TODO the sign of the resulting min should be reverted, add a sign to the objective
            objective.iter().map(|c| c * -1.0).collect(),
            true,
        ),
        OptimizationType::Min => (problem.get_objective_offset(), objective.clone(), false),
    };
    Ok(StandardLinearModel::new(
        objective,
        constraints,
        variables,
        objective_offset,
        flip_objective
    ))
}


pub struct NormalizationContext {
    pub surplus_index: usize,
    pub slack_index: usize,
    pub total_variables: usize,
}

pub fn normalize_constraint(
    constraint: &LinearConstraint,
    context: &mut NormalizationContext,
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
            coefficients.resize(context.total_variables, 0.0);
            coefficients.push(1.0);
            context.surplus_index += 1;
            let equality_constraint = EqualityConstraint::new(coefficients, constraint.get_rhs());
            (
                equality_constraint,
                Some(format!("$su_{}", context.surplus_index)),
            )
        }
        Comparison::UpperOrEqual => {
            let mut coefficients = constraint.get_coefficients().clone();
            coefficients.resize(context.total_variables, 0.0);
            coefficients.push(-1.0);
            context.slack_index += 1;
            let equality_constraint = EqualityConstraint::new(coefficients, constraint.get_rhs());
            (
                equality_constraint,
                Some(format!("$sl_{}", context.slack_index)),
            )
        }
    }
}
