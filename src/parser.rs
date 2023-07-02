use crate::rules_parser::{parse_bounds_list, parse_condition_list, parse_objective};
use pest::{
    iterators::{Pair},
    Parser,
};

/*
    TODO: Add Summatory of variables using indices where the name of the index is replaced by the value of the index, anything can be inside of the sum block, but only variables will be affected
    Example:
        sum(i in 1..2, j in 1..3, ...){Ci*Xij...}
        sum(i in 1..2){sum(j in 1..3){Ci*Xij}}
        etc...
        gets converted to:

        C11*X11 + C12*X12 + C13*X13 + C21*X21 + C22*X22 + C23*X23
    --------------------------------------------------------------
    TODO: Add declaration of constants, like arrays
    Example:
        min sum(i in 1..2){C[i]*Xi}
        s.t.
            X1 + X2 <= 1
        where
            C = [15, 30]
        gets converted to:

        min 15*X1 + 30*X2
        s.t.
            X1 + X2 <= 1
    --------------------------------------------------------------
    TODO: Make it possible to define constraints with iterable variables
    Example:
        max sum(i in 1..2){Ci*Xi}
        s.t.
            sum(i in 1..2){Xij} <= b[j] for j in 1..2
        where
            C = [15, 30]
            b = [1, 2]
        gets converted to:

        max 15*X1 + 30*X2
        s.t.
            X11 + X12 <= 1
            X21 + X22 <= 2
    --------------------------------------------------------------
    TODO: Change the bounds to be defined as shortcuts added as metadata
    Example:
        Xi Positive for i in 1..2
        Yj Binary for j in 1..2
        Z1, Z3 Free
    
 */



/*
--------------------------Grammar--------------------------

problem = {
    SOI ~
      objective ~ NEWLINE ~
      st ~ NEWLINE ~
          condition_list ~
          bounds_list ~
  EOI
}
st = _{ ^"s.t." }
objective = {(^"min" | ^"max") ~ exp_list}
condition = {exp_list ~ comparison ~ exp_list}
condition_list = { (condition ~ NEWLINE?)+ }
bounds = { comma_separated_vars ~ comparison ~ number }
bounds_list = { (bounds ~ NEWLINE?)* }
exp_list = _{ exp+ }
exp = _{ function | parenthesis | mod | number | variable | op  }
min = { ^"min" ~ "{" ~ comma_separated_exp ~ "}" }
max = { ^"max" ~ "{" ~ comma_separated_exp ~ "}" }
function = _{min | max}
comma_separated_exp = _{ ( exp_block~ ","?)+ }
comma_separated_vars = { ( variable~ ","?)+ }
exp_block = { exp+ }
min_max = _{ ^"min" | ^"max" }
mod = { "|" ~ exp_list ~ "|" }
parenthesis = { "(" ~ exp_list ~ ")" }
comparison = @{  "<=" | ">=" | "=" | "<" | ">"}
variable = @{ ( LETTER+ ~ (NUMBER | "_" | "-")*)}
number = @{ '0'..'9'+ ~ ("." ~ '0'..'9'+)?}
op = @{ "*" | "+" | "-" | "/" }
WHITESPACE = _{ " " | "\t" }
*/

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct PLParser;

#[derive(Debug, PartialEq, Clone)]
pub enum Comparison {
    LowerOrEqual,
    UpperOrEqual,
    Equal,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}
impl Operator {
    pub fn precedence(&self) -> u8 {
        match self {
            Operator::Add => 1,
            Operator::Sub => 1,
            Operator::Mul => 2,
            Operator::Div => 2,
        }
    }
}

#[derive(Debug)]
pub enum Exp {
    Number(f64),
    Variable(String),
    Mod(Box<Exp>),
    Min(Vec<Exp>),
    Max(Vec<Exp>),
    BinaryOperation(Operator, Box<Exp>, Box<Exp>),
    UnaryNegation(Box<Exp>),
}
impl Exp {
    pub fn to_boxed(self) -> Box<Exp> {
        Box::new(self)
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum OptimizationType {
    Min,
    Max,
}

#[derive(Debug)]
pub struct Objective {
    objective_type: OptimizationType,
    rhs: Exp,
}
impl Objective {
    pub fn new(objective_type: OptimizationType, rhs: Exp) -> Self {
        Self {
            objective_type,
            rhs,
        }
    }
}

#[derive(Debug)]
pub struct Condition {
    lhs: Exp,
    condition_type: Comparison,
    rhs: Exp,
}
impl Condition {
    pub fn new(lhs: Exp, condition_type: Comparison, rhs: Exp) -> Self {
        Self {
            lhs,
            condition_type,
            rhs,
        }
    }
}
#[derive(Debug)]
pub struct Bounds {
    variables: Vec<String>,
    condition_type: Comparison,
    bound: f64,
}
impl Bounds {
    pub fn new(variables: Vec<String>, condition_type: Comparison, bound: f64) -> Self {
        Self {
            variables,
            condition_type,
            bound,
        }
    }
}
#[derive(Debug)]
pub struct Problem {
    objective: Objective,
    conditions: Vec<Condition>,
    bounds_list: Vec<Bounds>,
}

impl Problem {
    pub fn new(objective: Objective, conditions: Vec<Condition>, bounds_list: Vec<Bounds>) -> Self {
        Self {
            objective,
            conditions,
            bounds_list,
        }
    }
}

pub fn parse(source: &String) -> Result<Problem, String> {
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
                Err(err) => Err(format!("{:#?}", err)),
            }
        },
        Err(err) => Err(format!("{:#?}", err)),
    }

}

#[derive(Debug)]
pub enum Error {
    UnexpectedToken(String),
    MissingToken(String),
}

fn parse_problem(problem: Pair<'_, Rule>) -> Result<Problem, Error> {
    let mut objective: Option<Objective> = None;
    let mut conditions: Option<Vec<Condition>> = None;
    let mut bounds_list: Option<Vec<Bounds>> = None;
    for pair in problem.into_inner() {
        match pair.as_rule() {
            Rule::objective => {
                let obj = parse_objective(pair)?;
                objective = Some(obj);
            }
            Rule::condition_list => {
                conditions = Some(parse_condition_list(pair)?);
            }
            Rule::bounds_list => {
                bounds_list = Some(parse_bounds_list(pair)?);
            }
            Rule::EOI => {}
            _ => return Err(Error::UnexpectedToken(pair.as_str().to_string())),
        }
    }
    if objective.is_none() {
        return Err(Error::MissingToken("Objective".to_string()));
    }
    if conditions.is_none() {
        return Err(Error::MissingToken("Conditions".to_string()));
    }
    if bounds_list.is_none() {
        return Err(Error::MissingToken("Bounds".to_string()));
    }
    Ok(Problem {
        objective: objective.unwrap(),
        conditions: conditions.unwrap(),
        bounds_list: bounds_list.unwrap(),
    })
}
