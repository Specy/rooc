use pest::iterators::{Pair, Pairs};

use crate::consts::{
    Comparison, CompilationError, Constant, ConstantValue, Operator, OptimizationType, ParseError,
};
use crate::parser::{
    PreAccess, PreArrayAccess, PreCondition, PreExp, PreIterOfArray, PreIterator, PreNode,
    PreObjective, PreRangeValue, Rule, PreSet,
};
use crate::transformer::{Graph, GraphEdge, GraphNode};
use crate::{err_missing_token, err_semantic_error, err_unexpected_token};

//done
pub fn parse_objective(objective: Pair<Rule>) -> Result<PreObjective, CompilationError> {
    match objective.as_rule() {
        Rule::objective => {
            let pairs = objective.clone().into_inner();
            let objective_type = pairs.find_first_tagged("objective_type");
            let objective_body = pairs.find_first_tagged("objective_body");
            if objective_type.is_none() {
                return err_missing_token!("Missing min/max in objective", objective);
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
                Some(rhs) => Ok(PreObjective::new(objective_type, parse_exp_list(&rhs)?)),
                None => err_missing_token!("Missing objective body", objective),
            }
        }
        _ => {
            return err_unexpected_token!("found {}, expected objective", objective);
        }
    }
}
//done
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
//done
pub fn parse_const_declaration(
    const_declaration: Pair<Rule>,
) -> Result<Constant, CompilationError> {
    match const_declaration.as_rule() {
        Rule::const_declaration => {
            let str = const_declaration.as_str();
            let pairs = const_declaration.clone().into_inner();
            let name = pairs
                .find_first_tagged("name")
                .map(|n| n.as_str().to_string());
            let value = pairs
                .find_first_tagged("value")
                .map(|v| parse_const_value(&v));
            if name.is_none() || value.is_none() {
                return err_unexpected_token!(
                    "Expected constant declaration but got: {}",
                    const_declaration
                );
            }
            let name = name.unwrap();
            let value = value.unwrap()?;
            Ok(Constant::new(name, value))
        }
        _ => err_unexpected_token!(
            "Expected constant declaration but got: {}",
            const_declaration
        ),
    }
}
//done
pub fn parse_const_value(const_value: &Pair<Rule>) -> Result<ConstantValue, CompilationError> {
    match const_value.as_rule() {
        Rule::number => Ok(ConstantValue::Number(parse_number(const_value)?)),
        Rule::array => {
            let values = const_value
                .clone()
                .into_inner()
                .map(|v| parse_array_value(&v))
                .collect::<Result<Vec<ArrayValue>, CompilationError>>()?;
            //make sure they are all the same type

            let first = values.first();
            if first.is_none() {
                return Ok(ConstantValue::OneDimArray(vec![]));
            }
            match first.unwrap() {
                ArrayValue::Number(_) => {
                    let values = values
                        .into_iter()
                        .map(|v| match v {
                            ArrayValue::Number(n) => Ok(n),
                            ArrayValue::Array(_) | ArrayValue::Empty => {
                                err_semantic_error!("Expected number but got an array", const_value)
                            }
                        })
                        .collect::<Result<Vec<f64>, CompilationError>>()?;
                    Ok(ConstantValue::OneDimArray(values))
                }
                ArrayValue::Array(_) => {
                    let values = values
                        .into_iter()
                        .map(|v| match v {
                            ArrayValue::Array(n) => Ok(n),
                            ArrayValue::Number(_) | ArrayValue::Empty => {
                                err_semantic_error!("Expected array but got a number", const_value)
                            }
                        })
                        .collect::<Result<Vec<Vec<f64>>, CompilationError>>()?;
                    //make sure all the arrays are the same length
                    let lengths = values.iter().map(|v| v.len()).collect::<Vec<usize>>();
                    let first = lengths.first();
                    if first.is_none() {
                        return Ok(ConstantValue::TwoDimArray(vec![]));
                    }
                    let max = lengths.iter().max().unwrap();
                    if lengths.iter().any(|l| l != max) {
                        let error = ParseError::SemanticError(format!(
                            "Arrays must all have the same size, using the biggest one of: {}",
                            max
                        ));
                        return Err(CompilationError::from_span(
                            error,
                            &const_value.as_span(),
                            false,
                        ));
                    }
                    Ok(ConstantValue::TwoDimArray(values))
                }
                ArrayValue::Empty => Ok(ConstantValue::OneDimArray(vec![])),
            }
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
                    Ok(ConstantValue::Graph(graph))
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
                return Err(CompilationError::from_span(error, &cost.as_span(), false));
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

//simplified as it only goes up to 2d arrays
#[derive(Debug)]
pub enum ArrayValue {
    Number(f64),
    Array(Vec<f64>),
    Empty,
}
//done
pub fn parse_array_value(array_value: &Pair<Rule>) -> Result<ArrayValue, CompilationError> {
    let pairs = array_value
        .clone()
        .into_inner()
        .collect::<Vec<Pair<Rule>>>();
    match array_value.as_rule() {
        Rule::number => Ok(ArrayValue::Number(parse_number(array_value)?)),
        Rule::array => {
            let values = pairs
                .into_iter()
                .map(|v| parse_number(&v))
                .collect::<Result<Vec<f64>, CompilationError>>()?;
            if values.is_empty() {
                return Ok(ArrayValue::Empty);
            }
            Ok(ArrayValue::Array(values))
        }
        _ => err_unexpected_token!("Expected array value but got: {}", array_value),
    }
}
//done
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
//done
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
                        Some(iteration) => Some(parse_set_iterator(&iteration)?),
                        None => None,
                    };
                    Ok(PreCondition::new(
                        parse_exp_list(&lhs)?,
                        parse_comparison(&relation_type)?,
                        parse_exp_list(&rhs)?,
                        iteration,
                    ))
                }
                _ => err_missing_token!("Missing condition body", condition),
            }
        }
        _ => err_unexpected_token!("Expected condition but got: {}", condition),
    }
}

