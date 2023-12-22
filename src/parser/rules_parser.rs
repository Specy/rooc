use std::vec;

use pest::iterators::{Pair, Pairs};

use crate::math::math_enums::{Comparison, OptimizationType};
use crate::math::operators::Op;
use crate::parser::iterable_utils::flatten_primitive_array_values;
use crate::primitives::consts::Constant;
use crate::primitives::functions::array_functions::{EnumerateArray, LenOfIterableFn};
use crate::primitives::functions::function_traits::FunctionCall;
use crate::primitives::functions::graph_functions::{
    EdgesOfGraphFn, NeighbourOfNodeFn, NeighboursOfNodeInGraphFn, NodesOfGraphFn,
};
use crate::primitives::functions::number_functions::NumericRange;
use crate::primitives::graph::{Graph, GraphEdge, GraphNode};
use crate::primitives::primitive::Primitive;
use crate::utils::{CompilationError, InputSpan, ParseError, Spanned};
use crate::{bail_missing_token, err_unexpected_token};

use super::parser::Rule;
use super::pre_parsed_problem::{
    ArrayAccess, BlockFunction, BlockFunctionKind, BlockScopedFunction, BlockScopedFunctionKind,
    CompoundVariable, IterableSet, PreCondition, PreExp, PreObjective,
};
use super::transformer::VariableType;

pub fn parse_objective(objective: Pair<Rule>) -> Result<PreObjective, CompilationError> {
    match objective.as_rule() {
        Rule::objective => {
            let pairs = objective.clone().into_inner();
            let objective_type = pairs.find_first_tagged("objective_type");
            let objective_body = pairs.find_first_tagged("objective_body");
            if objective_type.is_none() {
                return bail_missing_token!("Missing min/max in objective", objective);
            }
            let objective_type = objective_type.unwrap().as_str();
            let objective_type = match objective_type {
                "min" => OptimizationType::Min,
                "max" => OptimizationType::Max,
                _ => {
                    return err_unexpected_token!("found {}, expected min/max", objective);
                }
            };
            match objective_body {
                Some(rhs) => Ok(PreObjective::new(objective_type, parse_exp(&rhs)?)),
                None => bail_missing_token!("Missing objective body", objective),
            }
        }
        _ => {
            return err_unexpected_token!("found {}, expected objective", objective);
        }
    }
}

pub fn parse_consts_declaration(
    consts_declarations: Pair<Rule>,
) -> Result<Vec<Constant>, CompilationError> {
    match consts_declarations.as_rule() {
        Rule::consts_declaration => consts_declarations
            .into_inner()
            .map(|c| parse_const_declaration(c))
            .collect(),
        _ => err_unexpected_token!(
            "Expected consts declaration but got: {}",
            consts_declarations
        ),
    }
}

pub fn parse_const_declaration(
    const_declaration: Pair<Rule>,
) -> Result<Constant, CompilationError> {
    match const_declaration.as_rule() {
        Rule::const_declaration => {
            let pairs = const_declaration.clone().into_inner();
            let name = pairs
                .find_first_tagged("name")
                .map(|n| n.as_str().to_string());
            let value = pairs
                .find_first_tagged("value")
                .map(|v| parse_primitive(&v));
            match (name, value) {
                (Some(name), Some(value)) => Ok(Constant::new(name, value?)),
                _ => bail_missing_token!("Missing constant body", const_declaration),
            }
        }
        _ => err_unexpected_token!(
            "Expected constant declaration but got: {}",
            const_declaration
        ),
    }
}

