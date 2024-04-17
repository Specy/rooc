use std::collections::HashMap;

use crate::math::math_enums::Comparison;
use crate::math::operators::{BinOp, UnOp};
use crate::parser::model_transformer::model::{Constraint, Exp, Model};
use crate::parser::model_transformer::transformer_context::DomainVariable;
use crate::transformers::linear_model::LinearModel;

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
      x1 - y>= -b
 */

struct LinearVariable {
    name: String,
    multiplier: f64,
}

pub struct LinearizerContext {
    constraints: Vec<Constraint>,
    transformed_constraints: Vec<MidLinearizedConstraint>,
    current_context: MidLinearizedConstraint,
    surplus_count: u32,
    slack_count: u32,
    min_count: u32,
    max_count: u32,
    domain: HashMap<String, DomainVariable>,
}

#[derive(Debug, Clone)]
pub struct MidLinearizedConstraint {
    lhs: HashMap<String, f64>,
    rhs: f64,
    comparison: Comparison,
}
impl MidLinearizedConstraint {
    pub fn new(lhs: HashMap<String, f64>, rhs: f64, comparison: Comparison) -> Self {
        MidLinearizedConstraint {
            lhs,
            rhs,
            comparison,
        }
    }

    pub fn increment_coeff_by(&mut self, amount: f64, name: &String) -> f64 {
        if self.lhs.contains_key(name) {
            let val = self.lhs.get_mut(name).unwrap();
            *val += amount;
            *val
        } else {
            self.lhs.insert(name.clone(), amount);
            amount
        }
    }
    pub fn increment_rhs_by(&mut self, amount: f64) -> f64 {
        self.rhs += amount;
        self.rhs
    }
}
impl Default for MidLinearizedConstraint {
    fn default() -> Self {
        MidLinearizedConstraint {
            lhs: HashMap::new(),
            rhs: 0.0,
            comparison: Comparison::Equal,
        }
    }
}

impl Default for LinearizerContext {
    fn default() -> Self {
        LinearizerContext {
            constraints: Vec::new(),
            transformed_constraints: Vec::new(),
            current_context: MidLinearizedConstraint::new(HashMap::new(), 0.0, Comparison::Equal),
            surplus_count: 0,
            slack_count: 0,
            min_count: 0,
            max_count: 0,
            domain: HashMap::new(),
        }
    }
}
impl LinearizerContext {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_from(constraints: Vec<Constraint>, domain: HashMap<String, DomainVariable>) -> Self {
        let mut context = Self::default();
        context.constraints = constraints;
        context.domain = domain;
        context
    }
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
    pub fn create_new_context(&mut self, comparison: Comparison) {
        self.current_context = MidLinearizedConstraint::default();
        self.current_context.comparison = comparison;
    }
    pub fn save_context_as_constraint(&mut self) {
        self.transformed_constraints
            .push(self.current_context.clone());
    }
    pub fn get_constraints(&self) -> &Vec<Constraint> {
        &self.constraints
    }
    pub fn pop_constraint(&mut self) -> Option<Constraint> {
        self.constraints.pop()
    }
}

pub struct Linearizer {}

enum LinearExp {
    Variable { name: String, multiplier: f64 },
    Constant(f64),
    
}

impl Exp {
    fn linearize(self, linearizer_context: &mut LinearizerContext) {
        match self {
            Exp::Abs(exp) => {
                todo!("Implement Abs linearization")
            }
            Exp::BinOp(op, lhs, rhs) => match (*lhs, op, *rhs) {
                (Exp::Number(num), op, Exp::Variable(name)) => match op {
                    BinOp::Mul => {
                        linearizer_context
                            .current_context
                            .increment_coeff_by(num, &name);
                    }
                    BinOp::Div => {
                        linearizer_context
                            .current_context
                            .increment_coeff_by(1.0 / num, &name);
                    }
                    BinOp::Add => {
                        linearizer_context
                            .current_context
                            .increment_coeff_by(1.0, &name);
                        linearizer_context.current_context.increment_rhs_by(num);
                    }
                    BinOp::Sub => {
                        linearizer_context
                            .current_context
                            .increment_coeff_by(1.0, &name);
                        linearizer_context.current_context.increment_rhs_by(-num);
                    }
                },
            },
            Exp::UnOp(op, exp) => match op {
                UnOp::Neg => {
                    todo!()
                }
            },
            Exp::Number(num) => {
                linearizer_context.current_context.increment_rhs_by(num);
            }
            Exp::Variable(name) => {
                linearizer_context
                    .current_context
                    .increment_coeff_by(1.0, &name);
            }
            Exp::Min(exps) => {
                let var_name = format!("min_{}", linearizer_context.min_count);
                linearizer_context.min_count += 1;
                for exp in exps {
                    let constraint = Constraint::new(
                        Exp::Variable(var_name.clone()).clone(),
                        Comparison::LowerOrEqual,
                        exp.clone(),
                    );
                    linearizer_context.add_constraint(constraint)
                }
            }
            Exp::Max(exps) => {
                let var_name = format!("max_{}", linearizer_context.max_count);
                linearizer_context.max_count += 1;
                for exp in exps {
                    let constraint = Constraint::new(
                        Exp::Variable(var_name.clone()).clone(),
                        Comparison::UpperOrEqual,
                        exp.clone(),
                    );
                    linearizer_context.add_constraint(constraint)
                }
            }
        }
    }
}
pub fn linearize_model(model: Model) -> LinearModel {
    let (objective, constraints, domain) = model.into_components();
    let mut context = LinearizerContext::new_from(constraints, domain);
    while let Some(constraint) = context.pop_constraint() {
        let (lhs, op, rhs) = constraint.into_parts();
        let mut exp = Exp::BinOp(BinOp::Sub, Box::new(lhs), Box::new(rhs))
            .flatten()
            .simplify();
        context.create_new_context(op);
        exp.linearize(&mut context);
        context.save_context_as_constraint();
    }
    todo!()
}
