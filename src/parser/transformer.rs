use core::fmt;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use crate::math::math_enums::{Comparison, OptimizationType};
use crate::math::operators::{BinOp, UnOp};
use crate::primitives::consts::Constant;
use crate::primitives::functions::function_traits::FunctionCall;
use crate::primitives::primitive::PrimitiveKind;
use crate::{
    primitives::primitive::Primitive,
    utils::{InputSpan, Spanned},
};

use super::{
    parser::PreProblem,
    pre_parsed_problem::{AddressableAccess, PreCondition, PreExp, PreObjective},
    recursive_set_resolver::recursive_set_resolver,
};
use serde::{de::value, Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize)]
pub enum Exp {
    Number(f64),
    Variable(String),
    Mod(Box<Exp>),
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
    type: "Mod",
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
            Exp::Mod(exp) => format!("|{}|", exp),
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
pub struct Problem {
    objective: Objective,
    conditions: Vec<Condition>,
}
#[wasm_bindgen(typescript_custom_section)]
pub const IProblem: &'static str = r#"
export type SerializedProblem = {
    objective: Objective,
    conditions: SerializedCondition[]
}
"#;

impl Problem {
    pub fn new(objective: Objective, conditions: Vec<Condition>) -> Self {
        Self {
            objective,
            conditions,
        }
    }
}
impl fmt::Display for Problem {
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
impl Problem {
    pub fn to_string_wasm(&self) -> String {
        self.to_string()
    }
    pub fn serialize_wasm(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum TransformError {
    MissingVariable(String),
    AlreadyExistingVariable(String),
    OutOfBounds(String),
    WrongArgument {
        got: PrimitiveKind,
        expected: PrimitiveKind,
    },
    WrongExpectedArgument {
        got: PrimitiveKind,
        one_of: Vec<PrimitiveKind>,
    },
    SpannedError {
        spanned_error: Spanned<Box<TransformError>>,
        value: Option<String>,
    },
    WrongFunctionSignature {
        signature: Vec<PrimitiveKind>,
        got: Vec<PrimitiveKind>,
    },
    WrongNumberOfArguments {
        args: Vec<PreExp>,
        signature: Vec<PrimitiveKind>,
    },
    BinOpError {
        operator: BinOp,
        lhs: PrimitiveKind,
        rhs: PrimitiveKind,
    },
    UnOpError {
        operator: UnOp,
        exp: PrimitiveKind,
    },
    Unspreadable(PrimitiveKind),
    SpreadError {
        to_spread: PrimitiveKind,
        in_variables: Vec<String>,
    },
    Other(String),
}
#[wasm_bindgen(typescript_custom_section)]
pub const ITransformError: &'static str = r#"
export type SerializedTransformError = {
    type: "MissingVariable",
    value: string
} | {
    type: "AlreadyExistingVariable",
    value: string
} | {
    type: "OutOfBounds",
    value: string
} | {
    type: "WrongArgument",
    value: string
} | {
    type: "SpannedError",
    value: {
        spanned_error: SerializedSpanned<SerializedTransformError>,
        value?: string
    }
} | {
    type: "Unspreadable",
    value: string
} | {
    type: "Other",
    value: string
} | {
    type: "WrongNumberOfArguments",
    value: {
        signature: SerializedPrimitiveKind[],
        got: SerializedPrimitiveKind[]
    }
} | {
    type: "BinOpError",
    value: {
        operator: BinOp,
        lhs: SerializedPrimitiveKind,
        rhs: SerializedPrimitiveKind
    }
} | {
    type: "UnOpError",
    value: {
        operator: UnOp,
        exp: SerializedPrimitiveKind
    }
}
"#;
impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TransformError::MissingVariable(name) => format!("[Missing variable] {}", name),
            TransformError::AlreadyExistingVariable(name) => {
                format!(
                    "[AlreadyExistingVariable] Variable {} was already declared",
                    name
                )
            }
            TransformError::WrongFunctionSignature { signature, got } => {
                format!(
                    "[WrongFunctionSignature] Wrong number of arguments, expected \"{}\", got \"{}\"",
                    signature
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    got.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            TransformError::WrongNumberOfArguments {
                args,
                signature,
            } => {
                format!(
                    "[WrongNumberOfArguments] Wrong number of arguments, expected signature \"{}\", got parameters \"{}\"",
                    signature
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    args
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            TransformError::OutOfBounds(name) => format!("[OutOfBounds] {}", name),
            TransformError::WrongArgument { expected, got } => {
                format!(
                    "[WrongArgument] expected \"{}\", got \"{}\"",
                    expected.to_string(),
                    got.to_string()
                )
            }
            TransformError::WrongExpectedArgument { got, one_of } => {
                format!(
                    "[WrongExpectedArgument] expected one of \"{}\", got \"{}\"",
                    one_of
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    got.to_string()
                )
            }
            TransformError::SpreadError {
                to_spread,
                in_variables,
            } => format!(
                "[SpreadError] type \"{}\" cannot be spread in \"{}\"",
                to_spread.to_string(),
                in_variables
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            TransformError::Other(name) => format!("[Other] {}", name),
            TransformError::Unspreadable(kind) => format!(
                "[Unspreadable] type \"{}\" is not spreadable",
                kind.to_string()
            ),
            TransformError::SpannedError {
                spanned_error: span,
                ..
            } => span.to_string(),
            TransformError::BinOpError { operator, lhs, rhs } => {
                format!(
                    "[BinOpError] operator \"{}\" cannot be applied to \"{}\" and \"{}\"",
                    operator.to_string(),
                    lhs.to_string(),
                    rhs.to_string()
                )
            }
            TransformError::UnOpError { operator, exp } => {
                format!(
                    "[UnOpError] operator \"{}\" cannot be applied to \"{}\"",
                    operator.to_string(),
                    exp.to_string()
                )
            }
        };
        f.write_str(&s)
    }
}

impl TransformError {
    pub fn get_traced_error(&self) -> String {
        let error = self.to_string();
        let trace = self.get_trace();
        let trace = trace
            .iter()
            .map(|(span, origin)| {
                let origin = if let Some(o) = origin {
                    format!(" ({})", o)
                } else {
                    "".to_string()
                };
                format!("\tat {}:{}{}", span.start_line, span.start_column, origin)
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("{}\n{}", error, trace)
    }
    pub fn from_wrong_type(expected: PrimitiveKind, got: PrimitiveKind, span: InputSpan) -> Self {
        TransformError::WrongArgument { got, expected }.to_spanned_error(&span)
    }
    pub fn from_wrong_binop(
        operator: BinOp,
        lhs: PrimitiveKind,
        rhs: PrimitiveKind,
        span: InputSpan,
    ) -> Self {
        TransformError::BinOpError { operator, lhs, rhs }.to_spanned_error(&span)
    }
    pub fn from_wrong_unop(operator: UnOp, exp: PrimitiveKind, span: InputSpan) -> Self {
        TransformError::UnOpError { operator, exp }.to_spanned_error(&span)
    }
    pub fn to_spanned_error(self, span: &InputSpan) -> TransformError {
        TransformError::SpannedError {
            spanned_error: Spanned::new(Box::new(self), span.clone()),
            value: None,
        }
    }
    pub fn get_trace(&self) -> Vec<(InputSpan, Option<String>)> {
        match self {
            TransformError::SpannedError {
                spanned_error: span,
                value,
            } => {
                let mut trace = vec![(span.get_span().clone(), value.clone())];
                let mut last_error = span;
                while let TransformError::SpannedError {
                    spanned_error: ref span,
                    ref value,
                } = **last_error.get_span_value()
                {
                    let current_span = span.get_span().clone();
                    //don't add if the last span is the same as the current one
                    if let Some((last_span, _)) = trace.last() {
                        if last_span == &current_span {
                            last_error = span;
                            continue;
                        }
                    }
                    trace.push((current_span, value.clone()));
                    last_error = span;
                }
                trace.reverse();
                trace
            }
            _ => Vec::new(),
        }
    }
    pub fn get_origin_span(&self) -> Option<InputSpan> {
        let trace = self.get_trace();
        match trace.first() {
            Some((span, _)) => Some(span.clone()),
            None => None,
        }
    }
    pub fn get_base_error(&self) -> &TransformError {
        match self {
            TransformError::SpannedError {
                spanned_error: span,
                ..
            } => span.get_base_error(),
            _ => self,
        }
    }
    pub fn get_trace_from_source(&self, source: &str) -> Result<String, String> {
        let trace = self.get_trace();
        let trace = trace
            .into_iter()
            .map(|(span, _)| {
                let text = span.get_span_text(source)?;
                Ok(format!(
                    "at {}:{} {}",
                    span.start_line, span.start_column, text,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let join = trace.join("\n\t");
        Ok(format!("{}\n\t{}", self, join))
    }
}

#[derive(Debug)]
pub struct Frame<T> {
    pub variables: HashMap<String, T>,
}
impl<T> Frame<T> {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
    pub fn from_map(constants: HashMap<String, T>) -> Self {
        Self {
            variables: constants,
        }
    }

    pub fn get_value(&self, name: &str) -> Option<&T> {
        self.variables.get(name)
    }
    pub fn declare_variable(&mut self, name: &str, value: T) -> Result<(), TransformError> {
        if self.has_variable(name) {
            return Err(TransformError::AlreadyExistingVariable(name.to_string()));
        }
        self.variables.insert(name.to_string(), value);
        Ok(())
    }
    pub fn update_variable(&mut self, name: &str, value: T) -> Result<(), TransformError> {
        if !self.has_variable(name) {
            return Err(TransformError::MissingVariable(name.to_string()));
        }
        self.variables.insert(name.to_string(), value);
        Ok(())
    }
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }
    pub fn drop_variable(&mut self, name: &str) -> Result<T, TransformError> {
        if !self.variables.contains_key(name) {
            return Err(TransformError::MissingVariable(name.to_string()));
        }
        let value = self.variables.remove(name).unwrap();
        Ok(value)
    }
}
impl<T> Default for Frame<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct TransformerContext {
    frames: Vec<Frame<Primitive>>,
}
impl TransformerContext {
    pub fn new(primitives: HashMap<String, Primitive>) -> Self {
        let frame = Frame::from_map(primitives);
        Self {
            frames: vec![frame],
        }
    }
    pub fn new_from_constants(constants: Vec<Constant>) -> Self {
        let primitives = constants
            .into_iter()
            .map(|c| (c.name.into_span_value(), c.value))
            .collect::<HashMap<_, _>>();
        Self::new(primitives)
    }

    pub fn flatten_variable_name(
        &self,
        compound_name: &[String],
    ) -> Result<String, TransformError> {
        let flattened = compound_name
            .iter()
            .map(|name| match self.get_value(name) {
                Some(value) => match value {
                    Primitive::Number(value) => Ok(value.to_string()),
                    Primitive::String(value) => Ok(value.clone()),
                    Primitive::GraphNode(v) => Ok(v.get_name().clone()),
                    _ => Err(TransformError::WrongExpectedArgument {
                        got: value.get_type(),
                        one_of: vec![
                            PrimitiveKind::Number,
                            PrimitiveKind::String,
                            PrimitiveKind::GraphNode,
                        ],
                    }),
                },
                None => Err(TransformError::MissingVariable(name.to_string())),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(flattened.join("_"))
    }

    pub fn add_populated_scope(&mut self, frame: Frame<Primitive>) {
        self.frames.push(frame);
    }
    pub fn replace_last_frame(&mut self, frame: Frame<Primitive>) {
        self.frames.pop();
        self.frames.push(frame);
    }
    pub fn add_scope(&mut self) {
        let frame = Frame::new();
        self.frames.push(frame);
    }
    pub fn pop_scope(&mut self) -> Result<Frame<Primitive>, TransformError> {
        if self.frames.len() <= 1 {
            return Err(TransformError::Other("Missing frame to pop".to_string()));
        }
        Ok(self.frames.pop().unwrap())
    }
    pub fn get_value(&self, name: &str) -> Option<&Primitive> {
        for frame in self.frames.iter().rev() {
            match frame.get_value(name) {
                Some(value) => return Some(value),
                None => continue,
            }
        }
        None
    }
    pub fn exists_variable(&self, name: &str, strict: bool) -> bool {
        if strict {
            for frame in self.frames.iter().rev() {
                if frame.has_variable(name) {
                    return true;
                }
            }
        } else {
            return match self.frames.last() {
                Some(frame) => frame.has_variable(name),
                None => false,
            };
        }
        false
    }
    pub fn declare_variable(
        &mut self,
        name: &str,
        value: Primitive,
        strict: bool,
    ) -> Result<(), TransformError> {
        if name == "_" {
            return Ok(());
        }
        if strict && self.get_value(name).is_some() {
            return Err(TransformError::AlreadyExistingVariable(name.to_string()));
        }
        let frame = self.frames.last_mut().unwrap();
        frame.declare_variable(name, value)
    }
    pub fn update_variable(&mut self, name: &str, value: Primitive) -> Result<(), TransformError> {
        if name == "_" {
            return Ok(());
        }
        for frame in self.frames.iter_mut().rev() {
            if frame.has_variable(name) {
                return frame.update_variable(name, value);
            }
        }
        Err(TransformError::MissingVariable(name.to_string()))
    }
    pub fn remove_variable(&mut self, name: &str) -> Result<Primitive, TransformError> {
        if name == "_" {
            return Ok(Primitive::Undefined);
        }
        for frame in self.frames.iter_mut().rev() {
            if frame.has_variable(name) {
                return frame.drop_variable(name);
            }
        }
        Err(TransformError::MissingVariable(name.to_string()))
    }

    pub fn flatten_compound_variable(
        &self,
        name: &String,
        indexes: &[String],
    ) -> Result<String, TransformError> {
        let names: String = self.flatten_variable_name(indexes)?;
        let name = format!("{}_{}", name, names);
        Ok(name)
    }

    pub fn get_numerical_constant(&self, name: &str) -> Result<f64, TransformError> {
        match self.get_value(name) {
            Some(v) => v.as_number(),
            None => Err(TransformError::MissingVariable(name.to_string())),
        }
    }
    pub fn get_addressable_value(
        &self,
        addressable_access: &AddressableAccess,
    ) -> Result<Primitive, TransformError> {
        //TODO add support for object access like G["a"] or g.a
        match self.get_value(&addressable_access.name) {
            Some(a) => {
                let accesses = addressable_access
                    .accesses
                    .iter()
                    .map(|access| access.as_usize(self))
                    .collect::<Result<Vec<_>, TransformError>>()?;
                let value = a.as_iterator()?.read(accesses)?;
                Ok(value)
            }
            None => Err(TransformError::MissingVariable(
                addressable_access.name.to_string(),
            )),
        }
    }
}

pub fn transform_parsed_problem(pre_problem: PreProblem) -> Result<Problem, TransformError> {
    let mut context = TransformerContext::new_from_constants(pre_problem.get_constants().clone());
    transform_problem(&pre_problem, &mut context)
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
pub enum VariableType {
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

impl VariableType {
    pub fn to_string(&self) -> String {
        match self {
            VariableType::Single(name) => name.to_string(),
            VariableType::Tuple(names) => format!(
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
    .map_err(|e| e.to_spanned_error(&condition.span))?;
    Ok(results)
}

pub fn transform_objective(
    objective: &PreObjective,
    context: &mut TransformerContext,
) -> Result<Objective, TransformError> {
    let rhs = objective.rhs.into_exp(context)?;
    Ok(Objective::new(objective.objective_type.clone(), rhs))
}

pub fn transform_problem(
    problem: &PreProblem,
    context: &mut TransformerContext,
) -> Result<Problem, TransformError> {
    let objective = transform_objective(problem.get_objective(), context)?;
    let mut conditions: Vec<Condition> = Vec::new();
    for condition in problem.get_conditions().iter() {
        let transformed = transform_condition_with_iteration(condition, context)?;
        for condition in transformed {
            conditions.push(condition);
        }
    }
    Ok(Problem::new(objective, conditions))
}
