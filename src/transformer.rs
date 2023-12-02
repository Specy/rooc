use crate::{
    consts::{Comparison, ConstantValue, InputSpan, IterableKind, Op, OptimizationType, Primitive},
    parser::{PreArrayAccess, PreCondition, PreExp, PreIterator, PreObjective, PreProblem, PreSet},
};
use egg::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Exp {
    Number(f64),
    Variable(String),
    Mod(Box<Exp>),
    Min(Vec<Exp>),
    Max(Vec<Exp>),
    BinOp(Op, Box<Exp>, Box<Exp>),
    Neg(Box<Exp>),
}

impl Exp {
    pub fn make_binop(op: Op, lhs: Exp, rhs: Exp) -> Box<Self> {
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
                (Op::Mul, Exp::BinOp(inner_op @ (Op::Add | Op::Sub), lhs, rhs), c) => Exp::BinOp(
                    inner_op,
                    Exp::make_binop(Op::Mul, *lhs, c.clone()),
                    Exp::make_binop(Op::Mul, *rhs, c),
                )
                .flatten(),
                //c(a +- b) = ac +- bc
                (Op::Mul, c, Exp::BinOp(inner_op @ (Op::Add | Op::Sub), lhs, rhs)) => Exp::BinOp(
                    inner_op,
                    Exp::make_binop(Op::Mul, c.clone(), *lhs),
                    Exp::make_binop(Op::Mul, c, *rhs),
                )
                .flatten(),
                //-(a)b = -ab
                (Op::Mul, Exp::Neg(lhs), c) => {
                    Exp::Neg(Exp::make_binop(Op::Mul, *lhs, c).flatten().to_box())
                }
                //a(-b) = -ab
                (Op::Mul, c, Exp::Neg(rhs)) => {
                    Exp::Neg(Exp::make_binop(Op::Mul, c, *rhs).flatten().to_box())
                }
                //(a +- b)/c = a/c +- b/c
                (Op::Div, Exp::BinOp(inner_op @ (Op::Add | Op::Sub), lhs, rhs), c) => Exp::BinOp(
                    inner_op,
                    Exp::make_binop(Op::Div, *lhs, c.clone()),
                    Exp::make_binop(Op::Div, *rhs, c),
                ),

                (op, lhs, rhs) => Exp::BinOp(op, lhs.flatten().to_box(), rhs.flatten().to_box()),
            },
            _ => self,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Exp::Number(value) => value.to_string(),
            Exp::Variable(name) => name.clone(),
            Exp::Mod(exp) => format!("|{}|", exp.to_string()),
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
            Exp::BinOp(operator, lhs, rhs) => format!(
                "({} {} {})",
                lhs.to_string(),
                operator.to_string(),
                rhs.to_string(),
            ),
            //Exp::Parenthesis(exp) => format!("({})", exp.to_string()),
            Exp::Neg(exp) => format!("-{}", exp.to_string()),
        }
    }
    pub fn remove_root_parenthesis(&self) -> &Exp {
        match self {
            //Exp::Parenthesis(exp) => exp,
            _ => self,
        }
    }
}

#[derive(Debug)]
pub struct Objective {
    objective_type: OptimizationType,
    rhs: Exp,
}

impl Objective {
    pub fn new(objective_type: OptimizationType, rhs: &Exp) -> Self {
        Self {
            objective_type,
            rhs: rhs.remove_root_parenthesis().clone(),
        }
    }
    pub fn to_string(&self) -> String {
        format!(
            "{} {}",
            self.objective_type.to_string(),
            self.rhs.to_string()
        )
    }
}

#[derive(Debug, Clone)]
pub struct Condition {
    lhs: Exp,
    condition_type: Comparison,
    rhs: Exp,
}

impl Condition {
    pub fn new(lhs: &Exp, condition_type: Comparison, rhs: &Exp) -> Self {
        Self {
            lhs: lhs.remove_root_parenthesis().clone(),
            condition_type,
            rhs: rhs.remove_root_parenthesis().clone(),
        }
    }
    pub fn to_string(&self) -> String {
        format!(
            "{} {} {}",
            self.lhs.to_string(),
            self.condition_type.to_string(),
            self.rhs.to_string()
        )
    }
}