pub fn parse_primitive(const_value: &Pair<Rule>) -> Result<Primitive, CompilationError> {
    match const_value.as_rule() {
        Rule::primitive => {
            let mut inner = const_value.clone().into_inner();
            let inner = inner.next();
            match inner {
                Some(inner) => parse_primitive(&inner),
                None => err_unexpected_token!("Expected constant value but got: {}", const_value),
            }
        }
        Rule::number => Ok(Primitive::Number(parse_number(const_value)?)),
        Rule::boolean => {
            let value = match const_value.as_str() {
                "true" => true,
                "false" => false,
                _ => {
                    return err_unexpected_token!("Expected boolean but got: {}", const_value);
                }
            };
            Ok(Primitive::Boolean(value))
        }
        Rule::string => {
            let value = const_value.as_str().to_string();
            Ok(Primitive::String(value))
        }
        Rule::array => {
            let values = const_value
                .clone()
                .into_inner()
                .map(|v| parse_primitive(&v))
                .collect::<Result<Vec<_>, CompilationError>>()?;
            let values = flatten_primitive_array_values(values).map_err(|e| {
                CompilationError::from_pair(ParseError::UnexpectedToken(e), const_value, false)
            })?;
            Ok(values)
        }
        Rule::graph => {
            let inner = const_value.clone().into_inner();
            let body = inner.find_first_tagged("body");
            match body {
                Some(b) => {
                    let inner = b
                        .into_inner()
                        .into_iter()
                        .map(|n| parse_graph_node(&n))
                        .collect::<Result<Vec<GraphNode>, CompilationError>>()?;
                    let graph = Graph::new(inner);
                    Ok(Primitive::Graph(graph))
                }
                None => err_unexpected_token!("Expected graph but got: {}", const_value),
            }
        }
        _ => err_unexpected_token!("Expected constant value but got: {}", const_value),
    }
}

pub fn parse_graph_node(node: &Pair<Rule>) -> Result<GraphNode, CompilationError> {
    let inner = node.clone().into_inner();
    let name = inner.find_first_tagged("name");
    let edges = inner.find_first_tagged("edges");
    match (name, edges) {
        (Some(name), Some(edges)) => {
            let name = name.as_str().to_string();
            let edges = edges
                .into_inner()
                .map(|e| parse_graph_edge(&e, &name))
                .collect::<Result<Vec<GraphEdge>, CompilationError>>()?;
            Ok(GraphNode::new(name, edges))
        }
        (Some(name), None) => {
            let name = name.as_str().to_string();
            Ok(GraphNode::new(name, vec![]))
        }
        _ => err_unexpected_token!("Expected graph node but got: {}", node),
    }
}
pub fn parse_graph_edge(edge: &Pair<Rule>, from: &String) -> Result<GraphEdge, CompilationError> {
    let inner = edge.clone().into_inner();
    let node = inner.find_first_tagged("node");
    let cost = match inner.find_first_tagged("cost") {
        Some(cost) => {
            let parsed = cost.as_str().to_string().parse::<f64>();
            if !parsed.is_ok() {
                let error = ParseError::UnexpectedToken(format!(
                    "Expected number but got: {}, error: {}",
                    cost,
                    parsed.unwrap_err()
                ));
                return Err(CompilationError::from_pair(error, &cost, false));
            }
            Some(parsed.unwrap())
        }
        None => None,
    };
    match node {
        Some(node) => {
            let node = node.as_str().to_string();
            Ok(GraphEdge::new(from.clone(), node, cost))
        }
        _ => err_unexpected_token!("Expected graph edge but got: {}", edge),
    }
}

pub fn parse_condition_list(
    condition_list: &Pair<Rule>,
) -> Result<Vec<PreCondition>, CompilationError> {
    match condition_list.as_rule() {
        Rule::condition_list => condition_list
            .clone()
            .into_inner()
            .map(|c| parse_condition(&c))
            .collect(),
        _ => err_unexpected_token!("Expected condition list but got: {}", condition_list),
    }
}
/*
pub fn parse_bounds_list(bounds_list: Pair<Rule>) -> Result<Vec<Bounds>, CompilationError> {
    match bounds_list.as_rule() {
        Rule::bounds_list => bounds_list
            .into_inner()
            .map(|c| parse_bounds(c))
            .collect::<Result<Vec<Bounds>, ParseError>>(),
        _ => Err(ParseError::UnexpectedToken("Expected bounds list".to_string())),
    }
}
fn parse_comma_separated_vars(vars: Pair<Rule>) -> Result<Vec<String>, CompilationError> {
    match vars.as_rule() {
        Rule::comma_separated_vars => vars
            .into_inner()
            .map(|c| parse_var(c))
            .collect::<Result<Vec<String>, ParseError>>(),
        _ => Err(ParseError::UnexpectedToken(
            "Expected comma separated vars".to_string(),
        )),
    }
}
fn parse_var(var: Pair<Rule>) -> Result<String, CompilationError> {
    match var.as_rule() {
        Rule::variable => Ok(var.as_str().to_string()),
        _ => Err(ParseError::UnexpectedToken("Expected var".to_string())),
    }
}
*/

