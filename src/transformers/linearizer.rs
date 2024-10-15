use std::collections::{HashMap, VecDeque};
use std::fmt::Display;

use crate::math::math_enums::{Comparison, VariableType};
use crate::math::operators::{BinOp, UnOp};
use crate::parser::model_transformer::model::{Constraint, Exp, Model};
use crate::parser::model_transformer::transformer_context::DomainVariable;
use crate::transformers::linear_model::{LinearConstraint, LinearModel};
use crate::utils::InputSpan;

/**TODO
The linearizer module contains the code for attempting to linearize a problem into a linear problem
where the lhs is formed only by addition of variables with a constant multiplier and the rhs is formed by a single constant value.
It achieves this by following the linearization rules, where:
1. Multiplication and division of two variables is not permitted as it is not linear, but linearized if
   the lhs is all divided/multiplied by a variable, and the rhs is a constant.
2. The MIN function can be converted in a linear way by:
     min(x1, x2) + y <= b
      BECOMES:
3. The MAX function can be converted in a linear way by:
     max(x1, x2) <= b
      BECOMES:
4. The ABS function can be converted in a linear way by:
     |x1| + y <= b
       BECOMES:
      x1 + y>= b

    //TODO
      x1 - y>= -b
    -(-x1 + y) >= -b


 */

impl Exp {
    fn linearize(
        &self,
        linearizer_context: &mut Linearizer,
    ) -> Result<LinearizationContext, LinearizationError> {
        match self {
            Exp::BinOp(op, lhs, rhs) => {
                let mut lhs = lhs.linearize(linearizer_context)?;
                let mut rhs = rhs.linearize(linearizer_context)?;
                let context = match op {
                    BinOp::Add => {
                        lhs.merge_add(rhs);
                        lhs
                    }
                    BinOp::Sub => {
                        lhs.merge_sub(rhs);
                        lhs
                    }
                    BinOp::Mul => {
                        if lhs.has_no_vars() {
                            rhs.mul_by(lhs.get_rhs());
                            rhs
                        } else if rhs.has_no_vars() {
                            lhs.mul_by(rhs.get_rhs());
                            lhs
                        } else {
                            return Err(LinearizationError::NonLinearExpression(Box::new(
                                self.clone(),
                            )));
                        }
                    }
                    BinOp::Div => {
                        if rhs.has_no_vars() {
                            lhs.div_by(rhs.get_rhs());
                            lhs
                        } else if lhs.has_no_vars() {
                            rhs.div_by(lhs.get_rhs());
                            rhs
                        } else {
                            return Err(LinearizationError::NonLinearExpression(Box::new(
                                self.clone(),
                            )));
                        }
                    }
                };
                Ok(context)
            }
            Exp::UnOp(op, exp) => match op {
                UnOp::Neg => {
                    let mut context = exp.linearize(linearizer_context)?;
                    context.mul_by(-1.0);
                    Ok(context)
                }
            },
            Exp::Number(num) => Ok(LinearizationContext::from_rhs(*num)),
            Exp::Variable(name) => Ok(LinearizationContext::from_var(name.clone(), 1.0)),
            Exp::Min(exps) => {
                let var_name = format!("$min_{}", linearizer_context.min_count);
                linearizer_context.min_count += 1;
                for exp in exps {
                    let constraint = Constraint::new(
                        Exp::Variable(var_name.clone()).clone(),
                        Comparison::LowerOrEqual,
                        exp.clone(),
                    );
                    linearizer_context.add_constraint(constraint)
                }
                linearizer_context.declare_variable(var_name.clone(), VariableType::Real)?;
                Ok(LinearizationContext::from_var(var_name, 1.0))
            }
            Exp::Max(exps) => {
                let var_name = format!("$max_{}", linearizer_context.max_count);
                linearizer_context.max_count += 1;
                for exp in exps {
                    let constraint = Constraint::new(
                        Exp::Variable(var_name.clone()).clone(),
                        Comparison::UpperOrEqual,
                        exp.clone(),
                    );
                    linearizer_context.add_constraint(constraint)
                }
                linearizer_context.declare_variable(var_name.clone(), VariableType::Real)?;
                Ok(LinearizationContext::from_var(var_name, 1.0))
            }
            Exp::Abs(_) => Err(LinearizationError::UnimplementedExpression(Box::new(
                self.clone(),
            ))),
        }
    }
}

