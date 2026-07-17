use crate::math::{BinOp, UnOp, VariableType};
use crate::parser::model_transformer::{Constraint, DomainVariable, Exp};
use indexmap::{IndexMap, IndexSet};
use std::collections::VecDeque;

const DEFAULT_TOLERANCE: f64 = 1e-9;
const DEFAULT_MAX_STEPS: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Bounds {
    pub(crate) lower: f64,
    pub(crate) upper: f64,
}

impl Bounds {
    pub(crate) const UNBOUNDED: Self = Self {
        lower: f64::NEG_INFINITY,
        upper: f64::INFINITY,
    };

    pub(crate) fn new(lower: f64, upper: f64) -> Self {
        Self { lower, upper }
    }

    pub(crate) fn singleton(value: f64) -> Self {
        Self::new(value, value)
    }

    pub(crate) fn from_variable_type(variable_type: &VariableType) -> Self {
        match variable_type {
            VariableType::Boolean => Self::new(0.0, 1.0),
            VariableType::IntegerRange(lower, upper) => Self::new(*lower as f64, *upper as f64),
            VariableType::NonNegativeReal(lower, upper) | VariableType::Real(lower, upper) => {
                Self::new(*lower, *upper)
            }
        }
    }

    pub(crate) fn intersection(self, other: Self, tolerance: f64) -> Option<Self> {
        let lower = self.lower.max(other.lower);
        let upper = self.upper.min(other.upper);
        if lower <= upper {
            Some(Self::new(lower, upper))
        } else if lower - upper <= tolerance {
            Some(self)
        } else {
            None
        }
    }

    pub(crate) fn add(self, other: Self) -> Self {
        Self::new(
            lower_sum(self.lower, other.lower),
            upper_sum(self.upper, other.upper),
        )
    }

    pub(crate) fn sub(self, other: Self) -> Self {
        self.add(other.neg())
    }

    pub(crate) fn neg(self) -> Self {
        Self::new(-self.upper, -self.lower)
    }

    pub(crate) fn scale(self, coefficient: f64) -> Self {
        if coefficient == 0.0 {
            return Self::singleton(0.0);
        }
        if coefficient > 0.0 {
            Self::new(self.lower * coefficient, self.upper * coefficient)
        } else {
            Self::new(self.upper * coefficient, self.lower * coefficient)
        }
    }

    pub(crate) fn div_by(self, divisor: f64) -> Self {
        if divisor == 0.0 {
            Self::UNBOUNDED
        } else {
            self.scale(1.0 / divisor)
        }
    }

    pub(crate) fn abs(self) -> Self {
        if self.lower >= 0.0 {
            self
        } else if self.upper <= 0.0 {
            self.neg()
        } else {
            Self::new(0.0, (-self.lower).max(self.upper))
        }
    }
}

fn lower_sum(lhs: f64, rhs: f64) -> f64 {
    let value = lhs + rhs;
    if value.is_nan() {
        f64::NEG_INFINITY
    } else {
        value
    }
}

fn upper_sum(lhs: f64, rhs: f64) -> f64 {
    let value = lhs + rhs;
    if value.is_nan() { f64::INFINITY } else { value }
}

#[derive(Debug, Clone)]
pub(crate) struct BoundsAnalyzer {
    variable_bounds: IndexMap<String, Bounds>,
    tolerance: f64,
    reached_iteration_limit: bool,
    // a contradiction proves the model infeasible, which is a solver
    // outcome, not a compile error: propagation freezes and the original
    // constraint rows carry the infeasibility to the solver
    detected_infeasible: bool,
}

