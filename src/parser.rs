use crate::bail_missing_token;
use crate::consts::{
    Comparison, CompilationError, Constant, FunctionCall, InputSpan, Op, OptimizationType,
    Parameter, ParseError, Primitive, Spanned,
};
use crate::functions::ToNum;
use crate::rules_parser::{parse_condition_list, parse_consts_declaration, parse_objective};
use crate::transformer::{
    transform_set, Exp, NamedSet, PrimitiveSet, TransformError, TransformerContext,
};
use core::panic;
use pest::iterators::Pair;
use pest::Parser;
use std::fmt::Debug;

/*

   --------------------------------------------------------------
   TODO: Change the bounds to be defined as shortcuts added as metadata
   Example:
       Xi Positive for i in 1..2
       Yj Binary for j in 1..2
       Z1, Z3 Free

*/

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct PLParser;

#[derive(Debug)]
pub enum PreIterOfArray {
    Array(String),
    ArrayAccess(PreArrayAccess),
}
#[derive(Debug)]
pub struct PreSet {
    pub vars_tuple: Vec<String>,
    pub iterator: PreIterator,
    pub span: InputSpan,
}
impl PreSet {
    pub fn new(vars_tuple: Vec<String>, iterator: PreIterator, span: InputSpan) -> Self {
        Self {
            vars_tuple,
            iterator,
            span,
        }
    }
}

#[derive(Debug)]
pub enum PreNode {
    Name(String),
    Variable(String),
}

#[derive(Debug)]
pub enum PreIterator {
    Range {
        from: Box<dyn ToNum>,
        to: Box<dyn ToNum>,
        to_inclusive: bool,
    },
    Parameter(Parameter),
}

#[derive(Debug)]
pub struct PreArrayAccess {
    pub name: String,
    pub accesses: Vec<Parameter>,
}
impl PreArrayAccess {
    pub fn new(name: String, accesses: Vec<Parameter>) -> Self {
        Self { name, accesses }
    }
    pub fn to_string(&self) -> String {
        let rest = self
            .accesses
            .iter()
            .map(|a| format!("[{}]", a.to_string()))
            .collect::<Vec<String>>()
            .join("");
        format!("{}{}", self.name, rest)
    }
}

#[derive(Debug)]
pub struct CompoundVariable {
    pub name: String,
    pub indexes: Vec<String>,
}
impl CompoundVariable {
    pub fn new(name: String, indexes: Vec<String>) -> Self {
        Self { name, indexes }
    }
    pub fn to_string(&self) -> String {
        format!("{}_{}", self.name, self.indexes.join("_"))
    }
}

#[derive(Debug)]
pub enum PreExp {
    Number(Spanned<f64>),
    Mod(Spanned<Box<PreExp>>),
    Min(Spanned<Vec<PreExp>>),
    Max(Spanned<Vec<PreExp>>),
    Variable(Spanned<String>),
    CompoundVariable(Spanned<CompoundVariable>),
    BinaryOperation(Spanned<Op>, Box<PreExp>, Box<PreExp>),
    UnaryNegation(Spanned<Box<PreExp>>),
    ArrayAccess(Spanned<PreArrayAccess>),
    Sum(Vec<PreSet>, Spanned<Box<PreExp>>),
    FunctionCall(Spanned<Box<dyn FunctionCall>>),
}

