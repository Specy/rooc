use core::fmt;
use std::collections::HashMap;
use std::hash::Hash;

use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::{primitives::primitive::Primitive, utils::Spanned};
use crate::math::math_enums::{Comparison, OptimizationType};
use crate::math::operators::{BinOp, UnOp};
use crate::parser::il::il_exp::PreExp;
use crate::parser::il::il_problem::{PreConstraint, PreObjective};
use crate::parser::model_transformer::transform_error::TransformError;
use crate::parser::model_transformer::transformer_context::{DomainVariable, TransformerContext};
use crate::parser::parser::PreModel;
use crate::parser::recursive_set_resolver::recursive_set_resolver;
use crate::traits::latex::{escape_latex, ToLatex};

#[derive(Debug, Clone, Serialize)]
pub enum Exp {
    Number(f64),
    Variable(String),
    Abs(Box<Exp>),
    Min(Vec<Exp>),
    Max(Vec<Exp>),
    BinOp(BinOp, Box<Exp>, Box<Exp>),
    UnOp(UnOp, Box<Exp>),
}

#[wasm_bindgen(typescript_custom_section)]
pub const IExp: &'static str = r#"
export type SerializedExp = {
    type: "Number",
    value: number
} | {
    type: "Variable",
    value: string
} | {
    type: "Abs",
    value: SerializedExp
} | {
    type: "Min",
    value: SerializedExp[]
} | {
    type: "Max",
    value: SerializedExp[]
} | {
    type: "BinOp",
    value: {
        op: BinOp,
        lhs: SerializedExp,
        rhs: SerializedExp
    }
} | {
    type: "UnOp",
    value: {
        op: UnOp,
        exp: SerializedExp
    }
}
"#;

impl Exp {
    pub fn make_binop(op: BinOp, lhs: Exp, rhs: Exp) -> Box<Self> {
        Exp::BinOp(op, lhs.to_box(), rhs.to_box()).to_box()
    }

    pub fn to_box(self) -> Box<Exp> {
        Box::new(self)
    }
    pub fn from_pre_exp(
        pre_exp: &PreExp,
        context: &mut TransformerContext,
    ) -> Result<Self, TransformError> {
        pre_exp.into_exp(context)
    }

