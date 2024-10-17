use crate::math::math_enums::{Comparison, OptimizationType, VariableType};
use crate::parser::model_transformer::transformer_context::DomainVariable;
use crate::transformers::linear_model::{LinearConstraint, LinearModel};
use crate::transformers::standard_linear_model::{EqualityConstraint, StandardLinearModel};
use crate::utils::{remove_many, InputSpan};

pub fn to_standard_form(problem: LinearModel) -> Result<StandardLinearModel, ()> {
    let (objective, optimization_type, objective_offset, constraints, mut variables, mut domain) =
        problem.into_parts();
    let mut context = NormalizationContext {
        surplus_index: 0,
        slack_index: 0,
        total_variables: variables.len(),
    };
    //we first normalize the constraints
    let mut constraints: Vec<EqualityConstraint> = constraints
        .into_iter()
        .map(|c| {
            let (equality_constraint, added_variable) = normalize_constraint(c, &mut context);
            if let Some(variable) = added_variable {
                variables.push(variable.clone());
                domain.insert(
                    variable,
                    DomainVariable::new(VariableType::PositiveReal, InputSpan::default()),
                );
                context.total_variables += 1;
            };
            equality_constraint
        })
        .collect();
    //we now need to replace all free variables with positive variables
    let free_variables = variables
        .iter()
        .enumerate()
        .filter_map(|(i, v)| {
            let domain_variable = domain.get(v).unwrap();
            if *domain_variable.get_type() == VariableType::Real {
                Some(i)
            } else {
                None
            }
        })
        .collect::<Vec<usize>>();
    //for each free variable we need to replace it with x = x' - x'' where x' and x'' are positive
    //we first add the new variables to the domain
    for i in &free_variables {
        let var_name = variables[*i].clone();
        let (var_name1, var_name2) = (format!("$p{}", var_name), format!("$m{}", var_name));
        variables.push(var_name1.clone());
        variables.push(var_name2.clone());
        domain.insert(
            var_name1.clone(),
            DomainVariable::new(VariableType::PositiveReal, InputSpan::default()),
        );
        domain.insert(
            var_name2.clone(),
            DomainVariable::new(VariableType::PositiveReal, InputSpan::default()),
        );
        context.total_variables += 2;
        constraints.iter_mut().for_each(|c| {
            let original_coefficient = c.get_coefficients()[*i];
            c.get_coefficients_mut().insert(*i, original_coefficient);
            c.get_coefficients_mut().push(original_coefficient * -1.0);
        });
    }
    //we now remove the free variables from the constraints
    for c in constraints.iter_mut() {
        c.remove_coefficients_by_index(&free_variables);
    }

    //and remove them from the domain
    for i in &free_variables {
        domain.remove(&variables[*i]);
    }
    //and remove them from the variables
    remove_many(&mut variables, &free_variables);

    constraints
        .iter_mut()
        .for_each(|c| c.ensure_size(context.total_variables));
    let (objective_offset, objective, flip_objective) = match optimization_type {
        OptimizationType::Max => (
            objective_offset,
            objective.iter().map(|c| c * -1.0).collect(),
            true,
        ),
        OptimizationType::Min => (objective_offset, objective.clone(), false),
    };
    Ok(StandardLinearModel::new(
        objective,
        constraints,
        variables,
        objective_offset,
        flip_objective,
    ))
}

pub struct NormalizationContext {
    pub surplus_index: usize,
    pub slack_index: usize,
    pub total_variables: usize,
}

pub fn normalize_constraint(
    constraint: LinearConstraint,
    context: &mut NormalizationContext,
) -> (EqualityConstraint, Option<String>) {
    let (mut coefficients, constraint_type, rhs) = constraint.into_parts();
    match constraint_type {
        Comparison::Equal => {
            let equality_constraint = EqualityConstraint::new(coefficients, rhs);
            (equality_constraint, None)
        }
        Comparison::LowerOrEqual => {
            coefficients.resize(context.total_variables, 0.0);
            coefficients.push(1.0);
            context.surplus_index += 1;
            let equality_constraint = EqualityConstraint::new(coefficients, rhs);
            (
                equality_constraint,
                Some(format!("$su_{}", context.surplus_index)),
            )
        }
        Comparison::UpperOrEqual => {
            coefficients.resize(context.total_variables, 0.0);
            coefficients.push(-1.0);
            context.slack_index += 1;
            let equality_constraint = EqualityConstraint::new(coefficients, rhs);
            (
                equality_constraint,
                Some(format!("$sl_{}", context.slack_index)),
            )
        }
    }
}