impl PreExp {
    pub fn to_boxed(self) -> Box<PreExp> {
        Box::new(self)
    }
    pub fn get_span(&self) -> &InputSpan {
        match self {
            Self::Number(n) => n.get_span(),
            Self::Mod(exp) => exp.get_span(),
            Self::Min(exps) => exps.get_span(),
            Self::Max(exps) => exps.get_span(),
            Self::Variable(name) => name.get_span(),
            Self::CompoundVariable(c) => c.get_span(),
            Self::BinaryOperation(op, _, _) => op.get_span(),
            Self::UnaryNegation(exp) => exp.get_span(),
            Self::ArrayAccess(array_access) => array_access.get_span(),
            Self::Sum(_, exp_body) => exp_body.get_span(),
            Self::FunctionCall(function_call) => function_call.get_span(),
        }
    }
    //should this consume self?
    pub fn into_exp(&self, context: &mut TransformerContext) -> Result<Exp, TransformError> {
        let exp = match self {
            Self::BinaryOperation(op, lhs, rhs) => {
                let lhs = lhs
                    .into_exp(context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                let rhs = rhs
                    .into_exp(context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                match (op, lhs, rhs) {
                    (op, lhs, rhs) => Ok(Exp::BinOp(**op, lhs.to_box(), rhs.to_box())),
                }
            }
            Self::Number(n) => Ok(Exp::Number(**n)),
            Self::Mod(exp) => {
                let inner = exp
                    .into_exp(context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                Ok(Exp::Mod(inner.to_box()))
            }
            Self::Min(exps) => {
                let mut parsed_exp = Vec::new();
                let mut context = context;
                for exp in exps.iter() {
                    let inner = exp
                        .into_exp(context)
                        .map_err(|e| e.to_spanned_error(self.get_span()))?;
                    parsed_exp.push(inner);
                }
                Ok(Exp::Min(parsed_exp))
            }
            Self::Max(exps) => {
                let mut parsed_exp = Vec::new();
                let mut context = context;
                for exp in exps.iter() {
                    let inner = exp
                        .into_exp(context)
                        .map_err(|e| e.to_spanned_error(self.get_span()))?;
                    parsed_exp.push(inner);
                }
                Ok(Exp::Max(parsed_exp))
            }
            Self::UnaryNegation(exp) => {
                let inner = exp
                    .into_exp(context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                Ok(Exp::Neg(inner.to_box()))
            }
            Self::Variable(name) => {
                let value = context.get_value(&*name).map(|v| match v {
                    Primitive::Number(n) => Ok(Exp::Number(n.clone())),
                    _ => {
                        let err = TransformError::WrongArgument("Number".to_string());
                        Err(err.to_spanned_error(self.get_span()))
                    }
                });
                match value {
                    Some(value) => Ok(value?),
                    None => Ok(Exp::Variable(name.get_span_value().clone())),
                }
            }
            Self::CompoundVariable(c) => {
                let parsed_indexes = context
                    .flatten_variable_name(&c.indexes)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                Ok(Exp::Variable(format!("{}_{}", c.name, parsed_indexes)))
            }
            Self::ArrayAccess(array_access) => {
                let value = context
                    .get_array_access_value(array_access)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                Ok(Exp::Number(value))
            }
            Self::Sum(sets, exp) => {
                let sets = sets
                    .into_iter()
                    .map(|r| transform_set(r, context))
                    .collect::<Result<Vec<_>, TransformError>>()
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                let mut results = Vec::new();
                recursive_set_resolver(&sets, context, &mut results, 0, &|context| {
                    let inner = exp
                        .into_exp(context)
                        .map_err(|e| e.to_spanned_error(self.get_span()))?;
                    Ok(inner)
                })?;
                results.reverse();
                let mut sum = results.pop().unwrap_or(Exp::Number(0.0));
                for result in results {
                    sum = Exp::BinOp(Op::Add, sum.to_box(), result.to_box());
                }
                Ok(sum)
            }
            Self::FunctionCall(function_call) => {
                //TODO improve this, what other types of functions can there be?
                let value = function_call
                    .call(&context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                match value {
                    Primitive::Number(n) => Ok(Exp::Number(n)),
                    _ => {
                        let err = TransformError::WrongArgument("Number".to_string());
                        Err(err.to_spanned_error(self.get_span()))
                    }
                }
            }
        };
        exp.map(|e: Exp| e.flatten())
    }
}

/*

function flatten(exp, ranges, context, values):
    if ranges is empty:
        values.append(exp)
    range = ranges[0]
    for value in range.set:
        context.update_variable(range.name, value)
        flatten(exp, ranges[1:], context, values)
*/

pub fn recursive_set_resolver<T>(
    sets: &Vec<NamedSet>,
    context: &mut TransformerContext,
    results: &mut Vec<T>,
    current_level: usize,
    on_leaf: &dyn Fn(&mut TransformerContext) -> Result<T, TransformError>,
) -> Result<(), TransformError> {
    let range = sets.get(current_level).unwrap();
    context.add_scope();
    for name in range.vars.iter() {
        context.declare_variable(&name, Primitive::Undefined, true)?;
    }
    for value in range.set.iter() {
        if let Primitive::Tuple(t) = value {
            if range.vars.len() > t.len() {
                let error = format!(
                    "Cannot destructure tuple of size {} into {} variables",
                    t.len(),
                    range.vars.len()
                );
                return Err(TransformError::WrongArgument(error));
            }
        }
        for (i, var) in range.vars.iter().enumerate() {
            match value {
                Primitive::Tuple(t) => {
                    if let Some(value) = t.get(i) {
                        context.update_variable(var, value.clone())?;
                    } else {
                        let error = format!(
                            "Cannot destructure tuple of size {} into {} variables",
                            t.len(),
                            range.vars.len()
                        );
                        return Err(TransformError::WrongArgument(error));
                    }
                }
                _ => {
                    context.update_variable(var, value.clone())?;
                }
            }
        }
        if current_level + 1 >= sets.len() {
            let value = on_leaf(context)?;
            results.push(value);
        }else{
            recursive_set_resolver(sets, context, results, current_level + 1, on_leaf)?;
        }
    }
    context.pop_scope()?;
    Ok(())
}

#[derive(Debug)]
pub struct PreObjective {
    pub objective_type: OptimizationType,
    pub rhs: PreExp,
}

impl PreObjective {
    pub fn new(objective_type: OptimizationType, rhs: PreExp) -> Self {
        Self {
            objective_type,
            rhs,
        }
    }
}

#[derive(Debug)]
pub struct PreCondition {
    pub lhs: PreExp,
    pub condition_type: Comparison,
    pub rhs: PreExp,
    pub iteration: Vec<PreSet>,
    pub span: InputSpan,
}

impl PreCondition {
    pub fn new(
        lhs: PreExp,
        condition_type: Comparison,
        rhs: PreExp,
        iteration: Vec<PreSet>,
        span: InputSpan,
    ) -> Self {
        Self {
            lhs,
            condition_type,
            rhs,
            iteration,
            span,
        }
    }
}

#[derive(Debug)]
pub enum PreLen {
    LenOfArray(String),
    Number(i32),
}

#[derive(Debug)]
pub struct PreProblem {
    pub objective: PreObjective,
    pub conditions: Vec<PreCondition>,
    pub constants: Vec<Constant>,
}

impl PreProblem {
    pub fn new(
        objective: PreObjective,
        conditions: Vec<PreCondition>,
        constants: Vec<Constant>,
    ) -> Self {
        Self {
            objective,
            conditions,
            constants,
        }
    }
}

pub fn parse(source: &String) -> Result<PreProblem, String> {
    let source = source.trim();
    let problem = PLParser::parse(Rule::problem, source);
    match problem {
        Ok(mut problem) => {
            let problem = problem.next();
            if problem.is_none() {
                return Err("No problem found".to_string());
            }
            let problem = problem.unwrap();
            match parse_problem(problem) {
                Ok(problem) => Ok(problem),
                Err(err) => Err(format!("{}", err.to_string())),
            }
        }
        Err(err) => Err(format!("{:#?}", err)),
    }
}

fn parse_problem(problem: Pair<Rule>) -> Result<PreProblem, CompilationError> {
    let pairs = problem.clone().into_inner();
    let objective = pairs.find_first_tagged("objective").map(parse_objective);
    let conditions = pairs
        .find_first_tagged("conditions")
        .map(|v| parse_condition_list(&v));
    let consts = pairs
        .find_first_tagged("where")
        .map(parse_consts_declaration);
    match (objective, conditions) {
        (Some(obj), Some(cond)) => Ok(PreProblem::new(
            obj?,
            cond?,
            consts.unwrap_or(Ok(Vec::new()))?,
        )),
        _ => bail_missing_token!("Objective and conditions are required", problem),
    }
}