    pub fn simplify(&self) -> Exp {
        //implement the simplify function by using e-graphs egg
        match self {
            Exp::BinOp(op, lhs, rhs) => {
                let lhs = lhs.simplify();
                let rhs = rhs.simplify();
                match (op, lhs, rhs) {
                    (op, Exp::Number(lhs), Exp::Number(rhs)) => match op {
                        BinOp::Add => Exp::Number(lhs + rhs),
                        BinOp::Sub => Exp::Number(lhs - rhs),
                        BinOp::Mul => Exp::Number(lhs * rhs),
                        BinOp::Div => Exp::Number(lhs / rhs),
                    },
                    (BinOp::Add, Exp::Number(0.0), rhs) => rhs,
                    (BinOp::Add, lhs, Exp::Number(0.0)) => lhs,
                    (BinOp::Sub, lhs, Exp::Number(0.0)) => lhs,
                    (BinOp::Mul, Exp::Number(0.0), _) => Exp::Number(0.0),
                    (BinOp::Mul, _, Exp::Number(0.0)) => Exp::Number(0.0),
                    (BinOp::Mul, Exp::Number(1.0), rhs) => rhs,
                    (BinOp::Mul, lhs, Exp::Number(1.0)) => lhs,
                    (BinOp::Div, lhs, Exp::Number(1.0)) => lhs,
                    //this would be an error, keep it as it is
                    (BinOp::Div, lhs, Exp::Number(0.0)) => {
                        Exp::BinOp(BinOp::Div, lhs.to_box(), Exp::Number(0.0).to_box())
                    }
                    (BinOp::Div, Exp::Number(0.0), _) => Exp::Number(0.0),
                    // num1 + num2 + x = (num1 + num2) + x
                    // num1 - num2 - x = (num1 - num2) - x
                    // num1 * num2 * x = (num1 * num2) * x
                    // num1 / num2 / x = (num1 / num2) / x
                    (op, Exp::Number(lhs), Exp::BinOp(op2, inner_lhs, inner_rhs)) => {
                        let inner_lhs = inner_lhs.simplify();
                        let inner_rhs = inner_rhs.simplify();
                        if *op != op2 {
                            return Exp::BinOp(
                                op.clone(),
                                Exp::Number(lhs).to_box(),
                                Exp::BinOp(op2, inner_lhs.to_box(), inner_rhs.to_box()).to_box(),
                            )
                            .simplify();
                        }
                        if let Exp::Number(rhs) = inner_lhs {
                            let val = match op {
                                BinOp::Add => lhs + rhs,
                                BinOp::Sub => lhs - rhs,
                                BinOp::Mul => lhs * rhs,
                                BinOp::Div => lhs / rhs,
                            };
                            Exp::BinOp(op2, Exp::Number(val).to_box(), inner_rhs.to_box())
                        } else {
                            Exp::BinOp(
                                *op,
                                Exp::Number(lhs).to_box(),
                                Exp::BinOp(op2, inner_lhs.to_box(), inner_rhs.to_box()).to_box(),
                            )
                        }
                    }
                    //keep the rest equal
                    (op, lhs, rhs) => Exp::BinOp(*op, lhs.to_box(), rhs.to_box()),
                }
            }
            Exp::UnOp(op, exp) => {
                let exp = exp.simplify();
                match op {
                    UnOp::Neg => match exp {
                        Exp::Number(value) => Exp::Number(-value),
                        _ => Exp::UnOp(UnOp::Neg, exp.to_box()),
                    },
                }
            }
            Exp::Max(exps) => {
                //if they are all numbers, return the max
                let nums = exps
                    .iter()
                    .map(|exp| {
                        let exp = exp.simplify();
                        if let Exp::Number(value) = exp {
                            Some(value)
                        } else {
                            None
                        }
                    })
                    .collect::<Option<Vec<f64>>>();
                match nums {
                    Some(nums) => {
                        Exp::Number(nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
                    }
                    None => Exp::Max(exps.iter().map(|exp| exp.simplify()).collect::<Vec<_>>()),
                }
            }
            Exp::Min(exps) => {
                //if they are all numbers, return the min
                let nums = exps
                    .iter()
                    .map(|exp| {
                        let exp = exp.simplify();
                        if let Exp::Number(value) = exp {
                            Some(value)
                        } else {
                            None
                        }
                    })
                    .collect::<Option<Vec<f64>>>();
                match nums {
                    Some(nums) => Exp::Number(nums.iter().cloned().fold(f64::INFINITY, f64::min)),
                    None => Exp::Min(exps.iter().map(|exp| exp.simplify()).collect::<Vec<_>>()),
                }
            }
            exp => exp.clone(),
        }
    }

    pub fn flatten(self) -> Exp {
        match self {
            Exp::BinOp(op, lhs, rhs) => match (op, *lhs, *rhs) {
                //(a +- b)c = ac +- bc
                (BinOp::Mul, Exp::BinOp(inner_op @ (BinOp::Add | BinOp::Sub), lhs, rhs), c) => {
                    Exp::BinOp(
                        inner_op,
                        Exp::make_binop(BinOp::Mul, *lhs, c.clone()),
                        Exp::make_binop(BinOp::Mul, *rhs, c),
                    )
                    .flatten()
                }
                //c(a +- b) = ac +- bc
                (BinOp::Mul, c, Exp::BinOp(inner_op @ (BinOp::Add | BinOp::Sub), lhs, rhs)) => {
                    Exp::BinOp(
                        inner_op,
                        Exp::make_binop(BinOp::Mul, c.clone(), *lhs),
                        Exp::make_binop(BinOp::Mul, c, *rhs),
                    )
                    .flatten()
                }
                //-(a)b = -ab
                (BinOp::Mul, Exp::UnOp(op @ UnOp::Neg, lhs), c) => {
                    Exp::UnOp(op, Exp::make_binop(BinOp::Mul, *lhs, c).flatten().to_box())
                }
                //a(-b) = -ab
                (BinOp::Mul, c, Exp::UnOp(op @ UnOp::Neg, rhs)) => {
                    Exp::UnOp(op, Exp::make_binop(BinOp::Mul, c, *rhs).flatten().to_box())
                }
                //(a +- b)/c = a/c +- b/c
                (BinOp::Div, Exp::BinOp(inner_op @ (BinOp::Add | BinOp::Sub), lhs, rhs), c) => {
                    Exp::BinOp(
                        inner_op,
                        Exp::make_binop(BinOp::Div, *lhs, c.clone())
                            .flatten()
                            .to_box(),
                        Exp::make_binop(BinOp::Div, *rhs, c).flatten().to_box(),
                    )
                }

                (op, lhs, rhs) => Exp::BinOp(op, lhs.flatten().to_box(), rhs.flatten().to_box()),
            },
            _ => self,
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, Exp::BinOp(_, _, _) | Exp::UnOp(_, _))
    }

    pub fn to_string_with_precedence(&self, last_operator: BinOp) -> String {
        let last_precedence = last_operator.precedence();
        match self {
            Exp::BinOp(op, lhs, rhs) => {
                let string_lhs = lhs.to_string_with_precedence(*op);
                let string_rhs = rhs.to_string_with_precedence(*op);
                let precedence = op.precedence();
                if precedence < last_precedence {
                    format!("({} {} {})", string_lhs, op, string_rhs)
                } else {
                    //TODO improve this
                    match (op, lhs.is_leaf(), rhs.is_leaf()){
                        (BinOp::Add, true, true) => format!("{} + {}", string_lhs, string_rhs),
                        (BinOp::Sub, true, true) => format!("{} - {}", string_lhs, string_rhs),
                        (BinOp::Mul, true, true) => format!("{}{}", string_lhs, string_rhs),
                        (BinOp::Div, true, true) => format!("{} / {}", string_lhs, string_rhs),
                        _ => format!("({} {} {})", string_lhs, op, string_rhs)
                    }
                }
            }
            _ => self.to_string(),
        }
    }
}

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Exp::Number(value) => value.to_string(),
            Exp::Variable(name) => name.clone(),
            Exp::Abs(exp) => format!("|{}|", exp),
            Exp::Min(exps) => format!(
                "min{{ {} }}",
                exps.iter()
                    .map(|exp| exp.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Exp::Max(exps) => format!(
                "max{{ {} }}",
                exps.iter()
                    .map(|exp| exp.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Exp::BinOp(operator, lhs, rhs) => {
                //TODO: add parenthesis when needed
                let string_lhs = lhs.to_string_with_precedence(*operator);
                let string_rhs = rhs.to_string_with_precedence(*operator);
                format!("{} - {}", string_lhs, string_rhs)
            }
            Exp::UnOp(op, exp) => {
                if exp.is_leaf() {
                    format!("{}{}", op, exp)
                } else {
                    format!("{}({})", op, exp)
                }
            }
        };
        f.write_str(&s)
    }
}

#[derive(Debug, Serialize)]
pub struct Objective {
    pub objective_type: OptimizationType,
    pub rhs: Exp,
}

#[wasm_bindgen(typescript_custom_section)]
pub const IObjective: &'static str = r#"
export type SerializedObjective = {
    objective_type: OptimizationType,
    rhs: SerializedExp
}
"#;

impl Objective {
    pub fn new(objective_type: OptimizationType, rhs: Exp) -> Self {
        Self {
            objective_type,
            rhs,
        }
    }
}

impl fmt::Display for Objective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.objective_type, self.rhs)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Constraint {
    lhs: Exp,
    constraint_type: Comparison,
    rhs: Exp,
}

#[wasm_bindgen(typescript_custom_section)]
pub const ICondition: &'static str = r#"
export type SerializedCondition = {
    lhs: SerializedExp,
    constraint_type: Comparison,
    rhs: SerializedExp
}
"#;

impl Constraint {
    pub fn new(lhs: Exp, constraint_type: Comparison, rhs: Exp) -> Self {
        Self {
            lhs,
            constraint_type,
            rhs,
        }
    }
    pub fn into_parts(self) -> (Exp, Comparison, Exp) {
        (self.lhs, self.constraint_type, self.rhs)
    }
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.lhs, self.constraint_type, self.rhs)
    }
}