/*
fn parse_bounds(bounds: Pair<Rule>) -> Result<Bounds, CompilationError> {
    match bounds.as_rule() {
        Rule::bounds => {
            let mut inner = bounds.into_inner();
            let (vars, bounds_type, rhs) = (
                inner.next(),
                inner.next(),
                inner.next(),
            );
            match (vars, bounds_type, rhs) {
                (Some(vars), Some(bounds_type), Some(rhs)) => Ok(Bounds::new(
                    parse_comma_separated_vars(vars)?,
                    parse_comparison(bounds_type)?,
                    parse_number(rhs)?
                )),
                _ => Err(ParseError::UnexpectedToken(
                    "Missing bounds".to_string(),
                )),
            }
        }
        _ => Err(ParseError::UnexpectedToken("Expected bounds".to_string())),
    }
}
*/
//done
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

//done
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
//done
fn parse_exp_list(exp_to_parse: &Pair<Rule>) -> Result<PreExp, CompilationError> {
    //use shunting yard algorithm to parse expression list into a PreExp tree
    let mut output_queue: Vec<PreExp> = Vec::new();
    let mut operator_stack: Vec<Operator> = Vec::new();
    let mut last_token: Option<Rule> = None;
    let exp_list = exp_to_parse.clone().into_inner();
    for exp in exp_list.into_iter() {
        let rule = exp.as_rule();
        match rule {
            Rule::binary_op | Rule::unary_op => {
                let op = parse_operator(&exp)?;

                while should_unwind(&operator_stack, &op) {
                    let op = operator_stack.pop();
                    let rhs = output_queue.pop();
                    let lhs = output_queue.pop();
                    if op.is_none() || rhs.is_none() || lhs.is_none() {
                        return err_unexpected_token!("found {}, expected exp", exp);
                    }
                    let (op, rhs, lhs) = (op.unwrap(), rhs.unwrap(), lhs.unwrap());
                    output_queue.push(PreExp::BinaryOperation(op, lhs.to_boxed(), rhs.to_boxed()));
                }
                //check if the operator is unary, if so, add a zero to the output queue
                //old if last_token == Some(Rule::op) || last_token == None
                //TODO prove this works
                if rule == Rule::unary_op {
                    match op {
                        Operator::Sub => {
                            output_queue.push(PreExp::Number(0.0));
                        }
                        _ => {
                            return err_unexpected_token!(
                                "Unexpected unary token {}, expected exp",
                                exp
                            );
                        }
                    }
                }
                operator_stack.push(op);
            }
            Rule::len => {
                let len = parse_len(&exp)?;
                let len = englobe_if_multiplied_by_constant(
                    &last_token,
                    &mut output_queue,
                    PreExp::LenOf(len),
                );
                output_queue.push(len);
            }
            Rule::simple_variable => {
                let variable = PreExp::Variable(exp.as_str().to_string());
                let next =
                    englobe_if_multiplied_by_constant(&last_token, &mut output_queue, variable);
                output_queue.push(next);
            }
            Rule::compound_variable => {
                let fields = exp.as_str().split("_").collect::<Vec<&str>>();
                if fields.len() < 2 {
                    return err_unexpected_token!("found {}, expected compound variable", exp);
                }
                let (name, indexes) = (fields[0], fields[1..].join("_"));
                let variable = PreExp::CompoundVariable {
                    name: name.to_string(),
                    indexes,
                };
                let next =
                    englobe_if_multiplied_by_constant(&last_token, &mut output_queue, variable);
                output_queue.push(next);
            }
            Rule::sum => {
                let inner = exp.clone().into_inner();
                let range = inner
                    .find_first_tagged("range")
                    .map(|r| parse_set_iterator_list(&r.into_inner()));
                let body = inner.find_first_tagged("body").map(|b| parse_exp_list(&b));
                if range.is_none() || body.is_none() {
                    return err_unexpected_token!("found {}, expected sum", exp);
                }
                let range = range.unwrap()?;
                let body = body.unwrap()?;
                let sum = PreExp::Sum(range, body.to_boxed());
                let sum = englobe_if_multiplied_by_constant(&last_token, &mut output_queue, sum);
                output_queue.push(sum);
            }
            Rule::number => {
                let num = exp.as_str().parse::<f64>();
                match num {
                    Ok(num) => output_queue.push(PreExp::Number(num)),
                    Err(_) => {
                        return err_unexpected_token!("found {}, expected number", exp);
                    }
                }
            }
            Rule::parenthesis => {
                let par = parse_exp_list(&exp)?;
                let par = englobe_if_multiplied_by_constant(&last_token, &mut output_queue, par);
                output_queue.push(PreExp::Parenthesis(par.to_boxed()));
            }
            Rule::modulo => {
                let exp = parse_exp_list(&exp)?;
                let modulo = PreExp::Mod(Box::new(exp));
                let modulo =
                    englobe_if_multiplied_by_constant(&last_token, &mut output_queue, modulo);
                output_queue.push(modulo)
            }
            Rule::array_access => {
                let array_access = parse_array_access(&exp)?;
                let array_access = englobe_if_multiplied_by_constant(
                    &last_token,
                    &mut output_queue,
                    PreExp::ArrayAccess(array_access),
                );
                output_queue.push(array_access);
            }
            Rule::min | Rule::max => {
                let members = exp
                    .into_inner()
                    .map(|member| parse_exp_list(&member))
                    .collect::<Result<Vec<PreExp>, CompilationError>>()?;
                let fun = match rule {
                    Rule::min => PreExp::Min(members),
                    Rule::max => PreExp::Max(members),
                    _ => unreachable!(),
                };

                let fun = englobe_if_multiplied_by_constant(&last_token, &mut output_queue, fun);
                output_queue.push(fun);
            }
            _ => {
                return err_unexpected_token!("found {}, expected exp, binary_op, unary_op, len, variable, sum, number, parenthesis, array_access, min or max", exp);
            }
        }
        last_token = Some(rule);
    }
    while !operator_stack.is_empty() {
        let op = operator_stack.pop();
        let rhs = output_queue.pop();
        let lhs = output_queue.pop();
        if op.is_none() || rhs.is_none() || lhs.is_none() {
            return err_missing_token!("missing terminal expression", exp_to_parse);
        }
        let (op, rhs, lhs) = (op.unwrap(), rhs.unwrap(), lhs.unwrap());
        output_queue.push(PreExp::BinaryOperation(op, lhs.to_boxed(), rhs.to_boxed()));
    }
    match output_queue.pop() {
        Some(exp) => Ok(exp),
        None => err_unexpected_token!("Invalid empty expression: {}", exp_to_parse),
    }
}

