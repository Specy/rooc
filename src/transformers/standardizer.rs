use crate::math::float_eq;
use crate::math::{Comparison, OptimizationType, VariableType};
use crate::parser::model_transformer::DomainVariable;
use crate::solvers::{find_invalid_variables, SolverError};
use crate::transformers::linear_model::{LinearConstraint, LinearModel};
use crate::transformers::standard_linear_model::{EqualityConstraint, StandardLinearModel};
use crate::utils::{remove_many, InputSpan};

/// Converts a linear programming model into standard form.
///
/// Standard form requires:
/// - All constraints are equalities
/// - All variables are non-negative
/// - The objective function is minimization
///
/// # Arguments
/// * `problem` - The linear model to convert
///
/// # Returns
/// * `Ok(StandardLinearModel)` - The model in standard form
/// * `Err(SolverError)` - If the model cannot be converted
///
/// # Example
/// ```rust
/// use rooc::{OptimizationType, VariableType, Comparison, to_standard_form, LinearModel};
/// use rooc::model_transformer::DomainVariable;
///
/// let mut model = LinearModel::new();
///
/// // Add variables
/// model.add_variable("x", VariableType::NonNegativeReal(0.0, f64::INFINITY));
/// model.add_variable("y", VariableType::non_negative_real());
///
/// // Set objective function: minimize x + 2y
/// model.set_objective(vec![1.0, 2.0], OptimizationType::Min);
///
/// // Add constraint: x + y <= 10
/// model.add_constraint(vec![1.0, 1.0], Comparison::LessOrEqual, 10.0);
///
/// let standard_form = to_standard_form(model).unwrap();
///
/// ```
pub fn to_standard_form(problem: LinearModel) -> Result<StandardLinearModel, SolverError> {
    let (
        mut objective,
        optimization_type,
        objective_offset,
        mut constraints,
        mut variables,
        mut domain,
    ) = problem.into_parts();
    let mut context = NormalizationContext {
        surplus_index: 0,
        slack_index: 0,
        total_variables: variables.len(),
    };
    let invalid_variables = find_invalid_variables(&domain, |var| {
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
    //add constraints for variables that have bounds
    for (i, variable) in variables.iter().enumerate() {
        let domain_type = domain.get(variable).unwrap().get_type();

        match domain_type {
            VariableType::Real(min, max) => match (*min, *max) {
                (f64::NEG_INFINITY, f64::INFINITY) => continue,
                (min, max) if min != f64::NEG_INFINITY || max != f64::INFINITY => {
                    let mut coeffs = vec![0.0; variables.len()];
                    coeffs[i] = 1.0;
                    if min != f64::NEG_INFINITY {
                        constraints.push(LinearConstraint::new(
                            coeffs.clone(),
                            Comparison::GreaterOrEqual,
                            min,
                        ));
                    }
                    if max != f64::INFINITY {
                        constraints.push(LinearConstraint::new(
                            coeffs,
                            Comparison::LessOrEqual,
                            max,
                        ));
                    }
                }
                _ => (),
            },
            VariableType::NonNegativeReal(min, max) => match (*min, *max) {
                (0.0, f64::INFINITY) => continue,
                (min, max) if min != 0.0 || max != f64::INFINITY => {
                    let mut coeffs = vec![0.0; variables.len()];
                    coeffs[i] = 1.0;
                    if min != 0.0 {
                        constraints.push(LinearConstraint::new(
                            coeffs.clone(),
                            Comparison::GreaterOrEqual,
                            min,
                        ));
                    }
                    if max != f64::INFINITY {
                        constraints.push(LinearConstraint::new(
                            coeffs,
                            Comparison::LessOrEqual,
                            max,
                        ));
                    }
                }
                _ => ()
            },
            _ => (),
        }
    }
    //we now need to replace all free variables with positive variables
    let free_variables = variables
        .iter()
        .enumerate()
        .filter_map(|(i, v)| {
            let domain_variable = domain.get(v).unwrap();
            if matches!(domain_variable.get_type(), VariableType::Real(_, _)) {
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
            DomainVariable::new(
                VariableType::NonNegativeReal(0.0, f64::INFINITY),
                InputSpan::default(),
            ),
        );
        domain.insert(
            var_name2.clone(),
            DomainVariable::new(
                VariableType::NonNegativeReal(0.0, f64::INFINITY),
                InputSpan::default(),
            ),
        );
        context.total_variables += 1; //we add two variables, but one is removed, so only one is added
        constraints.iter_mut().for_each(|c| {
            let original_coefficient = c.coefficients()[*i];
            if float_eq(original_coefficient, 0.0) {
                return;
            }
            c.coefficients_mut().push(original_coefficient);
            c.coefficients_mut().push(original_coefficient * -1.0);
        });
        if float_eq(objective[*i], 0.0) {
            continue;
        }
        objective.push(objective[*i]);
        objective.push(objective[*i] * -1.0);
    }

    //we now remove the free variables from the constraints
    for c in constraints.iter_mut() {
        c.remove_coefficients_by_index(&free_variables);
    }

    //and remove them from the domain
    for i in &free_variables {
        domain.shift_remove(&variables[*i]);
    }
    //and remove them from the variables
    remove_many(&mut variables, &free_variables);

    //and from the objective
    remove_many(&mut objective, &free_variables);

    //we first normalize the constraints
    let mut constraints: Vec<EqualityConstraint> = constraints
        .into_iter()
        .map(|c| {
            let (equality_constraint, added_variable) = normalize_constraint(c, &mut context)?;
            if let Some(variable) = added_variable {
                variables.push(variable.clone());
                domain.insert(
                    variable,
                    DomainVariable::new(
                        VariableType::NonNegativeReal(0.0, f64::INFINITY),
                        InputSpan::default(),
                    ),
                );
                context.total_variables += 1;
            };
            Ok(equality_constraint)
        })
        .collect::<Result<_, _>>()?;

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
        _ => {
            return Err(SolverError::UnimplementedOptimizationType {
                expected: vec![OptimizationType::Max, OptimizationType::Min],
                got: optimization_type,
            })
        }
    };
    Ok(StandardLinearModel::new(
        objective,
        constraints,
        variables,
        objective_offset,
        flip_objective,
    ))
}

/// Context for tracking the normalization process of converting constraints to standard form.
///
/// This struct maintains indices for surplus and slack variables that are added during normalization,
/// as well as the total count of variables in the system.
pub struct NormalizationContext {
    /// Counter for surplus variables added to handle <= constraints
    pub surplus_index: usize,
    /// Counter for slack variables added to handle >= constraints
    pub slack_index: usize,
    /// Total number of variables in the system
    pub total_variables: usize,
}

/// Converts a linear constraint to standard form by adding slack or surplus variables as needed.
///
/// # Arguments
/// * `constraint` - The linear constraint to normalize
/// * `context` - Mutable reference to the normalization context
///
/// # Returns
/// * `Ok((EqualityConstraint, Option<String>))` - The normalized constraint and optionally the name of any added variable
/// * `Err(SolverError)` - If the constraint cannot be normalized
pub fn normalize_constraint(
    constraint: LinearConstraint,
    context: &mut NormalizationContext,
) -> Result<(EqualityConstraint, Option<String>), SolverError> {
    let (mut coefficients, constraint_type, rhs) = constraint.into_parts();
    match constraint_type {
        Comparison::Equal => {
            let equality_constraint = EqualityConstraint::new(coefficients, rhs);
            Ok((equality_constraint, None))
        }
        Comparison::LessOrEqual => {
            coefficients.resize(context.total_variables, 0.0);
            coefficients.push(1.0);
            context.surplus_index += 1;
            let equality_constraint = EqualityConstraint::new(coefficients, rhs);
            Ok((
                equality_constraint,
                Some(format!("$su_{}", context.surplus_index)),
            ))
        }
        Comparison::GreaterOrEqual => {
            coefficients.resize(context.total_variables, 0.0);
            coefficients.push(-1.0);
            context.slack_index += 1;
            let equality_constraint = EqualityConstraint::new(coefficients, rhs);
            Ok((
                equality_constraint,
                Some(format!("$sl_{}", context.slack_index)),
            ))
        }
        _ => Err(SolverError::UnavailableComparison {
            expected: vec![
                Comparison::Equal,
                Comparison::LessOrEqual,
                Comparison::GreaterOrEqual,
            ],
            got: constraint_type,
        }),
    }
}