#[derive(Debug)]
pub struct Problem {
    objective: Objective,
    conditions: Vec<Condition>,
}

impl Problem {
    pub fn new(objective: Objective, conditions: Vec<Condition>) -> Self {
        Self {
            objective,
            conditions,
        }
    }
    pub fn to_string(&self) -> String {
        let conditions = self
            .conditions
            .iter()
            .map(|condition| condition.to_string())
            .collect::<Vec<_>>()
            .join("\n\t");
        format!("{}\ns.t\n\t{}", self.objective.to_string(), conditions)
    }
}

#[derive(Debug)]
pub enum TransformError {
    MissingVariable(String),
    AlreadyExistingVariable(String),
    OutOfBounds(String),
    WrongArgument(String),
    //TODO not sure if this is the best way to handle this, but it makes it easier to propagate the span
    SpannedError(Box<TransformError>, InputSpan),
    Other(String),
}
impl TransformError {
    pub fn to_string(&self) -> String {
        match self {
            TransformError::MissingVariable(name) => format!("Missing variable {}", name),
            TransformError::AlreadyExistingVariable(name) => {
                format!("Already existing variable {}", name)
            }
            TransformError::OutOfBounds(name) => format!("Out of bounds {}", name),
            TransformError::WrongArgument(name) => format!("Wrong argument {}", name),
            TransformError::Other(name) => name.clone(),
            TransformError::SpannedError(error, span) => {
                format!(
                    "{} at line: {} col: {}",
                    error.to_string(),
                    span.start_line,
                    span.start_column
                )
            }
        }
    }
    pub fn to_spanned_error(self, span: &InputSpan) -> TransformError {
        TransformError::SpannedError(Box::new(self), span.clone())
    }
}

#[derive(Debug)]
pub struct Frame<'a> {
    pub variables: HashMap<String, Primitive<'a>>,
}
impl<'a> Frame<'a> {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
    pub fn from_constants(constants: HashMap<String, ConstantValue>) -> Self {
        let variables = constants
            .into_iter()
            .map(|(name, value)| (name.clone(), Primitive::from_constant_value(value)))
            .collect::<HashMap<_, _>>();
        Self { variables }
    }

    pub fn get_value(&self, name: &str) -> Option<&Primitive> {
        self.variables.get(name)
    }
    pub fn declare_variable(
        &mut self,
        name: &str,
        value: Primitive<'a>,
    ) -> Result<(), TransformError> {
        if self.has_variable(name) {
            return Err(TransformError::AlreadyExistingVariable(name.to_string()));
        }
        self.variables.insert(name.to_string(), value);
        Ok(())
    }
    pub fn update_variable(
        &mut self,
        name: &str,
        value: Primitive<'a>,
    ) -> Result<(), TransformError> {
        if !self.has_variable(name) {
            return Err(TransformError::MissingVariable(name.to_string()));
        }
        self.variables.insert(name.to_string(), value);
        Ok(())
    }
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }
    pub fn drop_variable(&mut self, name: &str) -> Result<Primitive<'a>, TransformError> {
        if !self.variables.contains_key(name) {
            return Err(TransformError::MissingVariable(name.to_string()));
        }
        let value = self.variables.remove(name).unwrap();
        Ok(value)
    }
}

#[derive(Debug)]
pub struct TransformerContext<'a> {
    frames: Vec<Frame<'a>>,
}
impl<'a> TransformerContext<'a> {
    pub fn new(constants: HashMap<String, ConstantValue>) -> Self {
        let frame = Frame::from_constants(constants);
        Self {
            frames: vec![frame],
        }
    }