#[derive(Debug)]
pub struct MidLinearConstraint {
    lhs: HashMap<String, f64>,
    rhs: f64,
    comparison: Comparison,
}
impl MidLinearConstraint {
    pub fn new(lhs: HashMap<String, f64>, rhs: f64, comparison: Comparison) -> Self {
        MidLinearConstraint {
            lhs,
            rhs,
            comparison,
        }
    }
    pub fn new_from_linearized_context(
        context: LinearizationContext,
        comparison: Comparison,
    ) -> Self {
        MidLinearConstraint {
            lhs: context.current_vars,
            rhs: -context.current_rhs,
            comparison,
        }
    }
    pub fn to_coefficient_vector(&self, vars: &HashMap<String, usize>) -> Vec<f64> {
        extract_coeffs(&self.lhs, vars)
    }
    pub fn to_linear_constraint(self, vars: &HashMap<String, usize>) -> LinearConstraint {
        let coeffs = self.to_coefficient_vector(vars);
        LinearConstraint::new(coeffs, self.comparison, self.rhs)
    }
}
impl Display for MidLinearConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut lhs = String::new();
        for (name, val) in self.lhs.iter() {
            if *val < 0.0 {
                lhs.push_str(&format!(" - {}{}", val.abs(), name));
            } else {
                lhs.push_str(&format!(" + {}{}", val, name));
            }
        }
        lhs.pop();
        write!(f, "{} {} {}", lhs, self.comparison, self.rhs)
    }
}

#[derive(Default)]
pub struct Linearizer {
    constraints: VecDeque<Constraint>,
    #[allow(dead_code)]
    surplus_count: u32,
    #[allow(dead_code)]
    slack_count: u32,
    min_count: u32,
    max_count: u32,
    domain: HashMap<String, DomainVariable>,
}

impl Linearizer {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_from(constraints: Vec<Constraint>, domain: HashMap<String, DomainVariable>) -> Self {
        let mut context = Self::default();
        context.constraints = constraints.into_iter().collect();
        context.domain = domain;
        context
    }
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push_front(constraint);
    }

    pub fn get_constraints(&self) -> &VecDeque<Constraint> {
        &self.constraints
    }
    pub fn pop_constraint(&mut self) -> Option<Constraint> {
        self.constraints.pop_front()
    }
    pub fn declare_variable(
        &mut self,
        name: String,
        as_type: VariableType,
    ) -> Result<(), LinearizationError> {
        if self.domain.contains_key(&name) {
            return Err(LinearizationError::VarAlreadyDeclared(name));
        }
        let mut var = DomainVariable::new(as_type, InputSpan::default());
        var.increment_usage();
        self.domain.insert(name, var);
        Ok(())
    }
    pub fn get_used_variables(&self) -> Vec<String> {
        self.domain
            .iter()
            .filter(|(_, v)| v.is_used())
            .map(|(name, _)| name.clone())
            .collect()
    }
    pub fn linearize(model: Model) -> Result<LinearModel, LinearizationError> {
        let (objective, constraints, domain) = model.into_components();

        let mut context = Linearizer::new_from(constraints, domain);
        let mut linear_constraints: Vec<MidLinearConstraint> = Vec::new();
        let objective_type = objective.objective_type.clone();
        let objective_exp = objective.rhs.flatten().simplify();
        let linearized_objective = objective_exp.linearize(&mut context)?;
        while let Some(constraint) = context.pop_constraint() {
            let (lhs, op, rhs) = constraint.into_parts();
            let exp = Exp::BinOp(BinOp::Sub, Box::new(lhs), Box::new(rhs))
                .flatten()
                .simplify();
            let res = exp.linearize(&mut context)?;
            linear_constraints.push(MidLinearConstraint::new_from_linearized_context(res, op));
        }
        let mut vars = context.get_used_variables();
        vars.sort();
        let vars_indexes: HashMap<String, usize> = vars
            .iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), i))
            .collect();
        let linear_constraints: Vec<LinearConstraint> = linear_constraints
            .into_iter()
            .map(|c| c.to_linear_constraint(&vars_indexes))
            .collect();
        let objective_coeffs = extract_coeffs(&linearized_objective.current_vars, &vars_indexes);
        let objective_offset = linearized_objective.current_rhs;
        Ok(LinearModel::new(
            objective_coeffs,
            objective_type,
            objective_offset,
            linear_constraints,
            vars,
        ))
    }
}