//done
fn parse_set_iterator_list(range_list: &Pairs<Rule>) -> Result<Vec<PreSet>, CompilationError> {
    range_list
        .clone()
        .map(|r| parse_set_iterator(&r))
        .collect::<Result<Vec<PreSet>, CompilationError>>()
}

//done
fn parse_set_iterator(range: &Pair<Rule>) -> Result<PreSet, CompilationError> {
    match range.as_rule() {
        Rule::iteration_declaration => {
            let inner = range.clone().into_inner();
            let name = inner
                .find_first_tagged("name")
                .map(|n| n.as_str().to_string());
            let iterator = inner
                .find_first_tagged("iterator")
                .map(|f| parse_iterator(&f));
            match (name, iterator) {
                (Some(name), Some(iterator)) => {
                    Ok(PreSet::new(name, iterator?))
                }
                _ => err_unexpected_token!("Expected set iterator but got: {}", range),
            }
        }
        _ => err_unexpected_token!("Expected set iterator but got: {}", range),
    }
}
fn parse_iterator(iterator: &Pair<Rule>) -> Result<PreIterator, CompilationError> {
    match iterator.as_rule() {
        Rule::for_iteration => {
            let inner = iterator.clone().into_inner();
            let from = inner
                .find_first_tagged("from")
                .map(|f| parse_range_value(&f));
            let to = inner.find_first_tagged("to").map(|t| parse_range_value(&t));
            match (from, to) {
                (Some(from), Some(to)) => Ok(PreIterator::Range {
                    from: from?,
                    to: to?,
                }),

                _ => err_unexpected_token!("Expected range but got: {}", iterator),
            }
        }
        Rule::edges_iterator => {
            let inner = iterator.clone().into_inner();
            let graph_name = inner.find_first_tagged("of_graph");
            match graph_name {
                Some(graph_name) => Ok(PreIterator::EdgesOf {
                    graph_name: graph_name.as_str().to_string(),
                }),
                None => err_unexpected_token!("Expected range but got: {}", iterator),
            }
        }
        Rule::neighbour_iterator => {
            let inner = iterator.clone().into_inner();
            let graph_name = inner.find_first_tagged("of_graph");
            let node = inner.find_first_tagged("node").map(|n| parse_node(&n));
            match node {
                Some(node) => Ok(PreIterator::NeighboursOf {
                    graph_name: graph_name.unwrap().as_str().to_string(),
                    node: node?,
                }),
                None => err_unexpected_token!("Expected range but got: {}", iterator),
            }
        }
        Rule::iter_iterator => {
            let of = iterator.clone().into_inner().find_first_tagged("of");
            match of {
                Some(of) => Ok(PreIterator::IterOf {
                    of: parse_array_iter(&of)?,
                }),
                None => err_unexpected_token!("Expected range but got: {}", iterator),
            }
        }
        _ => err_unexpected_token!("Expected range but got: {}", iterator),
    }
}