#[derive(Debug, Serialize)]
#[wasm_bindgen]
pub struct Model {
    objective: Objective,
    constraints: Vec<Constraint>,
    domain: HashMap<String, DomainVariable>,
}

#[wasm_bindgen(typescript_custom_section)]
pub const IModel: &'static str = r#"
export type SerializedModel = {
    objective: Objective,
    constraints: SerializedCondition[]
    domain: Record<string, DomainVariable>
}
"#;

impl Model {
    pub fn new(
        objective: Objective,
        constraints: Vec<Constraint>,
        domain: HashMap<String, DomainVariable>,
    ) -> Self {
        Self {
            objective,
            constraints,
            domain,
        }
    }
    pub fn into_components(self) -> (Objective, Vec<Constraint>, HashMap<String, DomainVariable>) {
        (self.objective, self.constraints, self.domain)
    }
    pub fn get_objective(&self) -> &Objective {
        &self.objective
    }
    pub fn get_constraints(&self) -> &Vec<Constraint> {
        &self.constraints
    }
    pub fn get_domain(&self) -> &HashMap<String, DomainVariable> {
        &self.domain
    }
    pub fn get_domain_mut(&mut self) -> &mut HashMap<String, DomainVariable> {
        &mut self.domain
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let constraints = self
            .constraints
            .iter()
            .map(|constraint| constraint.to_string())
            .collect::<Vec<_>>()
            .join("\n    ");
        write!(f, "{}\ns.t.\n    {}", self.objective, constraints)
    }
}