fn parse_condition(condition: &Pair<Rule>) -> Result<PreCondition, CompilationError> {
    match condition.as_rule() {
        Rule::condition => {
            let inner = condition.clone().into_inner();
            let lhs = inner.find_first_tagged("lhs");
            let relation = inner.find_first_tagged("relation");
            let rhs = inner.find_first_tagged("rhs");
            let iteration = inner.find_first_tagged("iteration");
            match (rhs, relation, lhs, iteration) {
                (Some(rhs), Some(relation_type), Some(lhs), iteration) => {
                    let iteration = match iteration {
                        Some(iteration) => parse_set_iterator_list(&iteration.into_inner())?,
                        None => vec![],
                    };
                    Ok(PreCondition::new(
                        parse_exp(&lhs)?,
                        parse_comparison(&relation_type)?,
                        parse_exp(&rhs)?,
                        iteration,
                        InputSpan::from_pair(condition),
                    ))
                }
                _ => bail_missing_token!("Missing condition body", condition),
            }
        }
        _ => err_unexpected_token!("Expected condition but got: {}", condition),
    }
}

fn parse_number(number: &Pair<Rule>) -> Result<f64, CompilationError> {
    match number.as_rule() {
        Rule::number => {
            let number = match number.as_str().parse::<f64>() {
                Ok(number) => number,
                Err(_) => {
                    return err_unexpected_token!("found {}, expected number", number);
                }
            };
            Ok(number)
        }
        _ => err_unexpected_token!("Expected number but got: {}", number),
    }
}

fn parse_comparison(comparison: &Pair<Rule>) -> Result<Comparison, CompilationError> {
    match comparison.as_rule() {
        Rule::comparison => {
            let comparison = match comparison.as_str() {
                "<=" => Comparison::LowerOrEqual,
                ">=" => Comparison::UpperOrEqual,
                "=" => Comparison::Equal,
                _ => {
                    return err_unexpected_token!("found {}, expected comparison", comparison);
                }
            };
            Ok(comparison)
        }
        _ => err_unexpected_token!("Expected comparison but got: {}", comparison),
    }
}