    pub fn flatten_variable_name(
        &self,
        compound_name: &Vec<String>,
    ) -> Result<String, TransformError> {
        let flattened = compound_name
            .iter()
            .map(|name| match self.get_value(name) {
                Some(value) => match value {
                    Primitive::Number(value) => Ok(value.to_string()),
                    Primitive::String(value) => Ok(value.clone()),
                    _ => Err(TransformError::WrongArgument(format!(
                        "Expected a number or string for flattening, check the definition of {}",
                        name
                    ))),
                },
                None => Err(TransformError::MissingVariable(name.to_string())),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(flattened.join("_"))
    }

    pub fn add_scope(&mut self) {
        let frame = Frame::new();
        self.frames.push(frame);
    }
    pub fn pop_scope(&mut self) -> Result<(), TransformError> {
        if self.frames.len() == 1 {
            return Err(TransformError::Other("Missing frame to pop".to_string()));
        }
        self.frames.pop();
        Ok(())
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
        value: Primitive<'a>,
        strict: bool,
    ) -> Result<(), TransformError> {
        if strict {
            if self.get_value(name).is_some() {
                return Err(TransformError::AlreadyExistingVariable(name.to_string()));
            }
        }
        let frame = self.frames.last_mut().unwrap();
        frame.declare_variable(name, value)
    }
    pub fn update_variable(&mut self, name: &str, value: Primitive<'a>) -> Result<(), TransformError> {
        for frame in self.frames.iter_mut().rev() {
            if frame.has_variable(name) {
                return frame.update_variable(name, value);
            }
        }
        Err(TransformError::MissingVariable(name.to_string()))
    }
    pub fn remove_variable(&mut self, name: &str) -> Result<Primitive, TransformError> {
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
        indexes: &Vec<String>,
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
    pub fn get_1d_array_number_value(&self, name: &str, i: usize) -> Result<f64, TransformError> {
        match self.get_value(name) {
            Some(a) => {
                let value = a.as_number_array()?.get(i).map(|v| *v);
                match value {
                    Some(value) => Ok(value),
                    None => Err(TransformError::OutOfBounds(format!("{}[{}]", name, i))),
                }
            }
            None => Err(TransformError::MissingVariable(name.to_string())),
        }
    }
    pub fn get_2d_array_number_value(
        &self,
        name: &str,
        i: usize,
        j: usize,
    ) -> Result<f64, TransformError> {
        match self.get_value(name) {
            Some(a) => {
                let value = a
                    .as_number_matrix()?
                    .get(i)
                    .map(|row| row.get(j).map(|v| *v));
                match value {
                    Some(Some(value)) => Ok(value),
                    Some(None) => Err(TransformError::OutOfBounds(format!(
                        "{}[{}][{}]",
                        name, i, j
                    ))),
                    None => Err(TransformError::OutOfBounds(format!(
                        "{}[{}][{}]",
                        name, i, j
                    ))),
                }
            }
            None => Err(TransformError::MissingVariable(name.to_string())),
        }
    }
    pub fn get_1d_array_length(&self, name: &str) -> Result<usize, TransformError> {
        match self.get_value(name) {
            Some(a) => {
                let value = a.as_number_array().map(|a| a.len())?;
                Ok(value)
            }
            None => Err(TransformError::MissingVariable(name.to_string())),
        }
    }
    pub fn get_2d_array_length(
        &self,
        name: &str,
        index: usize,
    ) -> Result<(usize, usize), TransformError> {
        match self.get_value(name) {
            Some(a) => {
                let value = a.as_number_matrix().map(|a| {
                    let row = a.get(index).map(|row| row.len());
                    match row {
                        Some(row) => (a.len(), row),
                        None => (a.len(), 0),
                    }
                })?;
                Ok(value)
            }
            None => Err(TransformError::MissingVariable(name.to_string())),
        }
    }

    pub fn get_array_access_value(
        &self,
        array_access: &PreArrayAccess,
    ) -> Result<f64, TransformError> {
        let indexes = array_access
            .accesses
            .iter()
            .map(|access| access.as_usize(self))
            .collect::<Result<Vec<_>, TransformError>>()?;
        match indexes.as_slice() {
            [i] => Ok(self.get_1d_array_number_value(&array_access.name, *i)?),
            [i, j] => Ok(self.get_2d_array_number_value(&array_access.name, *i, *j)?),
            _ => Err(TransformError::OutOfBounds(format!(
                "limit of 2d arrays, trying to access {}{}",
                array_access.name,
                indexes
                    .iter()
                    .map(|i| format!("[{}]", i))
                    .collect::<Vec<_>>()
                    .join("")
            ))),
        }
    }
}

pub fn transform(pre_problem: &PreProblem) -> Result<Problem, TransformError> {
    let constants = pre_problem
        .constants
        .iter()
        .map(|c| (c.name.clone(), c.value.clone()))
        .collect::<Vec<_>>();
    let constants = HashMap::from_iter(constants);
    let mut context = TransformerContext::new(constants);
    transform_problem(pre_problem, &mut context)
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

pub type PrimitiveSet<'a> = Vec<Primitive<'a>>;

pub struct NamedSet<'a> {
    pub vars: Vec<String>,
    pub set: PrimitiveSet<'a>,
}
impl<'a> NamedSet<'a> {
    pub fn to_string(&self) -> String {
        let variables = self.vars.join(", ");
        let set = self
            .set
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("set {{{}}} = {{{}}}", variables, set)
    }
    pub fn new(variables: Vec<String>, set: PrimitiveSet<'a>) -> Self {
        Self {
            vars: variables,
            set,
        }
    }
}

pub fn transform_set<'a>(
    range: &PreSet,
    context: &'a TransformerContext,
) -> Result<NamedSet<'a>, TransformError> {
    let set = match &range.iterator {
        PreIterator::Range {
            from,
            to,
            to_inclusive,
        } => {
            if range.vars_tuple.len() != 1 {
                return Err(TransformError::WrongArgument(
                    "Expected 1 variable in range".to_string(),
                ));
            }
            let from = from.to_int(context)?;
            let to = to.to_int(context)?;
            if *to_inclusive {
                (from..=to).map(|i| Primitive::Number(i as f64)).collect()
            } else {
                (from..to).map(|i| Primitive::Number(i as f64)).collect()
            }
        }
        PreIterator::Parameter(p) => {
            let value = p.as_iterator(context)?;
            value.to_primitive_set()
        }
    };
    Ok(NamedSet::new(range.vars_tuple.clone(), set))
}

pub fn transform_condition(
    condition: &PreCondition,
    context: &mut TransformerContext,
) -> Result<Condition, TransformError> {
    let lhs = condition.lhs.into_exp(context)?;
    let rhs = condition.rhs.into_exp(context)?;
    Ok(Condition::new(&lhs, condition.condition_type.clone(), &rhs))
}

pub fn transform_condition_with_iteration(
    condition: &PreCondition,
    context: &mut TransformerContext,
) -> Result<Vec<Condition>, TransformError> {
    /*
    let iteration = condition.iteration.as_ref().map(|i| transform_set(i, context)?);
        match iteration {
        //exclusive range
        Some(iteration_range) => (iteration_range.from..iteration_range.to)
            .map(|i| {
                context.add_scope();
                context.declare_variable(&iteration_range.name, i as f64)?;
                let condition = transform_condition(condition, context)?;
                context.pop_scope();
                Ok(condition)
            })
            .collect(),
        None => {
            let condition = transform_condition(condition, context)?;
            Ok(vec![condition])
        }
    }
     */
    todo!("implement the iteration transformation")
}

pub fn transform_objective(
    objective: &PreObjective,
    context: &mut TransformerContext,
) -> Result<Objective, TransformError> {
    let rhs = objective.rhs.into_exp(context)?;
    Ok(Objective::new(objective.objective_type.clone(), &rhs))
}

pub fn transform_problem(
    problem: &PreProblem,
    context: &mut TransformerContext,
) -> Result<Problem, TransformError> {
    let objective = transform_objective(&problem.objective, context)?;
    let conditions = problem
        .conditions
        .iter()
        .map(|condition| transform_condition_with_iteration(condition, context))
        .collect::<Result<Vec<Vec<Condition>>, TransformError>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    Ok(Problem::new(objective, conditions))
}
