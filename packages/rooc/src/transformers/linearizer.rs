use crate::math::float_lt;
use crate::math::{BinOp, UnOp};
use crate::math::{Comparison, OptimizationType, VariableType};
use crate::parser::model_transformer::DomainVariable;
use crate::parser::model_transformer::{Constraint, Exp, Model};
use crate::transformers::bounds::BoundsAnalyzer;
use crate::transformers::linear_model::{LinearConstraint, LinearModel};
use crate::utils::InputSpan;
use indexmap::{IndexMap, IndexSet};
use std::collections::VecDeque;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValueRequirement {
    PreferLower,
    PreferHigher,
    Exact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExtremeKind {
    Min,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogicRequirement {
    MustBeTrue,
    MustBeFalse,
}

enum NormalizedLogicConstraint {
    Assertion {
        exp: Exp,
        requirement: LogicRequirement,
    },
    Tautology,
    Contradiction,
}

impl ValueRequirement {
    fn reversed(self) -> Self {
        match self {
            ValueRequirement::PreferLower => ValueRequirement::PreferHigher,
            ValueRequirement::PreferHigher => ValueRequirement::PreferLower,
            ValueRequirement::Exact => ValueRequirement::Exact,
        }
    }

    fn through_scale(self, coefficient: f64) -> Self {
        if coefficient < 0.0 {
            self.reversed()
        } else {
            self
        }
    }

    fn description(self) -> &'static str {
        match self {
            ValueRequirement::PreferLower => "a lower-favorable context",
            ValueRequirement::PreferHigher => "a higher-favorable context",
            ValueRequirement::Exact => "an exact-value context",
        }
    }
}

impl Exp {
    /// Converts an expression into a linear form.
    ///
    /// # Arguments
    /// * `linearizer_context` - The context containing variables and constraints
    ///
    /// # Returns
    /// * `Ok(LinearizationContext)` - The linearized expression
    /// * `Err(LinearizationError)` - If the expression cannot be linearized
    fn linearize(
        &self,
        linearizer_context: &mut Linearizer,
        requirement: ValueRequirement,
    ) -> Result<LinearizationContext, LinearizationError> {
        match self {
            Exp::BinOp(op, lhs, rhs) => match op {
                BinOp::Add => {
                    let mut lhs = lhs.linearize(linearizer_context, requirement)?;
                    let rhs = rhs.linearize(linearizer_context, requirement)?;
                    lhs.merge_add(rhs);
                    Ok(lhs)
                }
                BinOp::Sub => {
                    let mut lhs = lhs.linearize(linearizer_context, requirement)?;
                    let rhs = rhs.linearize(linearizer_context, requirement.reversed())?;
                    lhs.merge_sub(rhs);
                    Ok(lhs)
                }
                BinOp::Mul => {
                    if let Exp::Number(coefficient) = &**lhs {
                        // exact test: near-zero coefficients are still meaningful scales
                        if *coefficient == 0.0 {
                            return Ok(LinearizationContext::from_rhs(0.0));
                        }
                        let mut rhs = rhs.linearize(
                            linearizer_context,
                            requirement.through_scale(*coefficient),
                        )?;
                        rhs.mul_by(*coefficient);
                        Ok(rhs)
                    } else if let Exp::Number(coefficient) = &**rhs {
                        if *coefficient == 0.0 {
                            return Ok(LinearizationContext::from_rhs(0.0));
                        }
                        let mut lhs = lhs.linearize(
                            linearizer_context,
                            requirement.through_scale(*coefficient),
                        )?;
                        lhs.mul_by(*coefficient);
                        Ok(lhs)
                    } else {
                        Err(LinearizationError::NonLinearExpression(Box::new(
                            self.clone(),
                        )))
                    }
                }
                BinOp::Div => {
                    if let Exp::Number(divisor) = &**rhs {
                        // exact test: only a true zero divisor is an error
                        if *divisor == 0.0 {
                            return Err(LinearizationError::DivisionByZero(Box::new(self.clone())));
                        }
                        let mut lhs = lhs.linearize(
                            linearizer_context,
                            requirement.through_scale(1.0 / divisor),
                        )?;
                        lhs.div_by(*divisor);
                        Ok(lhs)
                    } else {
                        Err(LinearizationError::NonLinearExpression(Box::new(
                            self.clone(),
                        )))
                    }
                }
                BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff => Err(
                    LinearizationError::UnimplementedExpression(Box::new(self.clone())),
                ),
            },
            Exp::UnOp(op, exp) => match op {
                UnOp::Neg => {
                    let mut context = exp.linearize(linearizer_context, requirement.reversed())?;
                    context.mul_by(-1.0);
                    Ok(context)
                }
                UnOp::Not => Err(LinearizationError::UnimplementedExpression(Box::new(
                    self.clone(),
                ))),
            },
            Exp::Number(num) => Ok(LinearizationContext::from_rhs(*num)),
            Exp::Variable(name) => Ok(LinearizationContext::from_var(name.clone(), 1.0)),
            Exp::Min(exps) => {
                linearize_extreme(ExtremeKind::Min, exps, linearizer_context, requirement)
            }
            Exp::Max(exps) => {
                linearize_extreme(ExtremeKind::Max, exps, linearizer_context, requirement)
            }
            Exp::Not(exp) => {
                //not e is affine: 1 - e, no auxiliary variable is needed
                let mut context = exp.linearize(linearizer_context, ValueRequirement::Exact)?;
                if !is_binary_context(&context, &linearizer_context.domain) {
                    return Err(LinearizationError::NonBinaryLogicOperand(Box::new(
                        (**exp).clone(),
                    )));
                }
                context.mul_by(-1.0);
                context.add_rhs(1.0);
                Ok(context)
            }
            Exp::And(exps) => {
                if exps.is_empty() {
                    return Ok(LinearizationContext::from_rhs(1.0));
                }
                let operands =
                    linearize_binary_operands(exps, linearizer_context, ValueRequirement::Exact)?;
                let and_id = linearizer_context.and_count;
                linearizer_context.and_count += 1;
                let var_name = format!("$and_{and_id}");
                //z <= e_i for each operand, z >= sum(e_i) - (n - 1)
                let mut constraints: Vec<(Comparison, Exp)> = operands
                    .iter()
                    .map(|exp| (Comparison::LessOrEqual, exp.clone()))
                    .collect();
                constraints.push((
                    Comparison::GreaterOrEqual,
                    sub_exp(
                        sum_exps(&operands),
                        Exp::Number((operands.len() - 1) as f64),
                    ),
                ));
                reify_logic_variable(var_name, constraints, linearizer_context)
            }
            Exp::Or(exps) => {
                if exps.is_empty() {
                    return Ok(LinearizationContext::from_rhs(0.0));
                }
                let operands =
                    linearize_binary_operands(exps, linearizer_context, ValueRequirement::Exact)?;
                let or_id = linearizer_context.or_count;
                linearizer_context.or_count += 1;
                let var_name = format!("$or_{or_id}");
                //z >= e_i for each operand, z <= sum(e_i)
                let mut constraints: Vec<(Comparison, Exp)> = operands
                    .iter()
                    .map(|exp| (Comparison::GreaterOrEqual, exp.clone()))
                    .collect();
                constraints.push((Comparison::LessOrEqual, sum_exps(&operands)));
                reify_logic_variable(var_name, constraints, linearizer_context)
            }
            Exp::Implies(lhs, rhs) => {
                let operands = linearize_binary_operands(
                    &[(**lhs).clone(), (**rhs).clone()],
                    linearizer_context,
                    ValueRequirement::Exact,
                )?;
                let (a, b) = (operands[0].clone(), operands[1].clone());
                let implies_id = linearizer_context.implies_count;
                linearizer_context.implies_count += 1;
                let var_name = format!("$implies_{implies_id}");
                //z >= 1 - a, z >= b, z <= 1 - a + b
                let constraints = vec![
                    (
                        Comparison::GreaterOrEqual,
                        sub_exp(Exp::Number(1.0), a.clone()),
                    ),
                    (Comparison::GreaterOrEqual, b.clone()),
                    (
                        Comparison::LessOrEqual,
                        add_exp(sub_exp(Exp::Number(1.0), a), b),
                    ),
                ];
                reify_logic_variable(var_name, constraints, linearizer_context)
            }
            Exp::Iff(lhs, rhs) => {
                let operands = linearize_binary_operands(
                    &[(**lhs).clone(), (**rhs).clone()],
                    linearizer_context,
                    ValueRequirement::Exact,
                )?;
                let (a, b) = (operands[0].clone(), operands[1].clone());
                let iff_id = linearizer_context.iff_count;
                linearizer_context.iff_count += 1;
                let var_name = format!("$iff_{iff_id}");
                //z >= a + b - 1, z >= 1 - a - b, z <= 1 - a + b, z <= 1 + a - b
                let constraints = vec![
                    (
                        Comparison::GreaterOrEqual,
                        sub_exp(add_exp(a.clone(), b.clone()), Exp::Number(1.0)),
                    ),
                    (
                        Comparison::GreaterOrEqual,
                        sub_exp(sub_exp(Exp::Number(1.0), a.clone()), b.clone()),
                    ),
                    (
                        Comparison::LessOrEqual,
                        add_exp(sub_exp(Exp::Number(1.0), a.clone()), b.clone()),
                    ),
                    (
                        Comparison::LessOrEqual,
                        sub_exp(add_exp(Exp::Number(1.0), a), b),
                    ),
                ];
                reify_logic_variable(var_name, constraints, linearizer_context)
            }
            Exp::Xor(lhs, rhs) => {
                let operands = linearize_binary_operands(
                    &[(**lhs).clone(), (**rhs).clone()],
                    linearizer_context,
                    ValueRequirement::Exact,
                )?;
                let (a, b) = (operands[0].clone(), operands[1].clone());
                let xor_id = linearizer_context.xor_count;
                linearizer_context.xor_count += 1;
                let var_name = format!("$xor_{xor_id}");
                //z <= a + b, z >= a - b, z >= b - a, z <= 2 - a - b
                let constraints = vec![
                    (Comparison::LessOrEqual, add_exp(a.clone(), b.clone())),
                    (Comparison::GreaterOrEqual, sub_exp(a.clone(), b.clone())),
                    (Comparison::GreaterOrEqual, sub_exp(b.clone(), a.clone())),
                    (
                        Comparison::LessOrEqual,
                        sub_exp(sub_exp(Exp::Number(2.0), a), b),
                    ),
                ];
                reify_logic_variable(var_name, constraints, linearizer_context)
            }
            Exp::Abs(exp) => {
                let inner_bounds = linearizer_context.bounds.bounds_of(exp);
                if inner_bounds.lower >= 0.0 {
                    return exp.linearize(linearizer_context, requirement);
                }
                if inner_bounds.upper <= 0.0 {
                    let mut value = exp.linearize(linearizer_context, requirement.reversed())?;
                    value.mul_by(-1.0);
                    return Ok(value);
                }

                let needs_exact_value = match requirement {
                    ValueRequirement::PreferLower => false,
                    ValueRequirement::PreferHigher | ValueRequirement::Exact => true,
                };
                if needs_exact_value
                    && (!inner_bounds.lower.is_finite() || !inner_bounds.upper.is_finite())
                {
                    return Err(LinearizationError::MissingFiniteBounds {
                        expression: Box::new(self.clone()),
                        requirement: requirement.description(),
                        lower: inner_bounds.lower,
                        upper: inner_bounds.upper,
                        variables: variables_without_finite_bounds(exp, &linearizer_context.bounds),
                    });
                }

                let inner = exp.linearize(linearizer_context, ValueRequirement::Exact)?;
                let inner = context_to_exp(&inner);
                let abs_id = linearizer_context.abs_count;
                linearizer_context.abs_count += 1;
                let var_name = format!("$abs_{abs_id}");
                linearizer_context.declare_variable(
                    var_name.clone(),
                    VariableType::NonNegativeReal(
                        0.0,
                        (-inner_bounds.lower).max(inner_bounds.upper),
                    ),
                )?;
                linearizer_context.add_constraint(Constraint::new(
                    Exp::Variable(var_name.clone()),
                    Comparison::GreaterOrEqual,
                    inner.clone(),
                    String::new(),
                ));
                linearizer_context.add_constraint(Constraint::new(
                    Exp::Variable(var_name.clone()),
                    Comparison::GreaterOrEqual,
                    Exp::UnOp(UnOp::Neg, inner.clone().to_box()),
                    String::new(),
                ));

                if needs_exact_value {
                    let positive_name = format!("$abs_{abs_id}_positive");
                    linearizer_context
                        .declare_variable(positive_name.clone(), VariableType::Boolean)?;
                    linearizer_context.add_constraint(Constraint::new(
                        Exp::Variable(var_name.clone()),
                        Comparison::LessOrEqual,
                        sub_exp(
                            inner.clone(),
                            mul_exp(
                                Exp::Number(2.0 * inner_bounds.lower),
                                sub_exp(Exp::Number(1.0), Exp::Variable(positive_name.clone())),
                            ),
                        ),
                        String::new(),
                    ));
                    linearizer_context.add_constraint(Constraint::new(
                        Exp::Variable(var_name.clone()),
                        Comparison::LessOrEqual,
                        add_exp(
                            Exp::UnOp(UnOp::Neg, inner.to_box()),
                            mul_exp(
                                Exp::Number(2.0 * inner_bounds.upper),
                                Exp::Variable(positive_name),
                            ),
                        ),
                        String::new(),
                    ));
                }

                Ok(LinearizationContext::from_var(var_name, 1.0))
            }
        }
    }
}

fn linearize_extreme(
    kind: ExtremeKind,
    exps: &[Exp],
    linearizer_context: &mut Linearizer,
    requirement: ValueRequirement,
) -> Result<LinearizationContext, LinearizationError> {
    let kind_name = match kind {
        ExtremeKind::Min => "min",
        ExtremeKind::Max => "max",
    };
    if exps.is_empty() {
        return Err(LinearizationError::EmptyAggregation(kind_name));
    }

    let operand_bounds = exps
        .iter()
        .map(|exp| linearizer_context.bounds.bounds_of(exp))
        .collect::<Vec<_>>();
    let mut retained_indices = Vec::with_capacity(exps.len());
    for index in 0..exps.len() {
        let mut dominated = false;
        for other_index in 0..exps.len() {
            if index == other_index {
                continue;
            }
            let bounds = operand_bounds[index];
            let other_bounds = operand_bounds[other_index];
            let other_dominates = match kind {
                ExtremeKind::Max => other_bounds.lower >= bounds.upper,
                ExtremeKind::Min => other_bounds.upper <= bounds.lower,
            };
            if !other_dominates {
                continue;
            }
            let equal_fixed = bounds.lower == bounds.upper
                && other_bounds.lower == other_bounds.upper
                && bounds.lower == other_bounds.lower;
            if !equal_fixed || other_index < index {
                dominated = true;
                break;
            }
        }
        if !dominated {
            retained_indices.push(index);
        }
    }

    if retained_indices.is_empty() {
        return Err(LinearizationError::EmptyAggregation(kind_name));
    }
    if retained_indices.len() == 1 {
        return exps[retained_indices[0]].linearize(linearizer_context, requirement);
    }

    let retained_exps = retained_indices
        .iter()
        .map(|index| exps[*index].clone())
        .collect::<Vec<_>>();
    let retained_bounds = retained_indices
        .iter()
        .map(|index| operand_bounds[*index])
        .collect::<Vec<_>>();
    let extreme_exp = match kind {
        ExtremeKind::Min => Exp::Min(retained_exps.clone()),
        ExtremeKind::Max => Exp::Max(retained_exps.clone()),
    };
    let extreme_bounds = linearizer_context.bounds.bounds_of(&extreme_exp);
    let one_sided = match (kind, requirement) {
        (ExtremeKind::Max, ValueRequirement::PreferLower)
        | (ExtremeKind::Min, ValueRequirement::PreferHigher) => true,
        (ExtremeKind::Max, ValueRequirement::PreferHigher)
        | (ExtremeKind::Max, ValueRequirement::Exact)
        | (ExtremeKind::Min, ValueRequirement::PreferLower)
        | (ExtremeKind::Min, ValueRequirement::Exact) => false,
    };

    if !one_sided {
        let has_finite_bounds = match kind {
            ExtremeKind::Max => {
                extreme_bounds.upper.is_finite()
                    && retained_bounds
                        .iter()
                        .all(|bounds| bounds.lower.is_finite())
            }
            ExtremeKind::Min => {
                extreme_bounds.lower.is_finite()
                    && retained_bounds
                        .iter()
                        .all(|bounds| bounds.upper.is_finite())
            }
        };
        if !has_finite_bounds {
            return Err(LinearizationError::MissingFiniteBounds {
                expression: Box::new(extreme_exp.clone()),
                requirement: requirement.description(),
                lower: extreme_bounds.lower,
                upper: extreme_bounds.upper,
                variables: variables_without_finite_bounds(
                    &extreme_exp,
                    &linearizer_context.bounds,
                ),
            });
        }
    }

    let extreme_id = match kind {
        ExtremeKind::Min => {
            let id = linearizer_context.min_count;
            linearizer_context.min_count += 1;
            id
        }
        ExtremeKind::Max => {
            let id = linearizer_context.max_count;
            linearizer_context.max_count += 1;
            id
        }
    };
    let var_name = format!("${kind_name}_{extreme_id}");
    linearizer_context.declare_variable(
        var_name.clone(),
        VariableType::Real(extreme_bounds.lower, extreme_bounds.upper),
    )?;

    let operand_requirement = match (kind, one_sided) {
        (ExtremeKind::Max, true) => ValueRequirement::PreferLower,
        (ExtremeKind::Min, true) => ValueRequirement::PreferHigher,
        (ExtremeKind::Max, false) | (ExtremeKind::Min, false) => ValueRequirement::Exact,
    };
    let mut operands = Vec::with_capacity(retained_exps.len());
    for exp in &retained_exps {
        let value = exp.linearize(linearizer_context, operand_requirement)?;
        operands.push(context_to_exp(&value));
    }

    if one_sided {
        for operand in operands {
            let comparison = match kind {
                ExtremeKind::Min => Comparison::LessOrEqual,
                ExtremeKind::Max => Comparison::GreaterOrEqual,
            };
            linearizer_context.add_constraint(Constraint::new(
                Exp::Variable(var_name.clone()),
                comparison,
                operand,
                String::new(),
            ));
        }
        return Ok(LinearizationContext::from_var(var_name, 1.0));
    }

    let mut selectors = Vec::with_capacity(operands.len());
    for index in 0..operands.len() {
        let selector = format!("${kind_name}_{extreme_id}_select_{index}");
        linearizer_context.declare_variable(selector.clone(), VariableType::Boolean)?;
        selectors.push(Exp::Variable(selector));
    }
    for ((operand, bounds), selector) in operands
        .iter()
        .zip(retained_bounds.iter())
        .zip(selectors.iter())
    {
        match kind {
            ExtremeKind::Max => {
                linearizer_context.add_constraint(Constraint::new(
                    Exp::Variable(var_name.clone()),
                    Comparison::GreaterOrEqual,
                    operand.clone(),
                    String::new(),
                ));
                linearizer_context.add_constraint(Constraint::new(
                    Exp::Variable(var_name.clone()),
                    Comparison::LessOrEqual,
                    add_exp(
                        operand.clone(),
                        mul_exp(
                            Exp::Number(extreme_bounds.upper - bounds.lower),
                            sub_exp(Exp::Number(1.0), selector.clone()),
                        ),
                    ),
                    String::new(),
                ));
            }
            ExtremeKind::Min => {
                linearizer_context.add_constraint(Constraint::new(
                    Exp::Variable(var_name.clone()),
                    Comparison::LessOrEqual,
                    operand.clone(),
                    String::new(),
                ));
                linearizer_context.add_constraint(Constraint::new(
                    Exp::Variable(var_name.clone()),
                    Comparison::GreaterOrEqual,
                    sub_exp(
                        operand.clone(),
                        mul_exp(
                            Exp::Number(bounds.upper - extreme_bounds.lower),
                            sub_exp(Exp::Number(1.0), selector.clone()),
                        ),
                    ),
                    String::new(),
                ));
            }
        }
    }
    linearizer_context.add_constraint(Constraint::new(
        sum_exps(&selectors),
        Comparison::Equal,
        Exp::Number(1.0),
        String::new(),
    ));

    Ok(LinearizationContext::from_var(var_name, 1.0))
}

fn comparison_holds(lhs: f64, comparison: Comparison, rhs: f64) -> bool {
    match comparison {
        Comparison::LessOrEqual => lhs <= rhs,
        Comparison::GreaterOrEqual => lhs >= rhs,
        Comparison::Equal => lhs == rhs,
        Comparison::Less => lhs < rhs,
        Comparison::Greater => lhs > rhs,
    }
}

fn reversed_comparison(comparison: Comparison) -> Comparison {
    match comparison {
        Comparison::LessOrEqual => Comparison::GreaterOrEqual,
        Comparison::GreaterOrEqual => Comparison::LessOrEqual,
        Comparison::Equal => Comparison::Equal,
        Comparison::Less => Comparison::Greater,
        Comparison::Greater => Comparison::Less,
    }
}

fn is_logic_value(exp: &Exp, linearizer_context: &Linearizer) -> bool {
    match exp {
        Exp::Number(value) => *value == 0.0 || *value == 1.0,
        Exp::Variable(name) => matches!(
            linearizer_context
                .domain
                .get(name)
                .map(|variable| variable.get_type()),
            Some(VariableType::Boolean)
        ),
        Exp::And(_) | Exp::Or(_) | Exp::Xor(_, _) | Exp::Implies(_, _) | Exp::Iff(_, _) => true,
        Exp::Not(inner) => is_logic_value(inner, linearizer_context),
        Exp::BinOp(op, _, _) => match op {
            BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff => true,
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => false,
        },
        Exp::UnOp(op, inner) => match op {
            UnOp::Not => is_logic_value(inner, linearizer_context),
            UnOp::Neg => false,
        },
        Exp::Abs(_) | Exp::Min(_) | Exp::Max(_) => false,
    }
}

fn try_normalize_logic_constraint(
    lhs: &Exp,
    comparison: Comparison,
    rhs: &Exp,
    linearizer_context: &Linearizer,
) -> Option<NormalizedLogicConstraint> {
    let (exp, comparison, constant) = if let Exp::Number(constant) = rhs {
        if !is_logic_value(lhs, linearizer_context) {
            return None;
        }
        (lhs, comparison, *constant)
    } else if let Exp::Number(constant) = lhs {
        if !is_logic_value(rhs, linearizer_context) {
            return None;
        }
        (rhs, reversed_comparison(comparison), *constant)
    } else {
        return None;
    };
    if let Exp::Number(value) = exp {
        return Some(if comparison_holds(*value, comparison, constant) {
            NormalizedLogicConstraint::Tautology
        } else {
            NormalizedLogicConstraint::Contradiction
        });
    }
    match (
        comparison_holds(0.0, comparison, constant),
        comparison_holds(1.0, comparison, constant),
    ) {
        (false, true) => Some(NormalizedLogicConstraint::Assertion {
            exp: exp.clone(),
            requirement: LogicRequirement::MustBeTrue,
        }),
        (true, false) => Some(NormalizedLogicConstraint::Assertion {
            exp: exp.clone(),
            requirement: LogicRequirement::MustBeFalse,
        }),
        (true, true) => Some(NormalizedLogicConstraint::Tautology),
        (false, false) => Some(NormalizedLogicConstraint::Contradiction),
    }
}

fn binary_affine_value(exp: &Exp, linearizer_context: &Linearizer) -> Option<LinearizationContext> {
    match exp {
        Exp::Number(value) if *value == 0.0 || *value == 1.0 => {
            Some(LinearizationContext::from_rhs(*value))
        }
        Exp::Variable(name)
            if matches!(
                linearizer_context
                    .domain
                    .get(name)
                    .map(|variable| variable.get_type()),
                Some(VariableType::Boolean)
            ) =>
        {
            Some(LinearizationContext::from_var(name.clone(), 1.0))
        }
        Exp::Not(inner) => {
            let mut inner = binary_affine_value(inner, linearizer_context)?;
            inner.mul_by(-1.0);
            inner.add_rhs(1.0);
            Some(inner)
        }
        Exp::UnOp(op, inner) => match op {
            UnOp::Not => {
                let mut inner = binary_affine_value(inner, linearizer_context)?;
                inner.mul_by(-1.0);
                inner.add_rhs(1.0);
                Some(inner)
            }
            UnOp::Neg => None,
        },
        Exp::BinOp(op, _, _) => match op {
            BinOp::Add
            | BinOp::Sub
            | BinOp::Mul
            | BinOp::Div
            | BinOp::And
            | BinOp::Or
            | BinOp::Xor
            | BinOp::Implies
            | BinOp::Iff => None,
        },
        Exp::Number(_)
        | Exp::Variable(_)
        | Exp::Abs(_)
        | Exp::Min(_)
        | Exp::Max(_)
        | Exp::And(_)
        | Exp::Or(_)
        | Exp::Xor(_, _)
        | Exp::Implies(_, _)
        | Exp::Iff(_, _) => None,
    }
}

fn try_lower_affine_logic_assertion(
    exp: &Exp,
    must_be_true: bool,
    source_name: &str,
    linearizer_context: &mut Linearizer,
) -> Result<bool, LinearizationError> {
    match exp {
        Exp::Not(inner) => {
            try_lower_affine_logic_assertion(inner, !must_be_true, source_name, linearizer_context)
        }
        Exp::And(exps) => {
            let Some(operands) = exps
                .iter()
                .map(|exp| {
                    binary_affine_value(exp, linearizer_context).map(|value| context_to_exp(&value))
                })
                .collect::<Option<Vec<_>>>()
            else {
                return Ok(false);
            };
            let (comparison, rhs) = if must_be_true {
                (Comparison::Equal, operands.len() as f64)
            } else {
                (Comparison::LessOrEqual, operands.len() as f64 - 1.0)
            };
            linearizer_context.emit_constraint(
                sum_exps(&operands),
                comparison,
                Exp::Number(rhs),
                source_name.to_string(),
            )?;
            Ok(true)
        }
        Exp::Or(exps) => {
            let Some(operands) = exps
                .iter()
                .map(|exp| {
                    binary_affine_value(exp, linearizer_context).map(|value| context_to_exp(&value))
                })
                .collect::<Option<Vec<_>>>()
            else {
                return Ok(false);
            };
            linearizer_context.emit_constraint(
                sum_exps(&operands),
                if must_be_true {
                    Comparison::GreaterOrEqual
                } else {
                    Comparison::Equal
                },
                Exp::Number(if must_be_true { 1.0 } else { 0.0 }),
                source_name.to_string(),
            )?;
            Ok(true)
        }
        Exp::Implies(lhs, rhs) => {
            let Some(lhs) = binary_affine_value(lhs, linearizer_context) else {
                return Ok(false);
            };
            let Some(rhs) = binary_affine_value(rhs, linearizer_context) else {
                return Ok(false);
            };
            let lhs = context_to_exp(&lhs);
            let rhs = context_to_exp(&rhs);
            let (constraint_lhs, comparison, constraint_rhs) = if must_be_true {
                (lhs, Comparison::LessOrEqual, rhs)
            } else {
                (sub_exp(lhs, rhs), Comparison::Equal, Exp::Number(1.0))
            };
            linearizer_context.emit_constraint(
                constraint_lhs,
                comparison,
                constraint_rhs,
                source_name.to_string(),
            )?;
            Ok(true)
        }
        Exp::Iff(lhs, rhs) => {
            let Some(lhs) = binary_affine_value(lhs, linearizer_context) else {
                return Ok(false);
            };
            let Some(rhs) = binary_affine_value(rhs, linearizer_context) else {
                return Ok(false);
            };
            let lhs = context_to_exp(&lhs);
            let rhs = context_to_exp(&rhs);
            linearizer_context.emit_constraint(
                if must_be_true {
                    lhs
                } else {
                    add_exp(lhs, rhs.clone())
                },
                Comparison::Equal,
                if must_be_true { rhs } else { Exp::Number(1.0) },
                source_name.to_string(),
            )?;
            Ok(true)
        }
        Exp::Xor(lhs, rhs) => {
            let Some(lhs) = binary_affine_value(lhs, linearizer_context) else {
                return Ok(false);
            };
            let Some(rhs) = binary_affine_value(rhs, linearizer_context) else {
                return Ok(false);
            };
            let lhs = context_to_exp(&lhs);
            let rhs = context_to_exp(&rhs);
            linearizer_context.emit_constraint(
                if must_be_true {
                    add_exp(lhs, rhs.clone())
                } else {
                    lhs
                },
                Comparison::Equal,
                if must_be_true { Exp::Number(1.0) } else { rhs },
                source_name.to_string(),
            )?;
            Ok(true)
        }
        Exp::Number(_) | Exp::Variable(_) => {
            let Some(value) = binary_affine_value(exp, linearizer_context) else {
                return Ok(false);
            };
            linearizer_context.emit_constraint(
                context_to_exp(&value),
                Comparison::Equal,
                Exp::Number(if must_be_true { 1.0 } else { 0.0 }),
                source_name.to_string(),
            )?;
            Ok(true)
        }
        Exp::UnOp(op, inner) => match op {
            UnOp::Not => try_lower_affine_logic_assertion(
                inner,
                !must_be_true,
                source_name,
                linearizer_context,
            ),
            UnOp::Neg => Ok(false),
        },
        Exp::BinOp(op, _, _) => match op {
            BinOp::Add
            | BinOp::Sub
            | BinOp::Mul
            | BinOp::Div
            | BinOp::And
            | BinOp::Or
            | BinOp::Xor
            | BinOp::Implies
            | BinOp::Iff => Ok(false),
        },
        Exp::Abs(_) | Exp::Min(_) | Exp::Max(_) => Ok(false),
    }
}

fn lower_logic_assertion(
    exp: &Exp,
    must_be_true: bool,
    source_name: &str,
    linearizer_context: &mut Linearizer,
) -> Result<(), LinearizationError> {
    if let Exp::Number(value) = exp {
        if *value != 0.0 && *value != 1.0 {
            return Err(LinearizationError::NonBinaryLogicOperand(Box::new(
                exp.clone(),
            )));
        }
        let value_is_true = *value == 1.0;
        if value_is_true != must_be_true {
            linearizer_context.emit_constraint(
                Exp::Number(0.0),
                Comparison::Equal,
                Exp::Number(1.0),
                source_name.to_string(),
            )?;
        }
        return Ok(());
    }
    if try_lower_affine_logic_assertion(exp, must_be_true, source_name, linearizer_context)? {
        return Ok(());
    }

    match exp {
        Exp::And(exps) => {
            if must_be_true {
                for exp in exps {
                    lower_logic_assertion(exp, true, source_name, linearizer_context)?;
                }
            } else {
                let mut witnesses = Vec::with_capacity(exps.len());
                for exp in exps {
                    witnesses.push(directional_logic_witness(exp, false, linearizer_context)?);
                }
                linearizer_context.emit_constraint(
                    sum_exps(&witnesses),
                    Comparison::GreaterOrEqual,
                    Exp::Number(1.0),
                    source_name.to_string(),
                )?;
            }
            Ok(())
        }
        Exp::Or(exps) => {
            if must_be_true {
                let mut witnesses = Vec::with_capacity(exps.len());
                for exp in exps {
                    witnesses.push(directional_logic_witness(exp, true, linearizer_context)?);
                }
                linearizer_context.emit_constraint(
                    sum_exps(&witnesses),
                    Comparison::GreaterOrEqual,
                    Exp::Number(1.0),
                    source_name.to_string(),
                )?;
            } else {
                for exp in exps {
                    lower_logic_assertion(exp, false, source_name, linearizer_context)?;
                }
            }
            Ok(())
        }
        Exp::Not(inner) => {
            lower_logic_assertion(inner, !must_be_true, source_name, linearizer_context)
        }
        Exp::Implies(lhs, rhs) => {
            if must_be_true {
                let witnesses = vec![
                    directional_logic_witness(lhs, false, linearizer_context)?,
                    directional_logic_witness(rhs, true, linearizer_context)?,
                ];
                linearizer_context.emit_constraint(
                    sum_exps(&witnesses),
                    Comparison::GreaterOrEqual,
                    Exp::Number(1.0),
                    source_name.to_string(),
                )
            } else {
                lower_logic_assertion(lhs, true, source_name, linearizer_context)?;
                lower_logic_assertion(rhs, false, source_name, linearizer_context)
            }
        }
        Exp::Iff(lhs, rhs) => {
            let operands = linearize_binary_operands(
                &[(**lhs).clone(), (**rhs).clone()],
                linearizer_context,
                ValueRequirement::Exact,
            )?;
            if must_be_true {
                linearizer_context.emit_constraint(
                    operands[0].clone(),
                    Comparison::Equal,
                    operands[1].clone(),
                    source_name.to_string(),
                )
            } else {
                linearizer_context.emit_constraint(
                    add_exp(operands[0].clone(), operands[1].clone()),
                    Comparison::Equal,
                    Exp::Number(1.0),
                    source_name.to_string(),
                )
            }
        }
        Exp::Xor(lhs, rhs) => {
            let operands = linearize_binary_operands(
                &[(**lhs).clone(), (**rhs).clone()],
                linearizer_context,
                ValueRequirement::Exact,
            )?;
            if must_be_true {
                linearizer_context.emit_constraint(
                    add_exp(operands[0].clone(), operands[1].clone()),
                    Comparison::Equal,
                    Exp::Number(1.0),
                    source_name.to_string(),
                )
            } else {
                linearizer_context.emit_constraint(
                    operands[0].clone(),
                    Comparison::Equal,
                    operands[1].clone(),
                    source_name.to_string(),
                )
            }
        }
        Exp::UnOp(op, inner) => match op {
            UnOp::Not => {
                lower_logic_assertion(inner, !must_be_true, source_name, linearizer_context)
            }
            UnOp::Neg => Err(LinearizationError::NonBinaryLogicOperand(Box::new(
                exp.clone(),
            ))),
        },
        Exp::Variable(_) => {
            let value = binary_affine_value(exp, linearizer_context)
                .ok_or_else(|| LinearizationError::NonBinaryLogicOperand(Box::new(exp.clone())))?;
            linearizer_context.emit_constraint(
                context_to_exp(&value),
                Comparison::Equal,
                Exp::Number(if must_be_true { 1.0 } else { 0.0 }),
                source_name.to_string(),
            )
        }
        Exp::BinOp(op, _, _) => match op {
            BinOp::Add
            | BinOp::Sub
            | BinOp::Mul
            | BinOp::Div
            | BinOp::And
            | BinOp::Or
            | BinOp::Xor
            | BinOp::Implies
            | BinOp::Iff => Err(LinearizationError::NonBinaryLogicOperand(Box::new(
                exp.clone(),
            ))),
        },
        Exp::Number(_) | Exp::Abs(_) | Exp::Min(_) | Exp::Max(_) => Err(
            LinearizationError::NonBinaryLogicOperand(Box::new(exp.clone())),
        ),
    }
}

fn directional_logic_witness(
    exp: &Exp,
    witness_truth: bool,
    linearizer_context: &mut Linearizer,
) -> Result<Exp, LinearizationError> {
    if let Some(mut value) = binary_affine_value(exp, linearizer_context) {
        if !witness_truth {
            value.mul_by(-1.0);
            value.add_rhs(1.0);
        }
        return Ok(context_to_exp(&value));
    }

    match exp {
        Exp::And(exps) => {
            let mut children = Vec::with_capacity(exps.len());
            for exp in exps {
                children.push(directional_logic_witness(
                    exp,
                    witness_truth,
                    linearizer_context,
                )?);
            }
            let witness_id = linearizer_context.logic_witness_count;
            linearizer_context.logic_witness_count += 1;
            let witness_name = format!("$logic_witness_{witness_id}");
            linearizer_context.declare_variable(witness_name.clone(), VariableType::Boolean)?;
            if witness_truth {
                for child in children {
                    linearizer_context.emit_constraint(
                        Exp::Variable(witness_name.clone()),
                        Comparison::LessOrEqual,
                        child,
                        String::new(),
                    )?;
                }
            } else {
                linearizer_context.emit_constraint(
                    Exp::Variable(witness_name.clone()),
                    Comparison::LessOrEqual,
                    sum_exps(&children),
                    String::new(),
                )?;
            }
            Ok(Exp::Variable(witness_name))
        }
        Exp::Or(exps) => {
            let mut children = Vec::with_capacity(exps.len());
            for exp in exps {
                children.push(directional_logic_witness(
                    exp,
                    witness_truth,
                    linearizer_context,
                )?);
            }
            let witness_id = linearizer_context.logic_witness_count;
            linearizer_context.logic_witness_count += 1;
            let witness_name = format!("$logic_witness_{witness_id}");
            linearizer_context.declare_variable(witness_name.clone(), VariableType::Boolean)?;
            if witness_truth {
                linearizer_context.emit_constraint(
                    Exp::Variable(witness_name.clone()),
                    Comparison::LessOrEqual,
                    sum_exps(&children),
                    String::new(),
                )?;
            } else {
                for child in children {
                    linearizer_context.emit_constraint(
                        Exp::Variable(witness_name.clone()),
                        Comparison::LessOrEqual,
                        child,
                        String::new(),
                    )?;
                }
            }
            Ok(Exp::Variable(witness_name))
        }
        Exp::Not(inner) => directional_logic_witness(inner, !witness_truth, linearizer_context),
        Exp::Implies(lhs, rhs) => directional_logic_witness(
            &Exp::Or(vec![Exp::Not((**lhs).clone().to_box()), (**rhs).clone()]),
            witness_truth,
            linearizer_context,
        ),
        Exp::Iff(lhs, rhs) => {
            let operands = linearize_binary_operands(
                &[(**lhs).clone(), (**rhs).clone()],
                linearizer_context,
                ValueRequirement::Exact,
            )?;
            let a = operands[0].clone();
            let b = operands[1].clone();
            let witness_id = linearizer_context.logic_witness_count;
            linearizer_context.logic_witness_count += 1;
            let witness_name = format!("$logic_witness_{witness_id}");
            linearizer_context.declare_variable(witness_name.clone(), VariableType::Boolean)?;
            let upper_bounds = if witness_truth {
                vec![
                    add_exp(sub_exp(Exp::Number(1.0), a.clone()), b.clone()),
                    sub_exp(add_exp(Exp::Number(1.0), a), b),
                ]
            } else {
                vec![
                    add_exp(a.clone(), b.clone()),
                    sub_exp(sub_exp(Exp::Number(2.0), a), b),
                ]
            };
            for upper_bound in upper_bounds {
                linearizer_context.emit_constraint(
                    Exp::Variable(witness_name.clone()),
                    Comparison::LessOrEqual,
                    upper_bound,
                    String::new(),
                )?;
            }
            Ok(Exp::Variable(witness_name))
        }
        Exp::Xor(lhs, rhs) => directional_logic_witness(
            &Exp::Iff((**lhs).clone().to_box(), (**rhs).clone().to_box()),
            !witness_truth,
            linearizer_context,
        ),
        Exp::UnOp(op, inner) => match op {
            UnOp::Not => directional_logic_witness(inner, !witness_truth, linearizer_context),
            UnOp::Neg => Err(LinearizationError::NonBinaryLogicOperand(Box::new(
                exp.clone(),
            ))),
        },
        Exp::BinOp(op, _, _) => match op {
            BinOp::Add
            | BinOp::Sub
            | BinOp::Mul
            | BinOp::Div
            | BinOp::And
            | BinOp::Or
            | BinOp::Xor
            | BinOp::Implies
            | BinOp::Iff => Err(LinearizationError::NonBinaryLogicOperand(Box::new(
                exp.clone(),
            ))),
        },
        Exp::Number(_) | Exp::Variable(_) | Exp::Abs(_) | Exp::Min(_) | Exp::Max(_) => Err(
            LinearizationError::NonBinaryLogicOperand(Box::new(exp.clone())),
        ),
    }
}

/// Rebuilds a linear expression from a linearization context, as the sum of
/// its terms plus the constant.
fn context_to_exp(context: &LinearizationContext) -> Exp {
    let mut exp = Exp::Number(context.rhs());
    for (name, coeff) in context.vars() {
        let term = Exp::BinOp(
            BinOp::Mul,
            Exp::Number(*coeff).to_box(),
            Exp::Variable(name.clone()).to_box(),
        );
        exp = Exp::BinOp(BinOp::Add, exp.to_box(), term.to_box());
    }
    exp
}

/// Checks that a linearized expression is guaranteed to take 0/1 values so it
/// can be used as a logic operand: a 0/1 constant, a boolean variable or a
/// negated boolean variable (1 - x). Reified logic expressions are single
/// boolean auxiliary variables, so they pass this check too.
fn is_binary_context(
    context: &LinearizationContext,
    domain: &IndexMap<String, DomainVariable>,
) -> bool {
    match context.vars().len() {
        0 => context.rhs() == 0.0 || context.rhs() == 1.0,
        1 => {
            let (name, coeff) = context.vars().get_index(0).unwrap();
            let is_boolean = matches!(
                domain.get(name).map(|d| d.get_type()),
                Some(VariableType::Boolean)
            );
            is_boolean
                && ((*coeff == 1.0 && context.rhs() == 0.0)
                    || (*coeff == -1.0 && context.rhs() == 1.0))
        }
        2.. => false,
    }
}

/// Linearizes the operands of a logic expression, validating that each one is
/// a 0/1 valued expression, and rebuilds them as linear expressions.
fn linearize_binary_operands(
    exps: &[Exp],
    linearizer_context: &mut Linearizer,
    requirement: ValueRequirement,
) -> Result<Vec<Exp>, LinearizationError> {
    exps.iter()
        .map(|exp| {
            let context = exp.linearize(linearizer_context, requirement)?;
            if !is_binary_context(&context, &linearizer_context.domain) {
                return Err(LinearizationError::NonBinaryLogicOperand(Box::new(
                    exp.clone(),
                )));
            }
            Ok(context_to_exp(&context))
        })
        .collect()
}

/// Declares a boolean auxiliary variable tied to the given constraints and
/// returns it as the linearized value of a logic expression.
fn reify_logic_variable(
    var_name: String,
    constraints: Vec<(Comparison, Exp)>,
    linearizer_context: &mut Linearizer,
) -> Result<LinearizationContext, LinearizationError> {
    for (comparison, rhs) in constraints {
        linearizer_context.add_constraint(Constraint::new(
            Exp::Variable(var_name.clone()),
            comparison,
            rhs,
            String::new(),
        ));
    }
    linearizer_context.declare_variable(var_name.clone(), VariableType::Boolean)?;
    Ok(LinearizationContext::from_var(var_name, 1.0))
}

fn add_exp(lhs: Exp, rhs: Exp) -> Exp {
    Exp::BinOp(BinOp::Add, lhs.to_box(), rhs.to_box())
}

fn sub_exp(lhs: Exp, rhs: Exp) -> Exp {
    Exp::BinOp(BinOp::Sub, lhs.to_box(), rhs.to_box())
}

fn mul_exp(lhs: Exp, rhs: Exp) -> Exp {
    Exp::BinOp(BinOp::Mul, lhs.to_box(), rhs.to_box())
}

fn sum_exps(exps: &[Exp]) -> Exp {
    let mut iter = exps.iter().cloned();
    let first = iter.next().unwrap_or(Exp::Number(0.0));
    iter.fold(first, add_exp)
}

fn variables_without_finite_bounds(exp: &Exp, bounds: &BoundsAnalyzer) -> Vec<String> {
    let mut variables = IndexSet::new();
    let mut pending = vec![exp];
    while let Some(current) = pending.pop() {
        match current {
            Exp::Number(_) => {}
            Exp::Variable(name) => {
                variables.insert(name.clone());
            }
            Exp::Abs(inner) | Exp::Not(inner) | Exp::UnOp(_, inner) => {
                pending.push(inner);
            }
            Exp::Min(exps) | Exp::Max(exps) | Exp::And(exps) | Exp::Or(exps) => {
                pending.extend(exps);
            }
            Exp::Xor(lhs, rhs)
            | Exp::Implies(lhs, rhs)
            | Exp::Iff(lhs, rhs)
            | Exp::BinOp(_, lhs, rhs) => {
                pending.push(lhs);
                pending.push(rhs);
            }
        }
    }
    let mut variables = variables
        .into_iter()
        .filter(|name| {
            let variable = Exp::Variable(name.clone());
            let bounds = bounds.bounds_of(&variable);
            !bounds.lower.is_finite() || !bounds.upper.is_finite()
        })
        .collect::<Vec<_>>();
    variables.sort();
    variables
}

/// Represents an intermediate linear constraint during the linearization process.
#[derive(Debug)]
struct MidLinearConstraint {
    name: String,
    lhs: IndexMap<String, f64>,
    rhs: f64,
    comparison: Comparison,
}

impl MidLinearConstraint {
    /// Creates a new intermediate linear constraint.
    ///
    /// # Arguments
    /// * `lhs` - Map of variable names to their coefficients
    /// * `rhs` - Right-hand side constant
    /// * `comparison` - Comparison operator
    /// * `name` - Name of the constraint
    #[allow(unused)]
    pub fn new(lhs: IndexMap<String, f64>, rhs: f64, comparison: Comparison, name: String) -> Self {
        MidLinearConstraint {
            name,
            lhs,
            rhs,
            comparison,
        }
    }

    /// Creates a new constraint from a linearization context.
    ///
    /// # Arguments
    /// * `context` - The linearization context
    /// * `comparison` - The comparison operator
    pub fn new_from_linearized_context(
        context: LinearizationContext,
        comparison: Comparison,
        name: String,
    ) -> Self {
        MidLinearConstraint {
            lhs: context.current_vars,
            rhs: -context.current_rhs,
            comparison,
            name,
        }
    }

    /// Converts the constraint's variables to a coefficient vector based on variable ordering.
    ///
    /// # Arguments
    /// * `vars` - Mapping of variable names to their positions
    pub fn to_coefficient_vector(&self, vars: &IndexMap<String, usize>) -> Vec<f64> {
        extract_coeffs(&self.lhs, vars)
    }

    /// Converts to a final LinearConstraint.
    ///
    /// # Arguments
    /// * `vars` - Mapping of variable names to their positions
    pub fn into_linear_constraint(self, vars: &IndexMap<String, usize>) -> LinearConstraint {
        let coeffs = self.to_coefficient_vector(vars);
        LinearConstraint::new_with_name(coeffs, self.comparison, self.rhs, self.name)
    }
}

impl Display for MidLinearConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut lhs = String::new();
        for (name, val) in self.lhs.iter() {
            if float_lt(*val, 0.0) {
                lhs.push_str(&format!(" - {}{}", val.abs(), name));
            } else {
                lhs.push_str(&format!(" + {}{}", val, name));
            }
        }
        lhs.pop();
        write!(f, "{} {} {}", lhs, self.comparison, self.rhs)
    }
}