impl Default for BoundsAnalyzer {
    fn default() -> Self {
        Self {
            variable_bounds: IndexMap::new(),
            tolerance: DEFAULT_TOLERANCE,
            reached_iteration_limit: false,
            detected_infeasible: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct BoundsOptions {
    tolerance: f64,
    max_steps: usize,
}

impl Default for BoundsOptions {
    fn default() -> Self {
        Self {
            tolerance: DEFAULT_TOLERANCE,
            max_steps: DEFAULT_MAX_STEPS,
        }
    }
}

#[derive(Debug, Clone)]
struct AffineForm {
    coefficients: IndexMap<String, f64>,
    constant: f64,
}

impl AffineForm {
    fn from_constraint(constraint: &Constraint) -> Option<Self> {
        let mut lhs = Self::from_exp(constraint.lhs())?;
        lhs.merge(Self::from_exp(constraint.rhs())?, -1.0);
        Some(lhs)
    }

    fn from_exp(exp: &Exp) -> Option<Self> {
        match exp {
            Exp::Number(value) => Some(Self {
                coefficients: IndexMap::new(),
                constant: *value,
            }),
            Exp::Variable(name) => Some(Self {
                coefficients: IndexMap::from([(name.clone(), 1.0)]),
                constant: 0.0,
            }),
            Exp::BinOp(op, lhs, rhs) => match op {
                BinOp::Add => {
                    let mut lhs = Self::from_exp(lhs)?;
                    lhs.merge(Self::from_exp(rhs)?, 1.0);
                    Some(lhs)
                }
                BinOp::Sub => {
                    let mut lhs = Self::from_exp(lhs)?;
                    lhs.merge(Self::from_exp(rhs)?, -1.0);
                    Some(lhs)
                }
                BinOp::Mul => {
                    if let Exp::Number(coefficient) = &**lhs {
                        let mut rhs = Self::from_exp(rhs)?;
                        rhs.scale(*coefficient);
                        Some(rhs)
                    } else if let Exp::Number(coefficient) = &**rhs {
                        let mut lhs = Self::from_exp(lhs)?;
                        lhs.scale(*coefficient);
                        Some(lhs)
                    } else {
                        None
                    }
                }
                BinOp::Div => {
                    if let Exp::Number(divisor) = &**rhs {
                        if *divisor == 0.0 {
                            None
                        } else {
                            let mut lhs = Self::from_exp(lhs)?;
                            lhs.scale(1.0 / divisor);
                            Some(lhs)
                        }
                    } else {
                        None
                    }
                }
                BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff => None,
            },
            Exp::UnOp(op, inner) => match op {
                UnOp::Neg => {
                    let mut inner = Self::from_exp(inner)?;
                    inner.scale(-1.0);
                    Some(inner)
                }
                UnOp::Not => None,
            },
            Exp::Abs(_)
            | Exp::Min(_)
            | Exp::Max(_)
            | Exp::And(_)
            | Exp::Or(_)
            | Exp::Not(_)
            | Exp::Xor(_, _)
            | Exp::Implies(_, _)
            | Exp::Iff(_, _) => None,
        }
    }

    fn merge(&mut self, other: Self, multiplier: f64) {
        for (name, coefficient) in other.coefficients {
            let coefficient =
                self.coefficients.get(&name).copied().unwrap_or(0.0) + coefficient * multiplier;
            if coefficient == 0.0 {
                self.coefficients.shift_remove(&name);
            } else {
                self.coefficients.insert(name, coefficient);
            }
        }
        self.constant += other.constant * multiplier;
    }

    fn scale(&mut self, coefficient: f64) {
        self.coefficients.retain(|_, value| {
            *value *= coefficient;
            *value != 0.0
        });
        self.constant *= coefficient;
    }
}

impl BoundsAnalyzer {
    pub(crate) fn from_domain(domain: &IndexMap<String, DomainVariable>) -> Self {
        Self {
            variable_bounds: domain
                .iter()
                .map(|(name, variable)| {
                    (
                        name.clone(),
                        Bounds::from_variable_type(variable.get_type()),
                    )
                })
                .collect(),
            ..Self::default()
        }
    }

    pub(crate) fn analyze(
        domain: &IndexMap<String, DomainVariable>,
        constraints: &[Constraint],
    ) -> Self {
        Self::analyze_with_options(domain, constraints, BoundsOptions::default())
    }

    fn analyze_with_options(
        domain: &IndexMap<String, DomainVariable>,
        constraints: &[Constraint],
        options: BoundsOptions,
    ) -> Self {
        let mut analyzer = Self {
            tolerance: options.tolerance,
            ..Self::from_domain(domain)
        };
        analyzer.propagate_affine_constraints(constraints, options.max_steps);
        analyzer
    }

    pub(crate) fn bounds_of(&self, exp: &Exp) -> Bounds {
        match exp {
            Exp::Number(value) => Bounds::singleton(*value),
            Exp::Variable(name) => self
                .variable_bounds
                .get(name)
                .copied()
                .unwrap_or(Bounds::UNBOUNDED),
            Exp::Abs(inner) => self.bounds_of(inner).abs(),
            Exp::Min(exps) => {
                let mut bounds = exps.iter().map(|exp| self.bounds_of(exp));
                let Some(first) = bounds.next() else {
                    return Bounds::UNBOUNDED;
                };
                bounds.fold(first, |current, next| {
                    Bounds::new(current.lower.min(next.lower), current.upper.min(next.upper))
                })
            }
            Exp::Max(exps) => {
                let mut bounds = exps.iter().map(|exp| self.bounds_of(exp));
                let Some(first) = bounds.next() else {
                    return Bounds::UNBOUNDED;
                };
                bounds.fold(first, |current, next| {
                    Bounds::new(current.lower.max(next.lower), current.upper.max(next.upper))
                })
            }
            Exp::And(_)
            | Exp::Or(_)
            | Exp::Not(_)
            | Exp::Xor(_, _)
            | Exp::Implies(_, _)
            | Exp::Iff(_, _) => Bounds::new(0.0, 1.0),
            Exp::BinOp(op, lhs, rhs) => match op {
                BinOp::Add => self.bounds_of(lhs).add(self.bounds_of(rhs)),
                BinOp::Sub => self.bounds_of(lhs).sub(self.bounds_of(rhs)),
                BinOp::Mul => match (&**lhs, &**rhs) {
                    (Exp::Number(value), rhs) => self.bounds_of(rhs).scale(*value),
                    (lhs, Exp::Number(value)) => self.bounds_of(lhs).scale(*value),
                    (
                        Exp::Variable(_)
                        | Exp::Abs(_)
                        | Exp::Min(_)
                        | Exp::Max(_)
                        | Exp::And(_)
                        | Exp::Or(_)
                        | Exp::Not(_)
                        | Exp::Xor(_, _)
                        | Exp::Implies(_, _)
                        | Exp::Iff(_, _)
                        | Exp::BinOp(_, _, _)
                        | Exp::UnOp(_, _),
                        Exp::Variable(_)
                        | Exp::Abs(_)
                        | Exp::Min(_)
                        | Exp::Max(_)
                        | Exp::And(_)
                        | Exp::Or(_)
                        | Exp::Not(_)
                        | Exp::Xor(_, _)
                        | Exp::Implies(_, _)
                        | Exp::Iff(_, _)
                        | Exp::BinOp(_, _, _)
                        | Exp::UnOp(_, _),
                    ) => Bounds::UNBOUNDED,
                },
                BinOp::Div => match &**rhs {
                    Exp::Number(value) if *value != 0.0 => self.bounds_of(lhs).div_by(*value),
                    Exp::Number(_)
                    | Exp::Variable(_)
                    | Exp::Abs(_)
                    | Exp::Min(_)
                    | Exp::Max(_)
                    | Exp::And(_)
                    | Exp::Or(_)
                    | Exp::Not(_)
                    | Exp::Xor(_, _)
                    | Exp::Implies(_, _)
                    | Exp::Iff(_, _)
                    | Exp::BinOp(_, _, _)
                    | Exp::UnOp(_, _) => Bounds::UNBOUNDED,
                },
                BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff => {
                    Bounds::new(0.0, 1.0)
                }
            },
            Exp::UnOp(op, inner) => match op {
                UnOp::Neg => self.bounds_of(inner).neg(),
                UnOp::Not => Bounds::new(0.0, 1.0),
            },
        }
    }

    pub(crate) fn insert_variable(&mut self, name: String, variable_type: &VariableType) {
        self.variable_bounds
            .insert(name, Bounds::from_variable_type(variable_type));
    }

    pub(crate) fn apply_to_domain(&self, domain: &mut IndexMap<String, DomainVariable>) {
        for (name, variable) in domain {
            let Some(bounds) = self.variable_bounds.get(name).copied() else {
                continue;
            };
            let tightened_type = match variable.get_type() {
                VariableType::Boolean => VariableType::Boolean,
                VariableType::IntegerRange(_, _) => {
                    // round within tolerance first: propagation divides by
                    // coefficients, so a bound like 1.9 * (1 / 1.9) sits just
                    // below the integer it represents
                    let lower = (bounds.lower - self.tolerance).ceil();
                    let upper = (bounds.upper + self.tolerance).floor();
                    if lower > upper {
                        // There is no integral point in the inferred interval.
                        // Keep the declared domain: the original constraint
                        // rows will report infeasibility at solve time.
                        continue;
                    }
                    VariableType::IntegerRange(lower as i32, upper as i32)
                }
                VariableType::NonNegativeReal(_, _) => {
                    VariableType::NonNegativeReal(bounds.lower.max(0.0), bounds.upper)
                }
                VariableType::Real(_, _) => VariableType::Real(bounds.lower, bounds.upper),
            };
            variable.set_type(tightened_type);
        }
    }

    fn propagate_affine_constraints(&mut self, constraints: &[Constraint], max_steps: usize) {
        let forms = constraints
            .iter()
            .map(AffineForm::from_constraint)
            .collect::<Vec<_>>();
        let mut dependencies: IndexMap<String, Vec<usize>> = IndexMap::new();
        for (index, constraint) in constraints.iter().enumerate() {
            let names = if let Some(form) = &forms[index] {
                form.coefficients.keys().cloned().collect()
            } else {
                let mut names = IndexSet::new();
                collect_variables(constraint.lhs(), &mut names);
                collect_variables(constraint.rhs(), &mut names);
                names
            };
            for name in names {
                dependencies.entry(name).or_default().push(index);
            }
        }

        let mut queue = (0..constraints.len()).collect::<VecDeque<_>>();
        let mut queued = vec![true; constraints.len()];
        let mut steps = 0usize;
        while let Some(index) = queue.pop_front() {
            queued[index] = false;
            if steps >= max_steps {
                self.reached_iteration_limit = true;
                break;
            }
            steps += 1;
            let required = required_bounds(constraints[index].constraint_type());
            let mut changed = IndexSet::new();
            if let Some(form) = &forms[index] {
                changed
                    .extend(self.tighten_affine_form(form, constraints[index].constraint_type()));
            } else {
                self.tighten_constraint_expression(&constraints[index], required, &mut changed);
            }
            if self.detected_infeasible {
                // every bound recorded so far is a valid implication of the
                // constraints; stop refining and let the solver report the
                // infeasibility
                break;
            }
            for name in changed {
                if let Some(dependent_constraints) = dependencies.get(&name) {
                    for dependent in dependent_constraints {
                        if !queued[*dependent] {
                            queue.push_back(*dependent);
                            queued[*dependent] = true;
                        }
                    }
                }
            }
        }
    }

    fn tighten_affine_form(
        &mut self,
        form: &AffineForm,
        comparison: crate::math::Comparison,
    ) -> IndexSet<String> {
        let required = required_bounds(comparison);
        let mut changed = IndexSet::new();
        let terms = form
            .coefficients
            .iter()
            .map(|(name, coefficient)| {
                self.variable_bounds
                    .get(name)
                    .copied()
                    .unwrap_or(Bounds::UNBOUNDED)
                    .scale(*coefficient)
            })
            .collect::<Vec<_>>();
        let mut prefixes = Vec::with_capacity(terms.len() + 1);
        prefixes.push(Bounds::singleton(form.constant));
        for term in &terms {
            prefixes.push(prefixes.last().copied().unwrap().add(*term));
        }
        let mut suffixes = vec![Bounds::singleton(0.0); terms.len() + 1];
        for index in (0..terms.len()).rev() {
            suffixes[index] = terms[index].add(suffixes[index + 1]);
        }

        for (index, (name, coefficient)) in form.coefficients.iter().enumerate() {
            let others = prefixes[index].add(suffixes[index + 1]);
            let candidate = required.sub(others).div_by(*coefficient);
            if self.tighten_variable(name, candidate) {
                changed.insert(name.clone());
            }
            if self.detected_infeasible {
                break;
            }
        }

        let current = prefixes.last().copied().unwrap();
        if current.intersection(required, self.tolerance).is_none() {
            self.detected_infeasible = true;
        }

        changed
    }

    fn tighten_constraint_expression(
        &mut self,
        constraint: &Constraint,
        required: Bounds,
        changed: &mut IndexSet<String>,
    ) {
        let lhs_bounds = self.bounds_of(constraint.lhs());
        let rhs_bounds = self.bounds_of(constraint.rhs());
        let current = lhs_bounds.sub(rhs_bounds);
        let Some(required) = current.intersection(required, self.tolerance) else {
            self.detected_infeasible = true;
            return;
        };

        self.tighten_expression(constraint.lhs(), required.add(rhs_bounds), changed);
        self.tighten_expression(constraint.rhs(), lhs_bounds.sub(required), changed);
    }

    fn tighten_expression(&mut self, exp: &Exp, required: Bounds, changed: &mut IndexSet<String>) {
        if self.detected_infeasible {
            return;
        }
        let current = self.bounds_of(exp);
        let Some(required) = current.intersection(required, self.tolerance) else {
            self.detected_infeasible = true;
            return;
        };
        match exp {
            Exp::Number(_) => {}
            Exp::Variable(name) => {
                if self.tighten_variable(name, required) {
                    changed.insert(name.clone());
                }
            }
            Exp::Abs(inner) => {
                if required.upper.is_finite() {
                    self.tighten_expression(
                        inner,
                        Bounds::new(-required.upper, required.upper),
                        changed,
                    );
                }
            }
            Exp::Min(exps) => {
                if required.lower.is_finite() {
                    for exp in exps {
                        self.tighten_expression(
                            exp,
                            Bounds::new(required.lower, f64::INFINITY),
                            changed,
                        );
                    }
                }
            }
            Exp::Max(exps) => {
                if required.upper.is_finite() {
                    for exp in exps {
                        self.tighten_expression(
                            exp,
                            Bounds::new(f64::NEG_INFINITY, required.upper),
                            changed,
                        );
                    }
                }
            }
            Exp::And(_)
            | Exp::Or(_)
            | Exp::Not(_)
            | Exp::Xor(_, _)
            | Exp::Implies(_, _)
            | Exp::Iff(_, _) => {}
            Exp::BinOp(op, lhs, rhs) => match op {
                BinOp::Add => {
                    let lhs_bounds = self.bounds_of(lhs);
                    let rhs_bounds = self.bounds_of(rhs);
                    self.tighten_expression(lhs, required.sub(rhs_bounds), changed);
                    self.tighten_expression(rhs, required.sub(lhs_bounds), changed);
                }
                BinOp::Sub => {
                    let lhs_bounds = self.bounds_of(lhs);
                    let rhs_bounds = self.bounds_of(rhs);
                    self.tighten_expression(lhs, required.add(rhs_bounds), changed);
                    self.tighten_expression(rhs, lhs_bounds.sub(required), changed);
                }
                BinOp::Mul => {
                    if let Exp::Number(coefficient) = &**lhs {
                        if *coefficient != 0.0 {
                            self.tighten_expression(rhs, required.div_by(*coefficient), changed);
                        }
                    } else if let Exp::Number(coefficient) = &**rhs
                        && *coefficient != 0.0
                    {
                        self.tighten_expression(lhs, required.div_by(*coefficient), changed);
                    }
                }
                BinOp::Div => {
                    if let Exp::Number(divisor) = &**rhs
                        && *divisor != 0.0
                    {
                        self.tighten_expression(lhs, required.scale(*divisor), changed);
                    }
                }
                BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff => {}
            },
            Exp::UnOp(op, inner) => match op {
                UnOp::Neg => self.tighten_expression(inner, required.neg(), changed),
                UnOp::Not => {}
            },
        }
    }

    fn tighten_variable(&mut self, name: &str, candidate: Bounds) -> bool {
        let current = self
            .variable_bounds
            .get(name)
            .copied()
            .unwrap_or(Bounds::UNBOUNDED);
        let Some(tightened) = current.intersection(candidate, self.tolerance) else {
            self.detected_infeasible = true;
            return false;
        };
        let changed = tightened.lower > current.lower + self.tolerance
            || tightened.upper < current.upper - self.tolerance;
        if changed {
            self.variable_bounds.insert(name.to_string(), tightened);
        }
        changed
    }
}

fn required_bounds(comparison: crate::math::Comparison) -> Bounds {
    match comparison {
        crate::math::Comparison::LessOrEqual | crate::math::Comparison::Less => {
            Bounds::new(f64::NEG_INFINITY, 0.0)
        }
        crate::math::Comparison::GreaterOrEqual | crate::math::Comparison::Greater => {
            Bounds::new(0.0, f64::INFINITY)
        }
        crate::math::Comparison::Equal => Bounds::singleton(0.0),
    }
}

fn collect_variables(exp: &Exp, variables: &mut IndexSet<String>) {
    match exp {
        Exp::Number(_) => {}
        Exp::Variable(name) => {
            variables.insert(name.clone());
        }
        Exp::Abs(inner) | Exp::Not(inner) | Exp::UnOp(_, inner) => {
            collect_variables(inner, variables);
        }
        Exp::Min(exps) | Exp::Max(exps) | Exp::And(exps) | Exp::Or(exps) => {
            for exp in exps {
                collect_variables(exp, variables);
            }
        }
        Exp::Xor(lhs, rhs)
        | Exp::Implies(lhs, rhs)
        | Exp::Iff(lhs, rhs)
        | Exp::BinOp(_, lhs, rhs) => {
            collect_variables(lhs, variables);
            collect_variables(rhs, variables);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Bounds, BoundsAnalyzer, BoundsOptions, DEFAULT_TOLERANCE};
    use crate::math::{BinOp, Comparison, VariableType};
    use crate::parser::model_transformer::{Constraint, DomainVariable, Exp};
    use crate::utils::InputSpan;
    use indexmap::IndexMap;

    fn domain_with(entries: &[(&str, VariableType)]) -> IndexMap<String, DomainVariable> {
        entries
            .iter()
            .map(|(name, variable_type)| {
                (
                    (*name).to_string(),
                    DomainVariable::new(*variable_type, InputSpan::default()),
                )
            })
            .collect()
    }

    fn analyzer_with(entries: &[(&str, VariableType)]) -> BoundsAnalyzer {
        BoundsAnalyzer::analyze(&domain_with(entries), &[])
    }

    fn add_exp(lhs: Exp, rhs: Exp) -> Exp {
        Exp::BinOp(BinOp::Add, lhs.to_box(), rhs.to_box())
    }

    fn scale_exp(coefficient: f64, exp: Exp) -> Exp {
        Exp::BinOp(BinOp::Mul, Exp::Number(coefficient).to_box(), exp.to_box())
    }

    #[test]
    fn interval_arithmetic_is_conservative() {
        assert_eq!(
            Bounds::new(-2.0, 3.0).add(Bounds::new(4.0, 6.0)),
            Bounds::new(2.0, 9.0)
        );
        assert_eq!(
            Bounds::new(-2.0, 3.0).sub(Bounds::new(4.0, 6.0)),
            Bounds::new(-8.0, -1.0)
        );
        assert_eq!(Bounds::new(-2.0, 3.0).scale(-2.0), Bounds::new(-6.0, 4.0));
        assert_eq!(Bounds::new(-2.0, 3.0).scale(0.0), Bounds::singleton(0.0));
    }

    #[test]
    fn infinite_interval_operations_never_return_nan() {
        let sum = Bounds::new(f64::NEG_INFINITY, 4.0).add(Bounds::new(2.0, f64::INFINITY));
        assert!(!sum.lower.is_nan());
        assert!(!sum.upper.is_nan());
        assert_eq!(sum, Bounds::UNBOUNDED);

        let absolute = Bounds::UNBOUNDED.abs();
        assert_eq!(absolute, Bounds::new(0.0, f64::INFINITY));
    }

    #[test]
    fn variable_domains_intersections_and_division_have_expected_bounds() {
        assert_eq!(
            Bounds::from_variable_type(&VariableType::Boolean),
            Bounds::new(0.0, 1.0)
        );
        assert_eq!(
            Bounds::from_variable_type(&VariableType::IntegerRange(-3, 7)),
            Bounds::new(-3.0, 7.0)
        );
        assert_eq!(
            Bounds::from_variable_type(&VariableType::NonNegativeReal(2.0, 9.0)),
            Bounds::new(2.0, 9.0)
        );
        assert_eq!(
            Bounds::from_variable_type(&VariableType::Real(-4.0, 8.0)),
            Bounds::new(-4.0, 8.0)
        );
        assert_eq!(
            Bounds::new(0.0, 5.0).intersection(Bounds::new(2.0, 7.0), 1e-9),
            Some(Bounds::new(2.0, 5.0))
        );
        assert_eq!(
            Bounds::new(0.0, 1.0).intersection(Bounds::new(2.0, 3.0), 1e-9),
            None
        );
        assert_eq!(Bounds::new(-4.0, 8.0).div_by(-2.0), Bounds::new(-4.0, 2.0));
    }

    #[test]
    fn model_accessors_expose_analysis_inputs_and_allow_safe_domain_updates() {
        let constraint = Constraint::new(
            Exp::Variable("x".to_string()),
            Comparison::LessOrEqual,
            Exp::Number(3.0),
            "limit".to_string(),
        );
        assert_eq!(constraint.name(), "limit");
        assert!(matches!(constraint.lhs(), Exp::Variable(name) if name == "x"));
        assert!(matches!(constraint.rhs(), Exp::Number(value) if *value == 3.0));
        assert_eq!(constraint.constraint_type(), Comparison::LessOrEqual);

        let mut variable = DomainVariable::new(VariableType::Real(0.0, 10.0), InputSpan::default());
        variable.set_type(VariableType::Real(2.0, 5.0));
        assert_eq!(variable.get_type(), &VariableType::Real(2.0, 5.0));
    }

    #[test]
    fn computes_forward_bounds_for_affine_expressions() {
        let analyzer = analyzer_with(&[
            ("x", VariableType::Real(-2.0, 3.0)),
            ("y", VariableType::IntegerRange(1, 4)),
        ]);
        let exp = Exp::BinOp(
            BinOp::Sub,
            Exp::BinOp(
                BinOp::Mul,
                Exp::Number(-2.0).to_box(),
                Exp::Variable("x".to_string()).to_box(),
            )
            .to_box(),
            Exp::Variable("y".to_string()).to_box(),
        );
        assert_eq!(analyzer.bounds_of(&exp), Bounds::new(-10.0, 3.0));
    }

    #[test]
    fn affine_constraints_tighten_each_variable() {
        let domain = domain_with(&[
            ("x", VariableType::Real(f64::NEG_INFINITY, f64::INFINITY)),
            ("y", VariableType::Real(1.0, 2.0)),
        ]);
        let constraints = vec![Constraint::new(
            add_exp(
                scale_exp(2.0, Exp::Variable("x".to_string())),
                Exp::Variable("y".to_string()),
            ),
            Comparison::LessOrEqual,
            Exp::Number(8.0),
            "limit".to_string(),
        )];
        let analyzer = BoundsAnalyzer::analyze(&domain, &constraints);
        assert_eq!(
            analyzer.variable_bounds["x"],
            Bounds::new(f64::NEG_INFINITY, 3.5)
        );
    }

    #[test]
    fn chained_affine_constraints_are_revisited() {
        let domain = domain_with(&[
            ("x", VariableType::Real(f64::NEG_INFINITY, f64::INFINITY)),
            ("y", VariableType::Real(f64::NEG_INFINITY, f64::INFINITY)),
        ]);
        let constraints = vec![
            Constraint::new(
                Exp::Variable("y".to_string()),
                Comparison::Equal,
                add_exp(Exp::Variable("x".to_string()), Exp::Number(2.0)),
                "link".to_string(),
            ),
            Constraint::new(
                Exp::Variable("y".to_string()),
                Comparison::LessOrEqual,
                Exp::Number(5.0),
                "upper".to_string(),
            ),
        ];
        let analyzer = BoundsAnalyzer::analyze(&domain, &constraints);
        assert_eq!(analyzer.variable_bounds["x"].upper, 3.0);
    }

    #[test]
    fn constant_affine_contradictions_are_recorded_as_infeasible() {
        let constraints = vec![Constraint::new(
            Exp::Number(1.0),
            Comparison::LessOrEqual,
            Exp::Number(0.0),
            "impossible".to_string(),
        )];
        assert!(BoundsAnalyzer::analyze(&IndexMap::new(), &constraints).detected_infeasible);
    }

    #[test]
    fn contradictory_variable_bounds_are_recorded_as_infeasible() {
        let domain = domain_with(&[("x", VariableType::Real(0.0, 1.0))]);
        let constraints = vec![Constraint::new(
            Exp::Variable("x".to_string()),
            Comparison::GreaterOrEqual,
            Exp::Number(2.0),
            "bad".to_string(),
        )];
        assert!(BoundsAnalyzer::analyze(&domain, &constraints).detected_infeasible);
    }

    #[test]
    fn iteration_limit_keeps_safe_partial_bounds() {
        let domain = domain_with(&[
            ("x", VariableType::Real(f64::NEG_INFINITY, f64::INFINITY)),
            ("y", VariableType::Real(f64::NEG_INFINITY, f64::INFINITY)),
        ]);
        let constraints = vec![
            Constraint::new(
                Exp::Variable("y".to_string()),
                Comparison::Equal,
                add_exp(Exp::Variable("x".to_string()), Exp::Number(2.0)),
                "link".to_string(),
            ),
            Constraint::new(
                Exp::Variable("y".to_string()),
                Comparison::LessOrEqual,
                Exp::Number(5.0),
                "upper".to_string(),
            ),
        ];
        let analyzer = BoundsAnalyzer::analyze_with_options(
            &domain,
            &constraints,
            BoundsOptions {
                tolerance: DEFAULT_TOLERANCE,
                max_steps: 1,
            },
        );
        assert!(analyzer.reached_iteration_limit);
        assert!(
            analyzer
                .variable_bounds
                .values()
                .all(|bounds| bounds.lower <= bounds.upper)
        );
    }

    #[test]
    fn safe_piecewise_reverse_rules_tighten_operands() {
        let domain = domain_with(&[
            ("x", VariableType::Real(f64::NEG_INFINITY, f64::INFINITY)),
            ("y", VariableType::Real(f64::NEG_INFINITY, f64::INFINITY)),
        ]);
        let constraints = vec![
            Constraint::new(
                Exp::Abs(Exp::Variable("x".to_string()).to_box()),
                Comparison::LessOrEqual,
                Exp::Number(4.0),
                "abs_upper".to_string(),
            ),
            Constraint::new(
                Exp::Max(vec![
                    Exp::Variable("x".to_string()),
                    Exp::Variable("y".to_string()),
                ]),
                Comparison::LessOrEqual,
                Exp::Number(5.0),
                "max_upper".to_string(),
            ),
            Constraint::new(
                Exp::Min(vec![
                    Exp::Variable("x".to_string()),
                    Exp::Variable("y".to_string()),
                ]),
                Comparison::GreaterOrEqual,
                Exp::Number(-2.0),
                "min_lower".to_string(),
            ),
        ];
        let analyzer = BoundsAnalyzer::analyze(&domain, &constraints);
        assert_eq!(analyzer.variable_bounds["x"], Bounds::new(-2.0, 4.0));
        assert_eq!(analyzer.variable_bounds["y"], Bounds::new(-2.0, 5.0));
    }

    #[test]
    fn disjunctive_piecewise_reverse_rules_are_not_applied() {
        let domain = domain_with(&[
            ("x", VariableType::Real(-5.0, 5.0)),
            ("y", VariableType::Real(-5.0, 5.0)),
        ]);
        let constraints = vec![
            Constraint::new(
                Exp::Abs(Exp::Variable("x".to_string()).to_box()),
                Comparison::GreaterOrEqual,
                Exp::Number(3.0),
                "abs_lower".to_string(),
            ),
            Constraint::new(
                Exp::Max(vec![
                    Exp::Variable("x".to_string()),
                    Exp::Variable("y".to_string()),
                ]),
                Comparison::GreaterOrEqual,
                Exp::Number(4.0),
                "max_lower".to_string(),
            ),
            Constraint::new(
                Exp::Min(vec![
                    Exp::Variable("x".to_string()),
                    Exp::Variable("y".to_string()),
                ]),
                Comparison::LessOrEqual,
                Exp::Number(-4.0),
                "min_upper".to_string(),
            ),
        ];
        let analyzer = BoundsAnalyzer::analyze(&domain, &constraints);
        assert_eq!(analyzer.variable_bounds["x"], Bounds::new(-5.0, 5.0));
        assert_eq!(analyzer.variable_bounds["y"], Bounds::new(-5.0, 5.0));
    }

    #[test]
    fn tightened_bounds_are_copied_to_domains_with_conservative_rounding() {
        let domain = domain_with(&[
            ("x", VariableType::IntegerRange(-10, 10)),
            ("y", VariableType::NonNegativeReal(0.0, 10.0)),
            ("z", VariableType::Real(-10.0, 10.0)),
        ]);
        let constraints = vec![
            Constraint::new(
                Exp::Variable("x".to_string()),
                Comparison::GreaterOrEqual,
                Exp::Number(-2.2),
                "x_lower".to_string(),
            ),
            Constraint::new(
                Exp::Variable("x".to_string()),
                Comparison::LessOrEqual,
                Exp::Number(3.7),
                "x_upper".to_string(),
            ),
            Constraint::new(
                Exp::Variable("y".to_string()),
                Comparison::GreaterOrEqual,
                Exp::Number(2.0),
                "y_lower".to_string(),
            ),
            Constraint::new(
                Exp::Variable("z".to_string()),
                Comparison::LessOrEqual,
                Exp::Number(4.0),
                "z_upper".to_string(),
            ),
        ];
        let analyzer = BoundsAnalyzer::analyze(&domain, &constraints);
        let mut tightened = domain.clone();
        analyzer.apply_to_domain(&mut tightened);
        assert_eq!(
            tightened["x"].get_type(),
            &VariableType::IntegerRange(-2, 3)
        );
        assert_eq!(
            tightened["y"].get_type(),
            &VariableType::NonNegativeReal(2.0, 10.0)
        );
        assert_eq!(tightened["z"].get_type(), &VariableType::Real(-10.0, 4.0));
    }
}