fn parse_node(node: &Pair<Rule>) -> Result<PreNode, CompilationError> {
    match node.as_rule() {
        Rule::string => {
            let name = node.as_str().to_string();
            Ok(PreNode::Name(name))
        }
        Rule::simple_variable => {
            let name = node.as_str().to_string();
            Ok(PreNode::Variable(name))
        }
        _ => err_unexpected_token!("Expected node but got: {}", node),
    }
}

//done
fn parse_range_value(range_value: &Pair<Rule>) -> Result<PreRangeValue, CompilationError> {
    match range_value.as_rule() {
        Rule::number => Ok(PreRangeValue::Number(
            range_value.as_str().parse::<i32>().unwrap(),
        )),
        Rule::len => Ok(PreRangeValue::LenOf(parse_len(&range_value)?)),
        _ => err_unexpected_token!("Expected range value but got: {}", range_value),
    }
}

//done
fn parse_len(len: &Pair<Rule>) -> Result<PreIterOfArray, CompilationError> {
    match len.as_rule() {
        Rule::len => {
            let inner = len.clone().into_inner();
            let of = inner.find_first_tagged("of");
            if of.is_none() {
                return err_unexpected_token!("Expected len but got: {}", len);
            }
            Ok(parse_array_iter(&of.unwrap())?)
        }
        _ => err_unexpected_token!("Expected len but got: {}", len),
    }
}
fn parse_array_iter(array_access: &Pair<Rule>) -> Result<PreIterOfArray, CompilationError> {
    match array_access.as_rule() {
        Rule::simple_variable => {
            let inner = array_access.clone().into_inner();
            Ok(PreIterOfArray::Array(inner.as_str().to_string()))
        }
        Rule::array_access => {
            let array_access = parse_array_access(&array_access)?;
            Ok(PreIterOfArray::ArrayAccess(array_access))
        }
        _ => err_unexpected_token!("Expected len but got: {}", array_access),
    }
}

