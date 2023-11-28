use crate::{
    consts::{Comparison, ConstantValue, Op, OptimizationType, Primitive},
    parser::{
        PreAccess, PreArrayAccess, PreCondition, PreExp, PreIterOfArray, PreIterator, PreObjective,
        PreProblem, PreSet,
    }, functions::ToNum,
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
#[derive(Debug)]
pub struct Range {
    pub name: String,
    pub from: i32,
    pub to: i32,
}
impl Range {
    pub fn new(name: String, from: i32, to: i32) -> Self {
        //doesn't matter the order of from and to, it only matters the range
        if from > to {
            Self {
                name,
                from: to,
                to: from,
            }
        } else {
            Self { name, from, to }
        }
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
    MissingConstant(String),
    MissingVariable(String),
    AlreadyExistingVariable(String),
    OutOfBounds(String),
    ExpectedNumber(String),
    WrongArgument(String),
    NotFound(String),
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

    pub fn flatten_variable_name(&self, name: &Vec<String>) -> Result<String, TransformError> {
        let mut replaced_vars = vec![false; name.len()];
        let mut name = name.clone();
        for (variable_name, value) in self.variables.iter() {
            let index = name.iter().position(|v| v == variable_name);
            match index {
                Some(index) => {
                    name[index] = value.to_string();
                    replaced_vars[index] = true;
                }
                None => continue,
            }
        }
        //check if every variable is replaced
        if replaced_vars.iter().all(|v| *v) {
            let name = name.join("_");
            Ok(name)
        } else {
            Err(TransformError::MissingVariable(name.join("_")))
        }
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
    pub fn add_variable(&mut self, variable: &String, value: f64) -> Result<f64, TransformError> {
        if self.variables.contains_key(variable) {
            return Err(TransformError::AlreadyExistingVariable(variable.clone()));
        }
        self.variables.insert(variable.clone(), value);
        Ok(value)
    }
    pub fn remove_variable(&mut self, variable: &String) -> Result<f64, TransformError> {
        if !self.variables.contains_key(variable) {
            return Err(TransformError::MissingVariable(variable.clone()));
        }
        let value = self.variables.remove(variable).unwrap();
        Ok(value)
    }
    pub fn update_variable(
        &mut self,
        variable: &String,
        value: f64,
    ) -> Result<f64, TransformError> {
        if !self.variables.contains_key(variable) {
            return Err(TransformError::MissingVariable(variable.clone()));
        }
        self.variables.insert(variable.clone(), value);
        Ok(value)
    }
    pub fn get_constant(&self, name: &str) -> Option<&ConstantValue> {
        self.constants.get(name)
    }
    pub fn get_primitive(&self, name: &str) -> Result<Primitive, TransformError> {
        match self.get_constant(name) {
            Some(constant) => match &constant {
                ConstantValue::Number(value) => Ok(Primitive::Number(*value)),
                ConstantValue::OneDimArray(array) => Ok(Primitive::NumberArray(array)),
                ConstantValue::TwoDimArray(array) => Ok(Primitive::NumberMatrix(array)),
                ConstantValue::Graph(graph) => Ok(Primitive::Graph(graph)),
                ConstantValue::String(string) => Ok(Primitive::String(string)),
            },
            None => Err(TransformError::MissingConstant(name.to_string())),
        }
    }
    pub fn get_numerical_constant(&self, name: &str) -> Result<&f64, TransformError> {
        match self.get_constant(name) {
            Some(constant) => match &constant {
                ConstantValue::Number(value) => Ok(value),
                _ => Err(TransformError::WrongArgument(format!(
                    "Expected a number, check the definition of {}",
                    name
                ))),
            },
            None => Err(TransformError::MissingConstant(name.to_string())),
        }
    }
    pub fn get_1d_array_constant_value(&self, name: &str, i: usize) -> Result<f64, TransformError> {
        match self.get_constant(name) {
            Some(constant) => match &constant {
                ConstantValue::OneDimArray(array) => match array.get(i).map(|v| *v) {
                    Some(value) => Ok(value),
                    None => Err(TransformError::OutOfBounds(format!("{}[{}]", name, i))),
                },
                _ => Err(TransformError::WrongArgument(format!(
                    "Expected a 1d array, check the definition of {}",
                    name
                ))),
            },
            None => Err(TransformError::MissingConstant(name.to_string())),
        }
    }
    pub fn get_2d_array_constant_value(
        &self,
        name: &str,
        i: usize,
        j: usize,
    ) -> Result<f64, TransformError> {
        match self.get_constant(name) {
            Some(constant) => match &constant {
                ConstantValue::TwoDimArray(array) => {
                    let value = array.get(i).and_then(|row| row.get(j)).map(|v| *v);
                    match value {
                        Some(value) => Ok(value),
                        None => Err(TransformError::OutOfBounds(format!(
                            "{}[{}][{}]",
                            name, i, j
                        ))),
                    }
                }
                _ => Err(TransformError::WrongArgument(format!(
                    "Expected a 2d array, check the definition of {}",
                    name
                ))),
            },
            None => Err(TransformError::MissingConstant(name.to_string())),
        }
    }
    pub fn get_1d_array_length(&self, name: &str) -> Result<usize, TransformError> {
        match self.get_constant(name) {
            Some(constant) => match &constant {
                ConstantValue::OneDimArray(array) => Ok(array.len()),
                _ => Err(TransformError::WrongArgument(format!(
                    "Expected a 1d array, check the definition of {}",
                    name
                ))),
            },
            None => Err(TransformError::MissingConstant(name.to_string())),
        }
    }
    pub fn get_2d_array_length(
        &self,
        name: &str,
        index: usize,
    ) -> Result<(usize, usize), TransformError> {
        match self.get_constant(name) {
            Some(constant) => match &constant {
                ConstantValue::TwoDimArray(array) => {
                    let rows = array.len();
                    let cols = array.get(index).map(|row| row.len());
                    match cols {
                        Some(cols) => Ok((rows, cols)),
                        None => Err(TransformError::OutOfBounds(format!("{}[{}]", name, index))),
                    }
                }
                _ => Err(TransformError::WrongArgument(format!(
                    "Expected a 2d array, check the definition of {}",
                    name
                ))),
            },
            None => Err(TransformError::MissingConstant(name.to_string())),
        }
    }
    pub fn get_array_access_value(
        &self,
        array_access: &PreArrayAccess,
    ) -> Result<f64, TransformError> {
        let indexes = array_access
            .accesses
            .iter()
            .map(|access| transform_pre_access(access, self))
            .collect::<Result<Vec<usize>, TransformError>>()?;
        match indexes.as_slice() {
            [i] => Ok(self.get_1d_array_constant_value(&array_access.name, *i)?),
            [i, j] => Ok(self.get_2d_array_constant_value(&array_access.name, *i, *j)?),
            _ => Err(TransformError::OutOfBounds(format!(
                "limit of 2d arrays, trying to access {}[{:?}]",
                array_access.name, indexes
            ))),
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<&f64> {
        self.variables.get(name)
    }
}

pub fn transform(pre_problem: &PreProblem) -> Result<Problem, TransformError> {
    let constants = pre_problem
        .constants
        .iter()
        .map(|c| (c.name.clone(), c.value.clone()))
        .collect::<Vec<_>>();
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
        .collect::<Vec<_>>();
    let constants = HashMap::from_iter(constants);
    let variables = HashMap::from_iter(initial_variables);
    let mut context = TransformerContext::new(constants, variables);
    transform_problem(pre_problem, &mut context)
}

pub fn transform_len_of(
    len_of: &PreIterOfArray,
    context: &TransformerContext,
) -> Result<usize, TransformError> {
    match len_of {
        PreIterOfArray::Array(name) => context.get_1d_array_length(name),
        PreIterOfArray::ArrayAccess(array_access) => {
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
                (None, _) => Err(TransformError::MissingConstant(array_access.name.clone())),
                _ => Err(TransformError::OutOfBounds(array_access.name.clone())),
            }
        }
    }
}

pub fn transform_pre_access(
    pre_access: &PreAccess,
    context: &TransformerContext,
) -> Result<usize, TransformError> {
    match pre_access {
        PreAccess::Number(value) => Ok(*value as usize),
        PreAccess::Variable(name) => {
            //make sure the number is an integer, or report an error
            let variable_value = get_variable_value(name, context)?;
            match variable_value == (variable_value as usize) as f64 {
                true => Ok(variable_value as usize),
                false => Err(TransformError::ExpectedNumber(name.clone())),
            }
        }
    }
}

pub fn get_variable_value(
    name: &String,
    context: &TransformerContext,
) -> Result<f64, TransformError> {
    let variable_value = match context.get_variable(name) {
        Some(value) => *value,
        None => return Err(TransformError::MissingVariable(name.clone())),
    };
    Ok(variable_value)
}

pub fn transform_range_value(
    range_value: &dyn ToNum,
    context: &TransformerContext,
    is_inclusive: bool,
) -> Result<i32, TransformError> {
    //range is exclusive, so we need to add 1 to get the correct value
    let append = if is_inclusive { 0 } else { 1 };
    match range_value.to_num(context) {
        Ok(value) => {
            if value == 0.0 {
                Ok(0)
            } else {
                Ok(value as i32 + append)
            }
        },
        //TODO: add a better error message
        Err(e) => Err(TransformError::WrongArgument("".to_string())),
    }
}

pub fn transform_set(
    range: &PreSet,
    context: &TransformerContext,
) -> Result<Range, TransformError> {
    todo!("implement the set transformation into a iterator")

    /*
    let from = transform_range_value(&range.from, context, 0)?;
    let to = transform_range_value(&range.to, context, 1)?;
    Ok(Range::new(range.name.clone(), from, to))
     */
}
pub fn transform_pre_array_access(
    array_access: &PreArrayAccess,
    context: &TransformerContext,
) -> Result<f64, TransformError> {
    let indexes = array_access
        .accesses
        .iter()
        .map(|access| transform_pre_access(access, context))
        .collect::<Result<Vec<usize>, TransformError>>()?;
    match indexes.as_slice() {
        [i] => Ok(context.get_1d_array_constant_value(&array_access.name, *i)?),
        [i, j] => Ok(context.get_2d_array_constant_value(&array_access.name, *i, *j)?),
        _ => Err(TransformError::OutOfBounds(format!(
            "limit of 2d arrays, trying to access {}[{:?}]",
            array_access.name, indexes
        ))),
    }
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
    let iteration = match &condition.iteration {
        Some(iteration) => Some(transform_set(iteration, context)?),
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