/// Manages the linearization process for expressions and constraints.
#[derive(Default)]
pub struct Linearizer {
    constraints: VecDeque<Constraint>,
    linear_constraints: Vec<MidLinearConstraint>,
    #[allow(dead_code)]
    surplus_count: u32,
    #[allow(dead_code)]
    slack_count: u32,
    min_count: u32,
    max_count: u32,
    abs_count: u32,
    and_count: u32,
    or_count: u32,
    xor_count: u32,
    implies_count: u32,
    iff_count: u32,
    logic_witness_count: u32,
    domain: IndexMap<String, DomainVariable>,
    bounds: BoundsAnalyzer,
}

impl Linearizer {
    /// Creates a new empty Linearizer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a Linearizer with initial constraints and domain.
    ///
    /// # Arguments
    /// * `constraints` - Initial constraints to process
    /// * `domain` - Variable domain information
    pub fn new_from(
        constraints: Vec<Constraint>,
        mut domain: IndexMap<String, DomainVariable>,
    ) -> Self {
        let bounds = BoundsAnalyzer::analyze(&domain, &constraints);
        bounds.apply_to_domain(&mut domain);
        Self::new_from_with_bounds(constraints, domain, bounds)
    }

    fn new_from_with_bounds(
        constraints: Vec<Constraint>,
        domain: IndexMap<String, DomainVariable>,
        bounds: BoundsAnalyzer,
    ) -> Self {
        Self {
            constraints: constraints.into_iter().collect(),
            domain,
            bounds,
            ..Self::default()
        }
    }

