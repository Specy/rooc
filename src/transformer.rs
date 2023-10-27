use std::collections::HashMap;

use crate::{
    consts::{Comparison, ConstantValue, Operator, OptimizationType},
    parser::{
        PreAccess, PreArrayAccess, PreCondition, PreLenOf, PreObjective, PreProblem, PreRange,
        PreRangeValue,
    },
};

#[derive(Debug, Clone)]
pub enum Exp {
    Number(f64),
    Variable(String),
    Mod(Box<Exp>),
    Min(Vec<Exp>),
    Max(Vec<Exp>),
    Parenthesis(Box<Exp>),
    BinaryOperation(Operator, Box<Exp>, Box<Exp>),
    UnaryNegation(Box<Exp>),
}

impl Exp {
    pub fn to_boxed(self) -> Box<Exp> {
        Box::new(self)
    }
    pub fn to_string(&self) -> String {
        match self {
            Exp::Number(value) => value.to_string(),
            Exp::Variable(name) => name.clone(),
            Exp::Mod(exp) => format!("|{}|", exp.to_string()),
            Exp::Min(exps) => format!(
                "min({})",
                exps.iter()
                    .map(|exp| exp.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Exp::Max(exps) => format!(
                "max({})",
                exps.iter()
                    .map(|exp| exp.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Exp::BinaryOperation(operator, lhs, rhs) => format!(
                "{} {} {}",
                lhs.to_string(),
                operator.to_string(),
                rhs.to_string()
            ),
            Exp::Parenthesis(exp) => format!("({})", exp.to_string()),
            Exp::UnaryNegation(exp) => format!("-{}", exp.to_string()),
        }
    }
    pub fn remove_root_parenthesis(&self) -> &Exp {
        match self {
            Exp::Parenthesis(exp) => exp,
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
#[derive(Debug)]
pub struct Range {
    pub name: String,
    pub from: i32,
    pub to: i32,
}
impl Range {
    pub fn new(name: String, from: i32, to: i32) -> Self {
        Self { name, from, to }
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
            .collect::<Vec<String>>()
            .join("\n\t");
        format!("{}\ns.t\n\t{}", self.objective.to_string(), conditions)
    }
}

#[derive(Debug)]
pub enum TransfromError {
    MissingConstant(String),
    MissingVariable(String),
    AlreadyExistingVariable(String),
    OutOfBounds(String),
    ExpectedNumber(String),
    WrongArgument(String),
}

#[derive(Debug)]
pub struct TransformerContext {
    constants: HashMap<String, ConstantValue>,
    variables: HashMap<String, f64>,
}
impl TransformerContext {
    pub fn new(constants: HashMap<String, ConstantValue>, variables: HashMap<String, f64>) -> Self {
        Self {
            constants,
            variables,
        }
    }

    pub fn flatten_variable_name(&self, name: &str) -> Result<String, TransfromError> {
        let mut name = name.to_string();

        for (variable_name, value) in self.variables.iter() {
            name = name.replace(variable_name, format!("_{}", value).as_str());
        }
        //check if every variable is replaced, the indexes must not have any letters
        if name.chars().any(|c| c.is_alphabetic()) {
            return Err(TransfromError::MissingVariable(name));
        }
        //remove the starting _ if it exists
        if name.starts_with('_') {
            name = name[1..].to_string();
        }
        Ok(name)
    }

    pub fn add_variable(&mut self, variable: &String, value: f64) -> Result<f64, TransfromError> {
        if self.variables.contains_key(variable) {
            return Err(TransfromError::AlreadyExistingVariable(variable.clone()));
        }
        self.variables.insert(variable.clone(), value);
        Ok(value)
    }
    pub fn remove_variable(&mut self, variable: &String) -> Result<f64, TransfromError> {
        if !self.variables.contains_key(variable) {
            return Err(TransfromError::MissingVariable(variable.clone()));
        }
        let value = self.variables.remove(variable).unwrap();
        Ok(value)
    }
    pub fn update_variable(
        &mut self,
        variable: &String,
        value: f64,
    ) -> Result<f64, TransfromError> {
        if !self.variables.contains_key(variable) {
            return Err(TransfromError::MissingVariable(variable.clone()));
        }
        self.variables.insert(variable.clone(), value);
        Ok(value)
    }
    pub fn get_constant(&self, name: &str) -> Option<&ConstantValue> {
        self.constants.get(name)
    }
    pub fn get_numerical_constant(&self, name: &str) -> Result<&f64, TransfromError> {
        match self.get_constant(name) {
            Some(constant) => match &constant {
                ConstantValue::Number(value) => Ok(value),
                _ => Err(TransfromError::WrongArgument(format!(
                    "Expected a number, check the definition of {}",
                    name
                ))),
            },
            None => Err(TransfromError::MissingConstant(name.to_string())),
        }
    }
    pub fn get_1d_array_constant_value(&self, name: &str, i: usize) -> Result<f64, TransfromError> {
        match self.get_constant(name) {
            Some(constant) => match &constant {
                ConstantValue::OneDimArray(array) => match array.get(i).map(|v| *v) {
                    Some(value) => Ok(value),
                    None => Err(TransfromError::OutOfBounds(format!("{}[{}]", name, i))),
                },
                _ => Err(TransfromError::WrongArgument(format!(
                    "Expected a 1d array, check the definition of {}",
                    name
                ))),
            },
            None => Err(TransfromError::MissingConstant(name.to_string())),
        }
    }
    pub fn get_2d_array_constant_value(
        &self,
        name: &str,
        i: usize,
        j: usize,
    ) -> Result<f64, TransfromError> {
        match self.get_constant(name) {
            Some(constant) => match &constant {
                ConstantValue::TwoDimArray(array) => {
                    let value = array.get(i).and_then(|row| row.get(j)).map(|v| *v);
                    match value {
                        Some(value) => Ok(value),
                        None => Err(TransfromError::OutOfBounds(format!(
                            "{}[{}][{}]",
                            name, i, j
                        ))),
                    }
                }
                _ => Err(TransfromError::WrongArgument(format!(
                    "Expected a 2d array, check the definition of {}",
                    name
                ))),
            },
            None => Err(TransfromError::MissingConstant(name.to_string())),
        }
    }
    pub fn get_1d_array_length(&self, name: &str) -> Result<usize, TransfromError> {
        match self.get_constant(name) {
            Some(constant) => match &constant {
                ConstantValue::OneDimArray(array) => Ok(array.len()),
                _ => Err(TransfromError::WrongArgument(format!(
                    "Expected a 1d array, check the definition of {}",
                    name
                ))),
            },
            None => Err(TransfromError::MissingConstant(name.to_string())),
        }
    }
    pub fn get_2d_array_length(
        &self,
        name: &str,
        index: usize,
    ) -> Result<(usize, usize), TransfromError> {
        match self.get_constant(name) {
            Some(constant) => match &constant {
                ConstantValue::TwoDimArray(array) => {
                    let rows = array.len();
                    let cols = array.get(index).map(|row| row.len());
                    match cols {
                        Some(cols) => Ok((rows, cols)),
                        None => Err(TransfromError::OutOfBounds(format!("{}[{}]", name, index))),
                    }
                }
                _ => Err(TransfromError::WrongArgument(format!(
                    "Expected a 2d array, check the definition of {}",
                    name
                ))),
            },
            None => Err(TransfromError::MissingConstant(name.to_string())),
        }
    }
    pub fn get_variable(&self, name: &str) -> Option<&f64> {
        self.variables.get(name)
    }
}

pub fn transform(pre_problem: &PreProblem) -> Result<Problem, TransfromError> {
    let constants = pre_problem
        .constants
        .iter()
        .map(|c| (c.name.clone(), c.value.clone()))
        .collect::<Vec<(String, ConstantValue)>>();
    let initial_variables = pre_problem
        .constants
        .iter()
        .filter(|c| match &c.value {
            ConstantValue::Number(_) => true,
            _ => false,
        })
        .map(|c| match &c.value {
            ConstantValue::Number(value) => (c.name.clone(), *value),
            _ => unreachable!(),
        })
        .collect::<Vec<(String, f64)>>();
    let constants = HashMap::from_iter(constants);
    let variables = HashMap::from_iter(initial_variables);
    let mut context = TransformerContext::new(constants, variables);
    transform_problem(pre_problem, &mut context)
}

pub fn transform_len_of(
    len_of: &PreLenOf,
    context: &TransformerContext,
) -> Result<usize, TransfromError> {
    match len_of {
        PreLenOf::Array(name) => context.get_1d_array_length(name),
        PreLenOf::ArrayAccess(array_access) => {
            let index = array_access.accesses.first();
            match (index, array_access.accesses.len()) {
                (Some(access), 1) => match access {
                    PreAccess::Number(i) => {
                        let value = context.get_2d_array_length(&array_access.name, *i as usize);
                        match value {
                            Ok((_, rows)) => Ok(rows),
                            Err(e) => Err(e),
                        }
                    }
                    PreAccess::Variable(name) => {
                        let variable_value = get_variable_value(name, context)?;
                        let value = context
                            .get_2d_array_length(&array_access.name, variable_value as usize);
                        match value {
                            Ok((_, rows)) => Ok(rows),
                            Err(e) => Err(e),
                        }
                    }
                },
                (None, _) => Err(TransfromError::MissingConstant(array_access.name.clone())),
                _ => Err(TransfromError::OutOfBounds(array_access.name.clone())),
            }
        }
    }
}

pub fn transform_pre_access(
    pre_access: &PreAccess,
    context: &TransformerContext,
) -> Result<usize, TransfromError> {
    match pre_access {
        PreAccess::Number(value) => Ok(*value as usize),
        PreAccess::Variable(name) => {
            //make sure the number is an integer, or report an error
            let variable_value = get_variable_value(name, context)?;
            match variable_value == (variable_value as usize) as f64 {
                true => Ok(variable_value as usize),
                false => Err(TransfromError::ExpectedNumber(name.clone())),
            }
        }
    }
}

pub fn get_variable_value(
    name: &String,
    context: &TransformerContext,
) -> Result<f64, TransfromError> {
    let variable_value = match context.get_variable(name) {
        Some(value) => *value,
        None => return Err(TransfromError::MissingVariable(name.clone())),
    };
    Ok(variable_value)
}

pub fn transform_range_value(
    range_value: &PreRangeValue,
    context: &TransformerContext,
    add_if_number: i32,
) -> Result<i32, TransfromError> {
    match range_value {
        //range is exclusive, so we need to add 1 to get the correct value
        PreRangeValue::Number(value) => Ok((*value as i32) + add_if_number),
        PreRangeValue::LenOf(len_of) => {
            let len = transform_len_of(len_of, context)?;
            Ok(len as i32)
        }
    }
}
pub fn transform_range(
    range: &PreRange,
    context: &TransformerContext,
) -> Result<Range, TransfromError> {
    let from = transform_range_value(&range.from, context, 0)?;
    let to = transform_range_value(&range.to, context, 1)?;
    Ok(Range::new(range.name.clone(), from, to))
}
pub fn transform_pre_array_access(
    array_access: &PreArrayAccess,
    context: &TransformerContext,
) -> Result<f64, TransfromError> {
    let indexes = array_access
        .accesses
        .iter()
        .map(|access| transform_pre_access(access, context))
        .collect::<Result<Vec<usize>, TransfromError>>()?;
    match indexes.as_slice() {
        [i] => Ok(context.get_1d_array_constant_value(&array_access.name, *i)?),
        [i, j] => Ok(context.get_2d_array_constant_value(&array_access.name, *i, *j)?),
        _ => Err(TransfromError::OutOfBounds(format!(
            "limit of 2d arrays, trying to access {}[{:?}]",
            array_access.name, indexes
        ))),
    }
}

pub fn transform_condition(
    condition: &PreCondition,
    context: &mut TransformerContext,
) -> Result<Condition, TransfromError> {
    let lhs = condition.lhs.into_exp(context)?;
    let rhs = condition.rhs.into_exp(context)?;
    Ok(Condition::new(&lhs, condition.condition_type.clone(), &rhs))
}

pub fn transform_condition_with_range(
    condition: &PreCondition,
    context: &mut TransformerContext,
) -> Result<Vec<Condition>, TransfromError> {
    let iteration = match &condition.iteration {
        Some(iteration) => Some(transform_range(iteration, context)?),
        None => None,
    };
    match iteration {
        //exclusive range
        Some(iteration_range) => (iteration_range.from..iteration_range.to)
            .map(|i| {
                context.add_variable(&iteration_range.name, i as f64)?;
                let condition = transform_condition(condition, context)?;
                context.remove_variable(&iteration_range.name)?;
                Ok(condition)
            })
            .collect(),
        None => {
            let condition = transform_condition(condition, context)?;
            Ok(vec![condition])
        }
    }
}

pub fn transform_objective(
    objective: &PreObjective,
    context: &mut TransformerContext,
) -> Result<Objective, TransfromError> {
    let rhs = objective.rhs.into_exp(context)?;
    Ok(Objective::new(objective.objective_type.clone(), &rhs))
}

pub fn transform_problem(
    problem: &PreProblem,
    context: &mut TransformerContext,
) -> Result<Problem, TransfromError> {
    let objective = transform_objective(&problem.objective, context)?;
    let conditions = problem
        .conditions
        .iter()
        .map(|condition| transform_condition_with_range(condition, context))
        .collect::<Result<Vec<Vec<Condition>>, TransfromError>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<Condition>>();
    Ok(Problem::new(objective, conditions))
}
