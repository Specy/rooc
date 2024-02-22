use core::fmt;
use std::collections::HashMap;
use std::hash::Hash;

use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::{primitives::primitive::Primitive, utils::Spanned};
use crate::math::math_enums::{Comparison, OptimizationType};
use crate::math::operators::{BinOp, UnOp};
use crate::parser::il::il_exp::PreExp;
use crate::parser::il::il_problem::{PreCondition, PreObjective};
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
        todo!("implement the simplify function by using e-graphs egg")
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

    pub fn to_string_with_precedence(&self, last_precedence: u8) -> String {
        match self {
            Exp::BinOp(op, lhs, rhs) => {
                let lhs = lhs.to_string_with_precedence(op.precedence());
                let rhs = rhs.to_string_with_precedence(op.precedence());
                let precedence = op.precedence();
                if precedence < last_precedence {
                    format!("({} {} {})", lhs, op, rhs)
                } else {
                    format!("{} {} {}", lhs, op, rhs)
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
                format!(
                    "{} {} {}",
                    lhs.to_string_with_precedence(operator.precedence()),
                    operator,
                    rhs.to_string_with_precedence(operator.precedence())
                )
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
#[wasm_bindgen]
pub struct Objective {
    objective_type: OptimizationType,
    rhs: Exp,
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
pub struct Condition {
    lhs: Exp,
    condition_type: Comparison,
    rhs: Exp,
}

#[wasm_bindgen(typescript_custom_section)]
pub const ICondition: &'static str = r#"
export type SerializedCondition = {
    lhs: SerializedExp,
    condition_type: Comparison,
    rhs: SerializedExp
}
"#;

impl Condition {
    pub fn new(lhs: Exp, condition_type: Comparison, rhs: Exp) -> Self {
        Self {
            lhs,
            condition_type,
            rhs,
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.lhs, self.condition_type, self.rhs)
    }
}

#[derive(Debug, Serialize)]
#[wasm_bindgen]
pub struct Model {
    objective: Objective,
    conditions: Vec<Condition>,
    domain: HashMap<String, DomainVariable>,
}

#[wasm_bindgen(typescript_custom_section)]
pub const IModel: &'static str = r#"
export type SerializedModel = {
    objective: Objective,
    conditions: SerializedCondition[]
    domain: Record<string, DomainVariable>
}
"#;

impl Model {
    pub fn new(
        objective: Objective,
        conditions: Vec<Condition>,
        domain: HashMap<String, DomainVariable>,
    ) -> Self {
        Self {
            objective,
            conditions,
            domain,
        }
    }
    pub fn into_components(self) -> (Objective, Vec<Condition>, HashMap<String, DomainVariable>) {
        (self.objective, self.conditions, self.domain)
    }
    pub fn get_objective(&self) -> &Objective {
        &self.objective
    }
    pub fn get_conditions(&self) -> &Vec<Condition> {
        &self.conditions
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
        let conditions = self
            .conditions
            .iter()
            .map(|condition| condition.to_string())
            .collect::<Vec<_>>()
            .join("\n    ");
        write!(f, "{}\ns.t.\n    {}", self.objective, conditions)
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

pub fn transform_condition(
    condition: &PreCondition,
    context: &mut TransformerContext,
) -> Result<Condition, TransformError> {
    let lhs = condition.lhs.into_exp(context)?;
    let rhs = condition.rhs.into_exp(context)?;
    Ok(Condition::new(lhs, condition.condition_type, rhs))
}

pub fn transform_condition_with_iteration(
    condition: &PreCondition,
    context: &mut TransformerContext,
) -> Result<Vec<Condition>, TransformError> {
    if condition.iteration.is_empty() {
        return Ok(vec![transform_condition(condition, context)?]);
    }
    let mut results: Vec<Condition> = Vec::new();
    recursive_set_resolver(&condition.iteration, context, &mut results, 0, &|c| {
        transform_condition(condition, c)
    })
        .map_err(|e| e.add_span(&condition.span))?;
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
    let mut conditions: Vec<Condition> = Vec::new();
    for condition in problem.get_conditions().iter() {
        let transformed = transform_condition_with_iteration(condition, &mut context)?;
        for condition in transformed {
            conditions.push(condition);
        }
    }
    let (domain) = context.into_components();
    Ok(Model::new(objective, conditions, domain))
}