    /// Adds a constraint to be processed.
    ///
    /// # Arguments
    /// * `constraint` - The constraint to add
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push_front(constraint);
    }

    /// Returns a reference to the current constraints.
    pub fn constraints(&self) -> &VecDeque<Constraint> {
        &self.constraints
    }

    /// Removes and returns the next constraint to process.
    pub fn pop_constraint(&mut self) -> Option<Constraint> {
        self.constraints.pop_front()
    }

    fn emit_constraint(
        &mut self,
        lhs: Exp,
        comparison: Comparison,
        rhs: Exp,
        name: String,
    ) -> Result<(), LinearizationError> {
        let exp = Exp::BinOp(BinOp::Sub, lhs.to_box(), rhs.to_box())
            .flatten()
            .simplify();
        let requirement = match comparison {
            Comparison::LessOrEqual | Comparison::Less => ValueRequirement::PreferLower,
            Comparison::GreaterOrEqual | Comparison::Greater => ValueRequirement::PreferHigher,
            Comparison::Equal => ValueRequirement::Exact,
        };
        let value = exp.linearize(self, requirement)?;
        self.linear_constraints
            .push(MidLinearConstraint::new_from_linearized_context(
                value, comparison, name,
            ));
        Ok(())
    }

    /// Declares a new variable in the domain.
    ///
    /// # Arguments
    /// * `name` - Variable name
    /// * `as_type` - Variable type
    pub fn declare_variable(
        &mut self,
        name: String,
        as_type: VariableType,
    ) -> Result<(), LinearizationError> {
        if self.domain.contains_key(&name) {
            return Err(LinearizationError::VarAlreadyDeclared(name));
        }
        self.bounds.insert_variable(name.clone(), &as_type);
        let mut var = DomainVariable::new(as_type, InputSpan::default());
        var.increment_usage();
        self.domain.insert(name, var);
        Ok(())
    }

    /// Returns names of all variables that are used in constraints.
    pub fn used_variables(&self) -> Vec<String> {
        self.domain
            .iter()
            .filter(|(_, v)| v.is_used())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Converts a model into linear form.
    ///
    /// # Arguments
    /// * `model` - The model to linearize
    ///
    /// # Returns
    /// * `Ok(LinearModel)` - The linearized model
    /// * `Err(LinearizationError)` - If linearization fails
    pub fn linearize(model: Model) -> Result<LinearModel, LinearizationError> {
        let (objective, constraints, mut domain) = model.into_components();
        let bounds = BoundsAnalyzer::analyze(&domain, &constraints);
        bounds.apply_to_domain(&mut domain);
        let mut context = Linearizer::new_from_with_bounds(constraints, domain, bounds);
        let objective_type = objective.objective_type.clone();
        let objective_exp = objective.rhs.flatten().simplify();
        let objective_requirement = match &objective_type {
            OptimizationType::Min => ValueRequirement::PreferLower,
            OptimizationType::Max => ValueRequirement::PreferHigher,
            OptimizationType::Satisfy => ValueRequirement::Exact,
        };
        let linearized_objective = objective_exp.linearize(&mut context, objective_requirement)?;
        while let Some(constraint) = context.pop_constraint() {
            let is_logic_assertion = constraint.is_logic_assertion();
            let (lhs, op, rhs, name) = constraint.into_parts();
            let lhs = lhs.flatten().simplify();
            let rhs = rhs.flatten().simplify();
            if is_logic_assertion {
                lower_logic_assertion(&lhs, true, &name, &mut context)?;
                continue;
            }
            if let Some(normalized) = try_normalize_logic_constraint(&lhs, op, &rhs, &context) {
                match normalized {
                    NormalizedLogicConstraint::Tautology => continue,
                    NormalizedLogicConstraint::Contradiction => {
                        context.emit_constraint(
                            Exp::Number(0.0),
                            Comparison::Equal,
                            Exp::Number(1.0),
                            name,
                        )?;
                        continue;
                    }
                    NormalizedLogicConstraint::Assertion { exp, requirement } => {
                        let must_be_true = match requirement {
                            LogicRequirement::MustBeTrue => true,
                            LogicRequirement::MustBeFalse => false,
                        };
                        lower_logic_assertion(&exp, must_be_true, &name, &mut context)?;
                        continue;
                    }
                }
            }
            context.emit_constraint(lhs, op, rhs, name)?;
        }

        let mut linear_constraints = std::mem::take(&mut context.linear_constraints);
        // only user-provided names need dedup; generated rows stay unnamed.
        // duplicates get a __{n} suffix, which is still valid rooc syntax, and
        // a suffixed candidate never shadows a name the user wrote themselves
        let source_names: IndexSet<String> = linear_constraints
            .iter()
            .filter(|constraint| !constraint.name.is_empty())
            .map(|constraint| constraint.name.clone())
            .collect();
        let mut assigned_names: IndexSet<String> = IndexSet::new();
        for constraint in &mut linear_constraints {
            if constraint.name.is_empty() || assigned_names.insert(constraint.name.clone()) {
                continue;
            }
            let mut counter = 2usize;
            loop {
                let candidate = format!("{}__{}", constraint.name, counter);
                if !source_names.contains(&candidate) && !assigned_names.contains(&candidate) {
                    assigned_names.insert(candidate.clone());
                    constraint.name = candidate;
                    break;
                }
                counter += 1;
            }
        }
        let mut vars = context.used_variables();
        vars.sort();
        let domain = context
            .domain
            .into_iter()
            .filter(|(name, _)| vars.contains(name))
            .collect::<IndexMap<String, DomainVariable>>();
        let vars_indexes: IndexMap<String, usize> = vars
            .iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), i))
            .collect();
        let linear_constraints: Vec<LinearConstraint> = linear_constraints
            .into_iter()
            .map(|c| c.into_linear_constraint(&vars_indexes))
            .collect();
        let objective_coeffs = extract_coeffs(&linearized_objective.current_vars, &vars_indexes);
        let objective_offset = linearized_objective.current_rhs;
        Ok(LinearModel::new_from_parts(
            objective_coeffs,
            objective_type,
            objective_offset,
            linear_constraints,
            vars,
            domain,
        ))
    }
}