fn parse_exp(exp_to_parse: &Pair<Rule>) -> Result<PreExp, CompilationError> {
    //use shunting yard algorithm to parse expression list into a PreExp tree
    let mut output_queue: Vec<PreExp> = Vec::new();
    let mut operator_stack: Vec<Op> = Vec::new();
    let mut last_token: Option<Rule> = None;
    let exp_list = exp_to_parse.clone().into_inner();
    let mut last_op_span = InputSpan::default();
    for exp in exp_list.into_iter() {
        let rule = exp.as_rule();
        let span = InputSpan::from_pair(&exp);
        match rule {
            Rule::binary_op => {
                last_op_span = span.clone();
                let op = parse_operator(&exp)?;
                while should_unwind(&operator_stack, &op) {
                    match (operator_stack.pop(), output_queue.pop(), output_queue.pop()) {
                        (Some(op), Some(rhs), Some(lhs)) => {
                            let spanned_op = Spanned::new(op, span.clone());
                            output_queue.push(PreExp::BinaryOperation(
                                spanned_op,
                                lhs.to_boxed(),
                                rhs.to_boxed(),
                            ));
                        }
                        _ => {
                            return err_unexpected_token!("found {}, expected exp", exp);
                        }
                    }
                }
            }
            Rule::unary_op => {
                last_op_span = span.clone();
                let op = parse_operator(&exp)?;
                match op {
                    Op::Sub => {
                        output_queue.push(PreExp::Primitive(Spanned::new(
                            Primitive::Number(0.0),
                            span,
                        )));
                    }
                    _ => {
                        return err_unexpected_token!(
                            "Unexpected unary token {}, expected exp",
                            exp
                        );
                    }
                }
                operator_stack.push(op);
            }
            Rule::function => {
                let fun = parse_function_call(&exp)?;
                let fun = englobe_if_multiplied_by_constant(
                    &last_token,
                    &mut output_queue,
                    PreExp::FunctionCall(Spanned::new(fun, span)),
                )?;
                output_queue.push(fun);
            }
            Rule::simple_variable => {
                let variable = exp.as_str().to_string();
                let variable = PreExp::Variable(Spanned::new(variable, span));
                let next =
                    englobe_if_multiplied_by_constant(&last_token, &mut output_queue, variable)?;
                output_queue.push(next);
            }
            Rule::compound_variable => {
                let variable = parse_compound_variable(&exp)?;
                let variable = PreExp::CompoundVariable(Spanned::new(variable, span));
                let next =
                    englobe_if_multiplied_by_constant(&last_token, &mut output_queue, variable)?;
                output_queue.push(next);
            }
            Rule::block_function => {
                let inner = exp.clone().into_inner();
                let name = inner.find_first_tagged("name");
                let body = inner.find_first_tagged("body");
                if name.is_none() || body.is_none() {
                    return err_unexpected_token!("found {}, expected block function", exp);
                }
                let members = body
                    .unwrap()
                    .into_inner()
                    .map(|member| parse_exp(&member))
                    .collect::<Result<Vec<PreExp>, CompilationError>>()?;
                let kind = parse_block_function_type(&name.unwrap())?;
                let fun = BlockFunction::new(kind, members);
                let fun_exp = PreExp::BlockFunction(Spanned::new(fun, span));
                let sum =
                    englobe_if_multiplied_by_constant(&last_token, &mut output_queue, fun_exp)?;
                output_queue.push(sum);
            }
            Rule::block_scoped_function => {
                let inner = exp.clone().into_inner();
                let name = inner.find_first_tagged("name");
                let iters = inner.find_first_tagged("range");
                let body = inner.find_first_tagged("body");
                if name.is_none() || iters.is_none() || body.is_none() {
                    return err_unexpected_token!("found {}, expected scoped block function", exp);
                }
                let body = body.unwrap();
                let body = parse_exp(&body)?.to_boxed();
                let iters = iters.unwrap();
                let iters = parse_set_iterator_list(&iters.into_inner())?;
                let kind = parse_scoped_block_function_type(&name.unwrap())?;
                let fun = BlockScopedFunction::new(kind, iters, body);
                let fun_exp = PreExp::BlockScopedFunction(Spanned::new(fun, span));
                let sum =
                    englobe_if_multiplied_by_constant(&last_token, &mut output_queue, fun_exp)?;
                output_queue.push(sum);
            }
            Rule::primitive => {
                let prim = parse_primitive(&exp)?;
                let prim = Spanned::new(prim, span);
                output_queue.push(PreExp::Primitive(prim));
            }
            Rule::parenthesis => {
                let par = parse_exp(&exp)?;
                let par = englobe_if_multiplied_by_constant(&last_token, &mut output_queue, par)?;
                output_queue.push(par);
            }
            Rule::modulo => {
                let exp = parse_exp(&exp)?;
                let modulo = PreExp::Mod(Spanned::new(Box::new(exp), span));
                let modulo =
                    englobe_if_multiplied_by_constant(&last_token, &mut output_queue, modulo)?;
                output_queue.push(modulo)
            }
            Rule::array_access => {
                let array_access = parse_array_access(&exp)?;
                let array_access = englobe_if_multiplied_by_constant(
                    &last_token,
                    &mut output_queue,
                    PreExp::ArrayAccess(Spanned::new(array_access, span)),
                )?;
                output_queue.push(array_access);
            }
            _ => {
                println!("exp: {:#?}", exp);
                return err_unexpected_token!(
                    "found \"{}\", expected exp, binary_op, unary_op, len, variable, sum, primitive, parenthesis, array_access, min, max, block function or scoped function", 
                    exp
                );
            }
        }
        last_token = Some(rule);
    }
    while !operator_stack.is_empty() {
        let op = operator_stack.pop();
        let rhs = output_queue.pop();
        let lhs = output_queue.pop();
        if op.is_none() || rhs.is_none() || lhs.is_none() {
            return bail_missing_token!("missing terminal expression", exp_to_parse);
        }
        let (op, rhs, lhs) = (op.unwrap(), rhs.unwrap(), lhs.unwrap());
        output_queue.push(PreExp::BinaryOperation(
            Spanned::new(op, last_op_span.clone()),
            lhs.to_boxed(),
            rhs.to_boxed(),
        ));
    }
    match output_queue.pop() {
        Some(exp) => Ok(exp),
        None => err_unexpected_token!("Invalid empty expression: {}", exp_to_parse),
    }
}