#[wasm_bindgen]
impl Model {
    pub fn to_string_wasm(&self) -> String {
        self.to_string()
    }
    pub fn serialize_wasm(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}

pub fn transform_parsed_problem(pre_problem: PreModel) -> Result<Model, TransformError> {
    let context = TransformerContext::new_from_constants(
        pre_problem.get_constants().clone(),
        pre_problem.get_domains().clone(),
    )?;
    transform_model(pre_problem, context)
}

/*
this function gets a set, defined by a number of variables with a certain name, and an iterator,
it should return a vector of vectors, where each vector is a set of values for the variables
ex:
checks that the iterator has at least the same number of elements as the set, and then returns the values in the iterator
    in:  set {i, j} and iterator [[0, 0], [1, 1]]
    out: [[0, 0], [1, 1]]
    in:  set {i} and iterator [[0, 0], [1, 1]]
    out: [[0], [1]]
    in:  set {i, j, k} and iterator [[0, 0], [1, 1]]
    out: error!
*/

pub type PrimitiveSet = Vec<Primitive>;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum VariableKind {
    Single(Spanned<String>),
    Tuple(Vec<Spanned<String>>),
}

#[wasm_bindgen(typescript_custom_section)]
pub const IVariableType: &'static str = r#"
export type SerializedVariableType = {
    type: "Single",
    value: SerializedSpanned<string>
} | {
    type: "Tuple",
    value: SerializedSpanned<string>[]
}
"#;

impl ToLatex for VariableKind {
    fn to_latex(&self) -> String {
        match self {
            VariableKind::Single(name) => name.to_string(),
            VariableKind::Tuple(names) => format!(
                "({})",
                names
                    .iter()
                    .map(|name| escape_latex(name.get_span_value()))
                    .collect::<Vec<_>>()
                    .join(",\\ ")
            ),
        }
    }
}

impl fmt::Display for VariableKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VariableKind::Single(name) => f.write_str(name),
            VariableKind::Tuple(names) => write!(
                f,
                "({})",
                names
                    .iter()
                    .map(|name| name.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

pub fn transform_constraint(
    constraint: &PreConstraint,
    context: &mut TransformerContext,
) -> Result<Constraint, TransformError> {
    let lhs = constraint.lhs.into_exp(context)?;
    let rhs = constraint.rhs.into_exp(context)?;
    Ok(Constraint::new(lhs, constraint.constraint_type, rhs))
}

pub fn transform_constraint_with_iteration(
    constraint: &PreConstraint,
    context: &mut TransformerContext,
) -> Result<Vec<Constraint>, TransformError> {
    if constraint.iteration.is_empty() {
        return Ok(vec![transform_constraint(constraint, context)?]);
    }
    let mut results: Vec<Constraint> = Vec::new();
    recursive_set_resolver(&constraint.iteration, context, &mut results, 0, &|c| {
        transform_constraint(constraint, c)
    })
    .map_err(|e| e.add_span(&constraint.span))?;
    Ok(results)
}

pub fn transform_objective(
    objective: &PreObjective,
    context: &mut TransformerContext,
) -> Result<Objective, TransformError> {
    let rhs = objective.rhs.into_exp(context)?;
    Ok(Objective::new(objective.objective_type.clone(), rhs))
}

pub fn transform_model(
    problem: PreModel,
    mut context: TransformerContext,
) -> Result<Model, TransformError> {
    let objective = transform_objective(problem.get_objective(), &mut context)?;
    let mut constraints: Vec<Constraint> = Vec::new();
    for constraint in problem.get_constraints().iter() {
        let transformed = transform_constraint_with_iteration(constraint, &mut context)?;
        for transformed_constraint in transformed {
            constraints.push(transformed_constraint);
        }
    }
    let (domain) = context.into_components();
    Ok(Model::new(objective, constraints, domain))
}