fn extract_coeffs(exp: &IndexMap<String, f64>, vars: &IndexMap<String, usize>) -> Vec<f64> {
    let mut vec = vec![0.0; vars.len()];
    for (name, val) in exp.iter() {
        // Skip names not present in `vars` rather than panicking (defensive: the normal
        // pipeline marks every referenced variable as used before this runs).
        if let Some(index) = vars.get(name) {
            vec[*index] = *val;
        }
    }
    vec
}

#[derive(Debug)]
pub enum LinearizationError {
    NonLinearExpression(Box<Exp>),
    DivisionByZero(Box<Exp>),
    EmptyAggregation(&'static str),
    VarAlreadyDeclared(String),
    UnimplementedExpression(Box<Exp>),
    NonBinaryLogicOperand(Box<Exp>),
    MissingFiniteBounds {
        expression: Box<Exp>,
        requirement: &'static str,
        lower: f64,
        upper: f64,
        variables: Vec<String>,
    },
}

impl Display for LinearizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinearizationError::NonLinearExpression(exp) => {
                write!(f, "Non linear expression: \"{}\"", exp)
            }
            LinearizationError::DivisionByZero(exp) => {
                write!(f, "Division by zero in expression: \"{}\"", exp)
            }
            LinearizationError::EmptyAggregation(kind) => {
                write!(
                    f,
                    "Numeric {} aggregation requires at least one operand",
                    kind
                )
            }
            LinearizationError::VarAlreadyDeclared(name) => {
                write!(f, "Variable \"{}\" already declared", name)
            }
            LinearizationError::UnimplementedExpression(exp) => {
                write!(f, "Unimplemented expression: \"{}\"", exp)
            }
            LinearizationError::NonBinaryLogicOperand(exp) => {
                write!(f, "Logic operands must be boolean values, got: \"{}\"", exp)
            }
            LinearizationError::MissingFiniteBounds {
                expression,
                requirement,
                lower,
                upper,
                variables,
            } => {
                let variables = if variables.is_empty() {
                    "none identified".to_string()
                } else {
                    variables.join(", ")
                };
                write!(
                    f,
                    "Cannot linearize \"{}\" in {} with derived bounds [{}, {}]. \
                     Variables without finite bounds: {}. Declare finite bounds or add \
                     constraints from which finite bounds can be inferred",
                    expression, requirement, lower, upper, variables
                )
            }
        }
    }
}

