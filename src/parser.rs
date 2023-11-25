use crate::consts::{
    Comparison, CompilationError, Constant, Op, OptimizationType, ParseError,
};
use crate::err_missing_token;
use crate::rules_parser::{parse_condition_list, parse_consts_declaration, parse_objective};
use crate::transformer::{
    transform_len_of, transform_pre_array_access, transform_set, Exp, Range, TransformError,
    TransformerContext,
};
use pest::iterators::Pair;
use pest::Parser;

/*
   (done)
   TODO: Add Summatory of variables using indices where the name of the index is replaced by the value of the index, anything can be inside of the sum block, but only variables will be affected
   Example:
       sum(i in 1..2, j in 1..3, ...){Ci*Xij...}
       sum(i in 1..2){sum(j in 1..3){Ci*Xij}}
       etc...
       gets converted to:

       C11*X11 + C12*X12 + C13*X13 + C21*X21 + C22*X22 + C23*X23
   --------------------------------------------------------------
   (done)
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
   (done)
   TODO: Make it possible to define constraints with iterable variables
   Example:
       max sum(i in 1..2){Ci*Xi} //TODO! should this be Xi or X_i, should there be an explicit definition?
       s.t.
           sum(i in 1..2){Xij} <= b[j] for j in 1..2
       where
           C = [15, 30]
           b = [1, 2]
       gets converted to:

       max 15*X1 + 30*X2
       s.t.
           X11 + X12 <= 1 //TODO wrong! this can be confused, it needs a separator in between the indexes X1_1 or X_1_1
           X21 + X22 <= 2
   --------------------------------------------------------------
   TODO: Change the bounds to be defined as shortcuts added as metadata
   Example:
       Xi Positive for i in 1..2
       Yj Binary for j in 1..2
       Z1, Z3 Free

*/
//TODO add back the possibility to do 2(10) or 2x etc... currently the parser only accepts 2*(10) or 2*x
/*
--------------------------Grammar--------------------------
problem = {
    SOI ~
    #objective = objective ~ nl+ ~
    ^"s.t." ~ nl+ ~
    #conditions = condition_list ~
    (nl+ ~
    ^"where" ~ nl+ ~
    #where = consts_declaration)? ~
     EOI
}
// required problem body
objective = {
  #objective_type = objective_type ~
  #objective_body = tagged_exp
}
condition_list = { (condition ~ nl+)* ~ condition }
// condition
condition = {
  #lhs = tagged_exp ~
  #relation = comparison ~
  #rhs = tagged_exp ~
  #iteration = for_iteration?
}
// constants declaration
consts_declaration = { (const_declaration ~ nl+)* ~ const_declaration }
const_declaration  = {
  #name = simple_variable ~
  "=" ~
  #value = constant
}

// iterations
for_iteration          = _{ (^"for" | "∀") ~ iteration_declaration }
iteration_declaration_list = { (iteration_declaration ~ comma)* ~ iteration_declaration }
iteration_declaration      =  {
  #name = tuple ~
  (^"in" | "∈") ~
  #iterator = iterator
}
tuple = { simple_variable | ("(" ~ simple_variable ~ (comma ~ simple_variable)* ~ ")")   }
iterator = { range_iterator | edges_iterator | iter_iterator | neighbour_iterator}
//iterators
//TODO should i do a generic syntax for iterators, so a
//generic function grammar with parameters? for the len() too and other functions
//it would mean more parser work but perhaps better error handling and removing the need
//to add it to the grammar every time, but then it would mean that the
//syntax can't allow for implicit multiplication as len(a) could be implied as sum * (a)
//
edges_iterator = { "edges(" ~ #of_graph = simple_variable ~ ")"}
iter_iterator =  { "iter(" ~ #of = (array_access | simple_variable)~")" }
neighbour_iterator = { "neighbours(" ~ #of_graph = simple_variable ~comma ~ #node = (simple_variable | string)~ ")"}
range_iterator = {
  #from = (number | len) ~
  ".." ~
  #to = (number | len)
}
// expressions
tagged_exp = { exp }
exp         = _{ unary_op? ~ exp_body ~ (binary_op ~ unary_op? ~ exp_body)* }
exp_body    = _{ function | parenthesis | modulo | number | array_access | variable }
modulo      =  { "|" ~ exp ~ "|" }
parenthesis =  { "(" ~ exp ~ ")" }
function    = _{ min | max | sum | len}
// functions
min = { ^"min" ~ "{" ~ nl* ~ comma_separated_exp ~ nl* ~"}" }
max = { ^"max" ~ "{" ~ nl* ~ comma_separated_exp ~ nl* ~ "}" }
sum = { (^"sum" | "∑") ~ "(" ~ nl* ~ #range = iteration_declaration_list ~ nl* ~")" ~ "{" ~ nl* ~ #body = tagged_exp ~ nl* ~ "}" }
len = { ^"len(" ~ (array_access | simple_variable) ~ ")" }
// pointer access var[i][j] or var[0] etc...
array_access        = {
  #name = simple_variable ~
  #accesses = pointer_access_list
}
pointer_access_list = { (pointer_access)+ }
pointer_access      = _{ ^"[" ~ (number | simple_variable) ~ ^"]" }
// constants
constant = _{ number | array | graph}
graph = { ^"Graph" ~ "{" ~nl* ~ #body = graph_node_list ~ nl* ~ "}" }
graph_node_list = { graph_node? ~ (comma ~ graph_node)* }
graph_node = { #name = simple_variable ~ ( "->" ~ "[" ~#edges = edges_list ~ "]")?}
edges_list = {  (edge ~ comma)* ~ edge?}
edge = { #node = simple_variable ~ (":" ~ #cost = signed_number)? }
array    =  { "[" ~ nl* ~ ((( number | array) ~ comma)* ~ constant) ~ nl* ~ "]" }
// utilities
comma_separated_exp = _{ (exp ~ comma)* ~ exp }
comma = _{ "," ~ nl? }
nl = _{NEWLINE}
variable       = _{ compound_variable | simple_variable }
// terminal characters
objective_type = @{ ^"min" | ^"max" }
comparison     = @{ "<=" | ">=" | "=" }
// should I make this not a terminal so that i can get variable > compound_variable?
simple_variable   = @{ LETTER+ ~ (NUMBER)* }
compound_variable = @{ simple_variable ~ ("_" ~ LETTER+)+ }
// maybe i should do ("_" ~ LETTER+)+
number    = @{ '0'..'9'+ ~ ("." ~ '0'..'9'+)? }
signed_number = @{ "-"? ~ number}
binary_op = @{ "*" | "+" | "-" | "/" }
unary_op  = @{ "-" }
string = @ { "\"" ~ LETTER* ~ "\""}
// ignore whitespace in whole grammar
WHITESPACE = _{ " " | "\t" }
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
}
impl PreSet {
    pub fn new(vars_tuple: Vec<String>, iterator: PreIterator) -> Self {
        Self {
            vars_tuple,
            iterator,
        }
    }
}

#[derive(Debug)]
pub enum PreRangeValue {
    Number(i32),
    LenOf(PreIterOfArray),
}

#[derive(Debug)]
pub enum PreNode {
    Name(String),
    Variable(String),
}
#[derive(Debug)]
pub enum PreIterator {
    Range {
        from: PreRangeValue,
        to: PreRangeValue,
    },
    EdgesOf {
        graph_name: String,
    },
    NeighboursOf {
        graph_name: String,
        node: PreNode,
    },
    IterOf {
        of: PreIterOfArray,
    },
}

#[derive(Debug)]
pub enum PreAccess {
    Number(i32),
    Variable(String),
}
#[derive(Debug)]
pub struct PreArrayAccess {
    pub name: String,
    pub accesses: Vec<PreAccess>,
}

#[derive(Debug)]
pub enum PreExp {
    Number(f64),
    Mod(Box<PreExp>),
    Min(Vec<PreExp>),
    Max(Vec<PreExp>),
    LenOf(PreIterOfArray),
    Variable(String),
    CompoundVariable { name: String, indexes: Vec<String> },
    BinaryOperation(Op, Box<PreExp>, Box<PreExp>),
    UnaryNegation(Box<PreExp>),
    ArrayAccess(PreArrayAccess),
    //Parenthesis(Box<PreExp>),
    Sum(Vec<PreSet>, Box<PreExp>),
}



impl PreExp {
    pub fn to_boxed(self) -> Box<PreExp> {
        Box::new(self)
    }

    //should this consume self?
    pub fn into_exp(&self, context: &mut TransformerContext) -> Result<Exp, TransformError> {
        let exp = match self {
            Self::BinaryOperation(op, lhs, rhs) => {
                match (op.clone(), lhs.into_exp(context)?, rhs.into_exp(context)?) {
                    (op, lhs, rhs) => Ok(Exp::BinOp(op, lhs.to_box(), rhs.to_box())),
                }
            }
            Self::Number(n) => Ok(Exp::Number(*n)),
            Self::Mod(exp) => Ok(Exp::Mod(exp.into_exp(context)?.to_box())),
            Self::Min(exps) => Ok(Exp::Min(
                exps.iter()
                    .map(|exp| exp.into_exp(context))
                    .collect::<Result<Vec<Exp>, TransformError>>()?,
            )),
            Self::Max(exps) => Ok(Exp::Max(
                exps.iter()
                    .map(|exp| exp.into_exp(context))
                    .collect::<Result<Vec<Exp>, TransformError>>()?,
            )),
            Self::UnaryNegation(exp) => Ok(Exp::Neg(exp.into_exp(context)?.to_box())),
            Self::LenOf(len_of) => {
                let value = transform_len_of(len_of, context)?;
                Ok(Exp::Number(value as f64))
            }
            Self::Variable(name) => {
                let value = context.get_variable(name);
                match value {
                    //try to see if the variable is a constant, else return the variable name
                    Some(value) => Ok(Exp::Number(*value)),
                    None => Ok(Exp::Variable(name.clone())),
                }
            }
            //Self::Parenthesis(exp) => Ok(Exp::Parenthesis(exp.into_exp(context)?.to_boxed())),
            Self::CompoundVariable { name, indexes } => {
                let parsed_indexes = context.flatten_variable_name(&indexes)?;
                Ok(Exp::Variable(format!("{}_{}", name, parsed_indexes)))
            }
            Self::ArrayAccess(array_access) => {
                let value = transform_pre_array_access(array_access, context)?;
                Ok(Exp::Number(value))
            }
            Self::Sum(ranges, exp_body) => {
                /* sum(i in 0..2, j in 0..2) { X_ij } becomes:
                x_0_0 + x_0_1 + x_0_2 + x_1_0 + x_1_1 + x_1_2 + x_2_0 + x_2_1 + x_2_2
                */
                let ranges = ranges
                    .iter()
                    .map(|r| transform_set(r, context))
                    .collect::<Result<Vec<Range>, TransformError>>()?;
                let mut results = Vec::new();
                recursive_sum_resolver(exp_body, &ranges, context, &mut results, 0)?;
                results.reverse();
                let mut sum = results.pop().unwrap_or(Exp::Number(0.0));
                for result in results {
                    sum = Exp::BinOp(Op::Add, sum.to_box(), result.to_box());
                }
                //Ok(Exp::Parenthesis(sum.to_boxed()))
                Ok(sum)
            }
        };
        exp.map(|e| e.flatten())
    }
}

fn recursive_sum_resolver(
    exp: &PreExp,
    ranges: &Vec<Range>,
    context: &mut TransformerContext,
    results: &mut Vec<Exp>,
    current_level: usize,
) -> Result<(), TransformError> {
    if current_level == ranges.len() {
        results.push(exp.into_exp(context)?);
        return Ok(());
    }
    let range = match ranges.get(current_level) {
        Some(range) => range,
        None => return Err(TransformError::OutOfBounds("Range".to_string())),
    };
    let mut current_value = range.from;
    while current_value < range.to {
        //range is exclusive
        context.add_variable(&range.name, current_value as f64)?;
        recursive_sum_resolver(exp, ranges, context, results, current_level + 1)?;
        context.remove_variable(&range.name)?;
        current_value += 1;
    }
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
    pub iteration: Option<PreSet>,
}

impl PreCondition {
    pub fn new(
        lhs: PreExp,
        condition_type: Comparison,
        rhs: PreExp,
        iteration: Option<PreSet>,
    ) -> Self {
        Self {
            lhs,
            condition_type,
            rhs,
            iteration,
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
                Err(err) => Err(format!("{:#?}", err)),
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
        _ => err_missing_token!("Objective and conditions are required", problem),
    }
}