fn extract_coeffs(exp: &HashMap<String, f64>, vars: &HashMap<String, usize>) -> Vec<f64> {
    let mut vec = vec![0.0; vars.len()];
    for (name, val) in exp.iter() {
        let index = vars.get(name).unwrap();
        vec[*index] = *val;
    }
    vec
}

#[derive(Debug)]
pub enum LinearizationError {
    NonLinearExpression(Box<Exp>),
    VarAlreadyDeclared(String),
    UnimplementedExpression(Box<Exp>),
}
impl Display for LinearizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinearizationError::NonLinearExpression(exp) => {
                write!(f, "Non linear expression: \"{}\"", exp)
            }
            LinearizationError::VarAlreadyDeclared(name) => {
                write!(f, "Variable \"{}\" already declared", name)
            }
            LinearizationError::UnimplementedExpression(exp) => {
                write!(f, "Unimplemented expression: \"{}\"", exp)
            }
        }
    }
}
pub struct LinearizationContext {
    current_vars: HashMap<String, f64>,
    current_rhs: f64,
}

impl LinearizationContext {
    pub fn new() -> Self {
        LinearizationContext {
            current_vars: HashMap::new(),
            current_rhs: 0.0,
        }
    }
    pub fn from_var(name: String, multiplier: f64) -> Self {
        let mut context = LinearizationContext::new();
        context.add_var(name, multiplier);
        context
    }
    pub fn from_rhs(rhs: f64) -> Self {
        let mut context = LinearizationContext::new();
        context.add_rhs(rhs);
        context
    }
    pub fn add_var(&mut self, name: String, multiplier: f64) {
        if self.current_vars.contains_key(&name) {
            let val = self.current_vars.get_mut(&name).unwrap();
            *val += multiplier;
        } else {
            self.current_vars.insert(name, multiplier);
        }
    }

    pub fn merge_add(&mut self, other: LinearizationContext) {
        for (name, multiplier) in other.current_vars {
            self.add_var(name, multiplier);
        }
        self.add_rhs(other.current_rhs);
    }
    pub fn merge_sub(&mut self, other: LinearizationContext) {
        for (name, multiplier) in other.current_vars {
            self.add_var(name, -multiplier);
        }
        self.add_rhs(-other.current_rhs);
    }
    pub fn add_rhs(&mut self, rhs: f64) {
        self.current_rhs += rhs;
    }
    pub fn get_vars(&self) -> &HashMap<String, f64> {
        &self.current_vars
    }
    pub fn get_rhs(&self) -> f64 {
        self.current_rhs
    }
    pub fn has_var(&self, name: &String) -> bool {
        self.current_vars.contains_key(name)
    }
    pub fn mul_by(&mut self, multiplier: f64) {
        for (_, val) in self.current_vars.iter_mut() {
            *val *= multiplier;
        }
        self.current_rhs *= multiplier;
    }
    pub fn div_by(&mut self, divisor: f64) {
        for (_, val) in self.current_vars.iter_mut() {
            *val /= divisor;
        }
        self.current_rhs /= divisor;
    }

    pub fn has_no_vars(&self) -> bool {
        self.current_vars.is_empty()
    }
}