impl std::error::Error for LinearizationError {}

/// Represents the intermediate state during expression linearization.
/// Contains a map of variables to their coefficients and a constant term (RHS).
struct LinearizationContext {
    current_vars: IndexMap<String, f64>,
    current_rhs: f64,
}

impl Default for LinearizationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl LinearizationContext {
    /// Creates a new empty linearization context.
    pub fn new() -> Self {
        LinearizationContext {
            current_vars: IndexMap::new(),
            current_rhs: 0.0,
        }
    }

    /// Creates a new context with a single variable term.
    ///
    /// # Arguments
    /// * `name` - Name of the variable
    /// * `multiplier` - Coefficient for the variable
    pub fn from_var(name: String, multiplier: f64) -> Self {
        let mut context = LinearizationContext::new();
        context.add_var(name, multiplier);
        context
    }

    /// Creates a new context with only a constant term.
    ///
    /// # Arguments
    /// * `rhs` - The constant value
    pub fn from_rhs(rhs: f64) -> Self {
        let mut context = LinearizationContext::new();
        context.add_rhs(rhs);
        context
    }

    /// Adds a variable term to the context, combining coefficients if the variable already exists.
    ///
    /// # Arguments
    /// * `name` - Name of the variable
    /// * `multiplier` - Coefficient to add for the variable
    #[allow(clippy::all)]
    pub fn add_var(&mut self, name: String, multiplier: f64) {
        if self.current_vars.contains_key(&name) {
            let val = self.current_vars.get_mut(&name).unwrap();
            *val += multiplier;
        } else {
            self.current_vars.insert(name, multiplier);
        }
    }