//done
fn parse_array_access(array_access: &Pair<Rule>) -> Result<PreArrayAccess, CompilationError> {
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
                .map(|a| parse_pointer_access(&a))
                .collect::<Result<Vec<PreAccess>, CompilationError>>()?;
            Ok(PreArrayAccess { name, accesses })
        }
        _ => err_unexpected_token!("Expected array access but got: {}", array_access),
    }
}

//done
fn parse_pointer_access(pointer_access: &Pair<Rule>) -> Result<PreAccess, CompilationError> {
    match pointer_access.as_rule() {
        Rule::number => Ok(PreAccess::Number(
            pointer_access.as_str().parse::<i32>().unwrap(),
        )),
        Rule::simple_variable => Ok(PreAccess::Variable(pointer_access.as_str().to_string())),
        _ => err_unexpected_token!("Expected pointer access but got: {}", pointer_access),
    }
}
//done
fn should_unwind(operator_stack: &Vec<Operator>, op: &Operator) -> bool {
    match operator_stack.last() {
        Some(top) => top.precedence() >= op.precedence(),
        None => false,
    }
}
//done
fn parse_operator(operator: &Pair<Rule>) -> Result<Operator, CompilationError> {
    match operator.as_rule() {
        //TODO add separate unary operators?
        Rule::binary_op | Rule::unary_op => match operator.as_str() {
            "+" => Ok(Operator::Add),
            "-" => Ok(Operator::Sub),
            "*" => Ok(Operator::Mul),
            "/" => Ok(Operator::Div),
            _ => err_unexpected_token!("found {}, expected +, -, *, /", operator),
        },
        _ => err_unexpected_token!("found {}, expected op", operator),
    }
}
//done
//if the previous token was a number, englobe the rhs in a multiplication, this is to implement the implicit multiplication
fn englobe_if_multiplied_by_constant(
    prev_token: &Option<Rule>,
    queue: &mut Vec<PreExp>,
    rhs: PreExp,
) -> PreExp {
    match prev_token {
        Some(Rule::number) => {
            let last_number = queue.pop().unwrap();
            PreExp::BinaryOperation(Operator::Mul, last_number.to_boxed(), rhs.to_boxed())
        }
        _ => rhs,
    }
}