fn parse_set_iterator_list(range_list: &Pairs<Rule>) -> Result<Vec<IterableSet>, CompilationError> {
    range_list
        .clone()
        .map(|r| parse_set_iterator(&r))
        .collect::<Result<Vec<IterableSet>, CompilationError>>()
}

fn parse_compound_variable(
    compound_variable: &Pair<Rule>,
) -> Result<CompoundVariable, CompilationError> {
    match compound_variable.as_rule() {
        Rule::compound_variable => {
            let fields = compound_variable.as_str().split("_").collect::<Vec<_>>();
            if fields.len() < 2 {
                return err_unexpected_token!(
                    "found {}, expected compound variable",
                    compound_variable
                );
            }
            let name = fields[0];
            let indexes = fields[1..]
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>();
            Ok(CompoundVariable {
                name: name.to_string(),
                indexes,
            })
        }
        _ => err_unexpected_token!("Expected compound variable but got: {}", compound_variable),
    }
}

pub fn parse_block_function_type(
    block_function_type: &Pair<Rule>,
) -> Result<BlockFunctionKind, CompilationError> {
    match block_function_type.as_str() {
        "max" => Ok(BlockFunctionKind::Max),
        "min" => Ok(BlockFunctionKind::Min),
        "avg" => Ok(BlockFunctionKind::Avg),
        _ => Err(CompilationError::from_pair(
            ParseError::SemanticError(format!(
                "Unknown block function \"{}\", expected one of \"{}\"",
                block_function_type.as_str(),
                BlockFunctionKind::kinds_to_string().join(", ")
            )),
            &block_function_type,
            false,
        )),
    }
}
pub fn parse_scoped_block_function_type(
    scoped_block_function_type: &Pair<Rule>,
) -> Result<BlockScopedFunctionKind, CompilationError> {
    match scoped_block_function_type.as_str() {
        "sum" => Ok(BlockScopedFunctionKind::Sum),
        "prod" => Ok(BlockScopedFunctionKind::Prod),
        "min" => Ok(BlockScopedFunctionKind::Min),
        "max" => Ok(BlockScopedFunctionKind::Max),
        "avg" => Ok(BlockScopedFunctionKind::Avg),
        _ => Err(CompilationError::from_pair(
            ParseError::SemanticError(format!(
                "Unknown scoped block function \"{}\", expected one of \"{}\"",
                scoped_block_function_type.as_str(),
                BlockScopedFunctionKind::kinds_to_string().join(", ")
            )),
            &scoped_block_function_type,
            true,
        )),
    }
}
fn parse_function_call(
    function_call: &Pair<Rule>,
) -> Result<Box<dyn FunctionCall>, CompilationError> {
    match function_call.as_rule() {
        Rule::function => {
            let inner = function_call.clone().into_inner();
            let name = inner.find_first_tagged("function_name");
            let args = inner.find_first_tagged("function_pars");
            match (name, args) {
                (Some(name), Some(args)) => Ok(parse_function(&name, args)?),
                _ => err_unexpected_token!("Expected function call but got: {}", function_call),
            }
        }
        _ => err_unexpected_token!("Expected function call but got: {}", function_call),
    }
}