    /// Adds another context to this one, combining like terms.
    ///
    /// # Arguments
    /// * `other` - The context to add
    pub fn merge_add(&mut self, other: LinearizationContext) {
        for (name, multiplier) in other.current_vars {
            self.add_var(name, multiplier);
        }
        self.add_rhs(other.current_rhs);
    }

    /// Subtracts another context from this one.
    ///
    /// # Arguments
    /// * `other` - The context to subtract
    pub fn merge_sub(&mut self, other: LinearizationContext) {
        for (name, multiplier) in other.current_vars {
            self.add_var(name, -multiplier);
        }
        self.add_rhs(-other.current_rhs);
    }

    /// Adds a constant term to the RHS.
    ///
    /// # Arguments
    /// * `rhs` - The constant value to add
    pub fn add_rhs(&mut self, rhs: f64) {
        self.current_rhs += rhs;
    }

    #[allow(unused)]
    /// Returns a reference to the map of variables and their coefficients.
    pub fn vars(&self) -> &IndexMap<String, f64> {
        &self.current_vars
    }

    /// Returns the constant term (RHS).
    pub fn rhs(&self) -> f64 {
        self.current_rhs
    }

    #[allow(unused)]
    /// Checks if a variable exists in the context.
    ///
    /// # Arguments
    /// * `name` - Name of the variable to check
    pub fn has_var(&self, name: &String) -> bool {
        self.current_vars.contains_key(name)
    }

    /// Multiplies all coefficients and the RHS by a scalar value.
    ///
    /// # Arguments
    /// * `multiplier` - The scalar value to multiply by
    pub fn mul_by(&mut self, multiplier: f64) {
        for (_, val) in self.current_vars.iter_mut() {
            *val *= multiplier;
        }
        self.current_rhs *= multiplier;
    }

    /// Divides all coefficients and the RHS by a scalar value.
    ///
    /// # Arguments
    /// * `divisor` - The scalar value to divide by
    pub fn div_by(&mut self, divisor: f64) {
        for (_, val) in self.current_vars.iter_mut() {
            *val /= divisor;
        }
        self.current_rhs /= divisor;
    }
}

#[cfg(test)]
mod tests {
    use super::{Linearizer, ValueRequirement};
    use crate::math::{Comparison, VariableType};
    use crate::parser::model_transformer::{Constraint, DomainVariable, Exp};
    use crate::utils::InputSpan;
    use indexmap::IndexMap;

    #[test]
    fn value_requirements_reverse_only_through_negative_signs() {
        assert_eq!(
            ValueRequirement::PreferLower.through_scale(-2.0),
            ValueRequirement::PreferHigher
        );
        assert_eq!(
            ValueRequirement::PreferHigher.through_scale(3.0),
            ValueRequirement::PreferHigher
        );
        assert_eq!(
            ValueRequirement::Exact.through_scale(-1.0),
            ValueRequirement::Exact
        );
    }

    #[test]
    fn new_from_uses_constraint_derived_bounds() {
        let variable = || Exp::Variable("x".to_string());
        let constraints = vec![
            Constraint::new(
                variable(),
                Comparison::GreaterOrEqual,
                Exp::Number(-3.0),
                "lower".to_string(),
            ),
            Constraint::new(
                variable(),
                Comparison::LessOrEqual,
                Exp::Number(2.0),
                "upper".to_string(),
            ),
        ];
        let mut domain = IndexMap::new();
        domain.insert(
            "x".to_string(),
            DomainVariable::new(VariableType::real(), InputSpan::default()),
        );

        let mut linearizer = Linearizer::new_from(constraints, domain);
        let result = linearizer.emit_constraint(
            Exp::Abs(variable().to_box()),
            Comparison::Equal,
            Exp::Number(1.0),
            "absolute".to_string(),
        );

        assert!(result.is_ok());
    }
}