fn parse_function(
    name: &Pair<Rule>,
    pars: Pair<Rule>,
) -> Result<Box<dyn FunctionCall>, CompilationError> {
    let parsed_pars = parse_parameters(&pars)?;
    match name.as_str() {
        "edges" => Ok(Box::new(EdgesOfGraphFn::from_parameters(
            parsed_pars,
            &pars,
        )?)),
        "len" => Ok(Box::new(LenOfIterableFn::from_parameters(
            parsed_pars,
            &pars,
        )?)),
        "nodes" => Ok(Box::new(NodesOfGraphFn::from_parameters(
            parsed_pars,
            &pars,
        )?)),
        "neigh_edges" => Ok(Box::new(NeighbourOfNodeFn::from_parameters(
            parsed_pars,
            &pars,
        )?)),
        "neighs_of" => Ok(Box::new(NeighboursOfNodeInGraphFn::from_parameters(
            parsed_pars,
            &pars,
        )?)),
        "enumerate" => Ok(Box::new(EnumerateArray::from_parameters(
            parsed_pars,
            &pars,
        )?)),
        str => Err(CompilationError::from_pair(
            ParseError::SemanticError(format!("Unknown function {}", str)),
            &name,
            true,
        )),
    }
}

fn parse_parameters(pars: &Pair<Rule>) -> Result<Vec<PreExp>, CompilationError> {
    match pars.as_rule() {
        Rule::function_pars => {
            let inner = pars.clone().into_inner();
            let pars = inner
                .map(|a| parse_parameter(&a))
                .collect::<Result<Vec<PreExp>, CompilationError>>()?;
            Ok(pars)
        }
        _ => err_unexpected_token!("Expected function args but got: {}", pars),
    }
}
fn parse_parameter(arg: &Pair<Rule>) -> Result<PreExp, CompilationError> {
    let span = InputSpan::from_pair(&arg);
    match arg.as_rule() {
        Rule::tagged_exp => parse_exp(&arg),
        _ => err_unexpected_token!("Expected function arg but got: {}", arg),
    }
}
fn parse_set_iterator(range: &Pair<Rule>) -> Result<IterableSet, CompilationError> {
    match range.as_rule() {
        Rule::iteration_declaration => {
            let inner = range.clone().into_inner();
            let vars_tuple = inner
                .find_first_tagged("tuple")
                .map(|t| parse_variable_type(&t));
            let iterator = inner.find_first_tagged("iterator").map(|f| {
                let span = InputSpan::from_pair(&range);
                parse_iterator(&f).map(|i| Spanned::new(i, span))
            });
            match (vars_tuple, iterator) {
                (Some(vars_tuple), Some(iterator)) => {
                    let span = InputSpan::from_pair(&range);
                    Ok(IterableSet::new(vars_tuple?, iterator?, span))
                }
                _ => err_unexpected_token!("Expected set iterator but got: {}", range),
            }
        }
        _ => err_unexpected_token!("Expected set iterator but got: {}", range),
    }
}
fn parse_variable_type(tuple: &Pair<Rule>) -> Result<VariableType, CompilationError> {
    match tuple.as_rule() {
        Rule::tuple => {
            let inner = tuple.clone().into_inner();
            let inner = inner
                .map(|i| {
                    let span = InputSpan::from_pair(&tuple);
                    match i.as_rule() {
                        Rule::simple_variable | Rule::no_par => {
                            Ok(Spanned::new(i.as_str().to_string(), span))
                        }
                        _ => err_unexpected_token!("Expected variable but got: {}", i),
                    }
                })
                .collect::<Result<Vec<_>, CompilationError>>()?;
            Ok(VariableType::Tuple(inner))
        }
        Rule::simple_variable => {
            let span = InputSpan::from_pair(&tuple);
            Ok(VariableType::Single(Spanned::new(
                tuple.as_str().to_string(),
                span,
            )))
        }
        _ => err_unexpected_token!("Expected tuple but got: {}", tuple),
    }
}

fn parse_iterator(iterator: &Pair<Rule>) -> Result<PreExp, CompilationError> {
    match iterator.as_rule() {
        Rule::iterator => {
            let mut inner = iterator.clone().into_inner();
            let first: Option<Rule> = inner.next().map(|i| i.as_rule());
            match first {
                Some(Rule::range_iterator) => {
                    let inner = iterator.clone().into_inner();
                    let from = inner.find_first_tagged("from").map(|f| parse_parameter(&f));
                    let to = inner.find_first_tagged("to").map(|t| parse_parameter(&t));
                    let range_type = inner.find_first_tagged("range_type");
                    match (from, to, range_type) {
                        (Some(from), Some(to), Some(range_type)) => {
                            let to_inclusive = match range_type.as_str() {
                                ".." => false,
                                "..=" => true,
                                _ => {
                                    return err_unexpected_token!(
                                        "Expected range type but got: {}",
                                        range_type
                                    );
                                }
                            };
                            let span = InputSpan::from_pair(&iterator);
                            let function = NumericRange::new(from?, to?, to_inclusive);
                            Ok(PreExp::FunctionCall(Spanned::new(Box::new(function), span)))
                        }

                        _ => err_unexpected_token!("Expected range iterator but got: {}", iterator),
                    }
                }
                Some(Rule::tagged_exp) => {
                    let mut inner = iterator.clone().into_inner();
                    let first = inner.next();
                    if first.is_none() {
                        return err_unexpected_token!("Expected parameter but got: {}", iterator);
                    }
                    let function = parse_parameter(&first.unwrap())?;
                    Ok(function)
                }

                _ => err_unexpected_token!("Expected range or parameter but got: {}", iterator),
            }
        }
        _ => err_unexpected_token!("Expected iterator but got: {}", iterator),
    }
}

fn parse_array_access(array_access: &Pair<Rule>) -> Result<ArrayAccess, CompilationError> {
    match array_access.as_rule() {
        Rule::array_access => {
            let inner = array_access.clone().into_inner();
            let name = inner
                .find_first_tagged("name")
                .map(|n| n.as_str().to_string());
            let accesses = inner.find_first_tagged("accesses");
            if name.is_none() || accesses.is_none() {
                return err_unexpected_token!("Expected array access but got: {}", array_access);
            }
            let name = name.unwrap();
            let accesses = accesses.unwrap();
            let accesses = accesses
                .into_inner()
                .map(|a| parse_parameter(&a))
                .collect::<Result<Vec<_>, CompilationError>>()?;
            Ok(ArrayAccess::new(name, accesses))
        }
        _ => err_unexpected_token!("Expected array access but got: {}", array_access),
    }
}

fn should_unwind(operator_stack: &Vec<Op>, op: &Op) -> bool {
    match operator_stack.last() {
        Some(top) => top.precedence() >= op.precedence(),
        None => false,
    }
}

fn parse_operator(operator: &Pair<Rule>) -> Result<Op, CompilationError> {
    match operator.as_rule() {
        //TODO add separate unary operators?
        Rule::binary_op | Rule::unary_op => match operator.as_str() {
            "+" => Ok(Op::Add),
            "-" => Ok(Op::Sub),
            "*" => Ok(Op::Mul),
            "/" => Ok(Op::Div),
            _ => err_unexpected_token!("found {}, expected +, -, *, /", operator),
        },
        _ => err_unexpected_token!("found {}, expected op", operator),
    }
}

//if the previous token was a number, englobe the rhs in a multiplication, this is to implement the implicit multiplication
fn englobe_if_multiplied_by_constant(
    prev_token: &Option<Rule>,
    queue: &mut Vec<PreExp>,
    rhs: PreExp,
) -> Result<PreExp, CompilationError> {
    match prev_token {
        Some(Rule::number) => {
            let last_number = match queue.pop() {
                Some(n @ PreExp::Primitive(_)) => n,
                _ => unreachable!(), //could this ever happen?
            };
            let span = rhs.get_span();
            let exp = PreExp::BinaryOperation(
                Spanned::new(Op::Mul, span.clone()),
                last_number.to_boxed(),
                rhs.to_boxed(),
            );
            Ok(exp)
        }
        _ => Ok(rhs),
    }
}
