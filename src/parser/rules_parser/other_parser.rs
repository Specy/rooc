use std::vec;

use pest::iterators::{Pair, Pairs};

use crate::math::{Comparison, OptimizationType, PreVariableType};
use crate::parser::domain_declaration::{Variable, VariablesDomainDeclaration};
use crate::parser::il::IterableSet;
use crate::parser::il::PreExp;
use crate::parser::il::{AddressableAccess, CompoundVariable, PreConstraint, PreObjective};
use crate::parser::il::{
    BlockFunction, BlockFunctionKind, BlockScopedFunction, BlockScopedFunctionKind,
};
use crate::parser::iterable_utils::flatten_primitive_array_values;
use crate::parser::model_transformer::VariableKind;
use crate::parser::pre_model::Rule;
use crate::primitives::Constant;
use crate::primitives::Primitive;
use crate::primitives::{Graph, GraphEdge, GraphNode};
use crate::utils::{CompilationError, InputSpan, ParseError, Spanned};

use super::exp_parser::parse_exp;

use crate::runtime_builtin::FunctionCall;
use crate::{bail_missing_token, err_unexpected_token};

pub fn parse_objective(objective: Pair<Rule>) -> Result<PreObjective, CompilationError> {
    match objective.as_rule() {
        Rule::objective => {
            let pairs = objective.clone().into_inner();
            let objective_type = pairs.find_first_tagged("objective_type");
            let objective_body = pairs.find_first_tagged("objective_body");
            if objective_type.is_none() {
                return bail_missing_token!("Missing min/max in objective", objective);
            }
            match (objective_body, objective_type) {
                (Some(body), Some(objective_type)) => {
                    let obj_type = objective_type.as_str().parse::<OptimizationType>();
                    if obj_type.is_err() {
                        return err_unexpected_token!(
                            "Unknown objective type \"{}\", expected one of \"{}\"",
                            objective_type,
                            OptimizationType::kinds_to_string().join(", ")
                        );
                    }
                    Ok(PreObjective::new(obj_type.unwrap(), parse_exp(body)?))
                }
                (None, Some(objective_type)) => {
                    let obj_type = objective_type.as_str().parse::<OptimizationType>();
                    match obj_type {
                        Ok(OptimizationType::Satisfy) => Ok(PreObjective::new(
                            obj_type.unwrap(),
                            PreExp::Primitive(Spanned::new(
                                Primitive::Boolean(true),
                                InputSpan::from_pair(&objective_type),
                            )),
                        )),
                        _ => {
                            err_unexpected_token!(
                                "Unknown objective type \"{}\", expected one of \"{}\"",
                                objective_type,
                                OptimizationType::kinds_to_string().join(", ")
                            )
                        }
                    }
                }
                _ => bail_missing_token!("Missing or wrong objective", objective),
            }
        }
        _ => {
            err_unexpected_token!("found {}, expected objective", objective)
        }
    }
}

pub fn parse_consts_declaration(
    consts_declarations: Pair<Rule>,
) -> Result<Vec<Constant>, CompilationError> {
    match consts_declarations.as_rule() {
        Rule::consts_declaration => consts_declarations
            .into_inner()
            .map(parse_const_declaration)
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
                .map(|n| Spanned::new(n.as_str().to_string(), InputSpan::from_span(n.as_span())));
            let value = pairs.find_first_tagged("value").map(|v| parse_exp(v));
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

pub fn parse_domains_declaration(
    domains_declarations: Pair<Rule>,
) -> Result<Vec<VariablesDomainDeclaration>, CompilationError> {
    match domains_declarations.as_rule() {
        Rule::domains_declaration => domains_declarations
            .into_inner()
            .map(parse_domain_declaration)
            .collect(),
        _ => err_unexpected_token!(
            "Expected domains declaration but got: {}",
            domains_declarations
        ),
    }
}

pub fn parse_domain_declaration(
    rule: Pair<Rule>,
) -> Result<VariablesDomainDeclaration, CompilationError> {
    match rule.as_rule() {
        Rule::domain_declaration => {
            let inner = rule.clone().into_inner();
            let variables = inner
                .find_first_tagged("vars")
                .map(|v| parse_variables_assertion(v));
            let iteration = inner
                .find_first_tagged("iteration")
                .map(|v| parse_set_iterator_list(&v.into_inner()));
            let as_type = inner
                .find_first_tagged("as_type")
                .map(|v| parse_as_assertion_type(&v));
            let span = InputSpan::from_pair(&rule);
            match (variables, as_type) {
                (Some(variables), Some(as_type)) => {
                    let iteration = iteration.unwrap_or(Ok(vec![]));
                    Ok(VariablesDomainDeclaration::new(
                        variables?, as_type?, iteration?, span,
                    ))
                }
                _ => bail_missing_token!("Missing domain declaration body", rule),
            }
        }
        _ => err_unexpected_token!("Expected domain declaration but got: {}", rule),
    }
}

pub fn parse_variables_assertion(
    variables_assertions: Pair<Rule>,
) -> Result<Vec<Spanned<Variable>>, CompilationError> {
    match variables_assertions.as_rule() {
        Rule::domain_variables => variables_assertions
            .into_inner()
            .map(|v| parse_variable(&v))
            .collect(),
        _ => err_unexpected_token!(
            "Expected variables assertions but got: {}",
            variables_assertions
        ),
    }
}

pub fn parse_as_assertion_type(pair: &Pair<Rule>) -> Result<PreVariableType, CompilationError> {
    let as_type = pair.clone().into_inner();
    let (as_type, as_data) = (
        as_type.find_first_tagged("type"),
        as_type.find_first_tagged("values"),
    );
    if as_type.is_none() {
        return err_unexpected_token!("Expected type assertion but got: {}", pair);
    }
    let as_type = as_type.unwrap();
    let str = as_type.as_str();
    if let Some(as_data) = as_data {
        let mut values = as_data.clone().into_inner();
        let min = values.next().map(|v| parse_exp(v)).transpose()?;
        let max = values.next().map(|v| parse_exp(v)).transpose()?;

        match str {
            "IntegerRange" => {
                if min.is_none() || max.is_none() {
                    return err_unexpected_token!(
                        "IntegerRange must have min and max values: {}",
                        pair
                    );
                }
                return Ok(PreVariableType::IntegerRange(min.unwrap(), max.unwrap()));
            }
            "NonNegativeReal" => return Ok(PreVariableType::NonNegativeReal(min, max)),
            "Real" => return Ok(PreVariableType::Real(min, max)),
            _ => {
                return err_unexpected_token!(
                    "Unknown variable type \"{}\", expected one of \"{}\"",
                    pair,
                    PreVariableType::kinds_to_string().join(", ")
                )
            }
        }
    }
    if str == "IntegerRange" {
        return err_unexpected_token!("IntegerRange must have min and max: {}", pair);
    }
    match as_type.as_str().parse() {
        Ok(kind) => Ok(kind),
        Err(_) => err_unexpected_token!(
            "Unknown variable type \"{}\", expected one of \"{}\"",
            pair,
            PreVariableType::kinds_to_string().join(", ")
        ),
    }
}

pub fn parse_variable(pair: &Pair<Rule>) -> Result<Spanned<Variable>, CompilationError> {
    let span = InputSpan::from_pair(pair);
    match pair.as_rule() {
        Rule::compound_variable => {
            let compound_variable = parse_compound_variable(pair);
            match compound_variable {
                Ok(compound_variable) => Ok(Spanned::new(
                    Variable::CompoundVariable(compound_variable),
                    span,
                )),
                Err(e) => Err(e),
            }
        }
        Rule::simple_variable => Ok(Spanned::new(
            Variable::Variable(pair.as_str().to_string()),
            span,
        )),
        Rule::escaped_compound_variable => Ok(Spanned::new(
            Variable::Variable(pair.as_str()[1..].to_string()),
            span,
        )),
        _ => err_unexpected_token!("Expected variable but got: {}", pair),
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
        Rule::float | Rule::integer => parse_number(const_value),
        Rule::boolean => match const_value.as_str() {
            "true" => Ok(Primitive::Boolean(true)),
            "false" => Ok(Primitive::Boolean(false)),
            _ => err_unexpected_token!("Expected boolean but got: {}", const_value),
        },
        Rule::string => {
            let value = const_value.as_str();
            //remove quotes
            if value.len() < 2 {
                return err_unexpected_token!("Expected string but got: {}", const_value);
            }
            let value = value[1..value.len() - 1].to_string();
            Ok(Primitive::String(value))
        }
        Rule::array => {
            let values = const_value
                .clone()
                .into_inner()
                .map(|v| parse_primitive(&v))
                .collect::<Result<Vec<_>, CompilationError>>()?;
            Ok(Primitive::Iterable(flatten_primitive_array_values(values)))
        }
        Rule::graph => {
            let inner = const_value.clone().into_inner();
            let body = inner.find_first_tagged("body");
            match body {
                Some(b) => {
                    let inner = b
                        .into_inner()
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

pub fn parse_graph_edge(edge: &Pair<Rule>, from: &str) -> Result<GraphEdge, CompilationError> {
    let inner = edge.clone().into_inner();
    let node = inner.find_first_tagged("node");
    let cost = match inner.find_first_tagged("cost") {
        Some(cost) => {
            let parsed = cost.as_str().to_string().parse::<f64>();
            if parsed.is_err() {
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
            Ok(GraphEdge::new(from.to_owned(), node, cost))
        }
        _ => err_unexpected_token!("Expected graph edge but got: {}", edge),
    }
}

pub fn parse_constraint_list(
    constraint_list: &Pair<Rule>,
) -> Result<Vec<PreConstraint>, CompilationError> {
    match constraint_list.as_rule() {
        Rule::constraint_list => constraint_list
            .clone()
            .into_inner()
            .map(|c| parse_constraint(&c))
            .collect(),
        _ => err_unexpected_token!("Expected constraint list but got: {}", constraint_list),
    }
}

pub fn parse_constraint(constraint: &Pair<Rule>) -> Result<PreConstraint, CompilationError> {
    match constraint.as_rule() {
        Rule::constraint => {
            let inner = constraint.clone().into_inner();
            let name = inner.find_first_tagged("constraint_name");
            let name = match name {
                Some(v) => { 
                    
                    let first = v.clone().into_inner().next();
                    if first.is_none() {
                        return err_unexpected_token!("Expected variable but got: {}", v);
                    }
                    Some(parse_variable(&first.unwrap())?)
                }
                None => None,
            };
            let lhs = inner.find_first_tagged("constraint_lhs");
            let relation = inner.find_first_tagged("constraint_relation");
            let rhs = inner.find_first_tagged("constraint_rhs");
            let iteration = inner.find_first_tagged("constraint_iteration");
            match (rhs, relation, lhs, iteration) {
                (Some(rhs), Some(relation_type), Some(lhs), iteration) => {
                    let iteration = match iteration {
                        Some(iteration) => parse_set_iterator_list(&iteration.into_inner())?,
                        None => vec![],
                    };
                    Ok(PreConstraint::new(
                        name,
                        parse_exp(lhs)?,
                        parse_comparison(&relation_type)?,
                        parse_exp(rhs)?,
                        iteration,
                        InputSpan::from_pair(constraint),
                    ))
                }
                _ => bail_missing_token!("Missing constraint body", constraint),
            }
        }
        _ => err_unexpected_token!("Expected constraint but got: {}", constraint),
    }
}

pub fn parse_number(number: &Pair<Rule>) -> Result<Primitive, CompilationError> {
    match number.as_rule() {
        Rule::float => match number.as_str().parse::<f64>() {
            Ok(number) => Ok(Primitive::Number(number)),
            Err(_) => err_unexpected_token!("found {}, expected number", number),
        },
        Rule::integer => match number.as_str().parse::<i64>() {
            Ok(number) => Ok(Primitive::Integer(number)),
            Err(_) => err_unexpected_token!("found {}, expected number", number),
        },
        _ => err_unexpected_token!("Expected number but got: {}", number),
    }
}

pub fn parse_comparison(comparison: &Pair<Rule>) -> Result<Comparison, CompilationError> {
    match comparison.as_rule() {
        Rule::comparison => match comparison.as_str().parse() {
            Ok(comparison) => Ok(comparison),
            Err(_) => err_unexpected_token!("found {}, expected comparison", comparison),
        },
        _ => err_unexpected_token!("Expected comparison but got: {}", comparison),
    }
}

pub fn parse_set_iterator_list(
    range_list: &Pairs<Rule>,
) -> Result<Vec<IterableSet>, CompilationError> {
    range_list
        .clone()
        .map(|r| parse_set_iterator(&r))
        .collect::<Result<Vec<IterableSet>, CompilationError>>()
}

pub fn parse_block_scoped_function(exp: &Pair<Rule>) -> Result<PreExp, CompilationError> {
    let span = InputSpan::from_pair(exp);
    let inner = exp.clone().into_inner();
    let name = inner.find_first_tagged("name");
    let body = inner.find_first_tagged("body");
    let iters = inner.find_first_tagged("range");
    if name.is_none() || iters.is_none() || body.is_none() {
        return err_unexpected_token!("found {}, expected scoped block function", exp);
    }
    let body = body.unwrap();
    let name = name.unwrap();
    let iters = iters.unwrap();

    let iters = parse_set_iterator_list(&iters.into_inner())?;
    let kind = parse_scoped_block_function_type(&name)?;
    let body = parse_exp(body)?.to_boxed();
    let fun = BlockScopedFunction::new(kind, iters, body);
    Ok(PreExp::BlockScopedFunction(Spanned::new(fun, span)))
}

pub fn parse_block_function(exp: &Pair<Rule>) -> Result<PreExp, CompilationError> {
    let span = InputSpan::from_pair(exp);
    let inner = exp.clone().into_inner();
    let name = inner.find_first_tagged("name");
    let body = inner.find_first_tagged("body");
    if name.is_none() || body.is_none() {
        return err_unexpected_token!("found {}, expected block function", exp);
    }
    let members = body
        .unwrap()
        .into_inner()
        .map(parse_exp)
        .collect::<Result<Vec<PreExp>, CompilationError>>()?;
    let kind = parse_block_function_type(&name.unwrap())?;
    let fun = BlockFunction::new(kind, members);
    Ok(PreExp::BlockFunction(Spanned::new(fun, span)))
}

pub fn parse_compound_variable(
    compound_variable: &Pair<Rule>,
) -> Result<CompoundVariable, CompilationError> {
    match compound_variable.as_rule() {
        Rule::compound_variable => {
            let mut fields = compound_variable.clone().into_inner().collect::<Vec<_>>();
            let str = compound_variable.as_str();
            let starts_with = str.chars().next().unwrap_or('_');
            if starts_with == '_' {
                //compound variable has no name, only indexes
                let indexes = fields
                    .into_iter()
                    .map(|i| parse_compound_variable_index(i))
                    .collect::<Result<Vec<_>, CompilationError>>()?;
                Ok(CompoundVariable::new("".to_string(), indexes))
            } else if fields.len() >= 2 {
                //compound variable has name and indexes
                let name = fields.remove(0);
                let indexes = fields
                    .into_iter()
                    .map(|i| parse_compound_variable_index(i))
                    .collect::<Result<Vec<_>, CompilationError>>()?;
                Ok(CompoundVariable::new(name.as_str().to_string(), indexes))
            } else {
                err_unexpected_token!("found {}, expected compound variable", compound_variable)
            }
        }
        _ => err_unexpected_token!("Expected compound variable but got: {}", compound_variable),
    }
}

pub fn parse_compound_variable_index(
    compound_variable_index: Pair<Rule>,
) -> Result<PreExp, CompilationError> {
    match compound_variable_index.as_rule() {
        Rule::simple_variable => {
            let span = InputSpan::from_pair(&compound_variable_index);
            Ok(PreExp::Variable(Spanned::new(
                compound_variable_index.as_str().to_string(),
                span,
            )))
        }
        Rule::escaped_compound_variable => {
            let span = InputSpan::from_pair(&compound_variable_index);
            Ok(PreExp::Variable(Spanned::new(
                compound_variable_index.as_str()[1..].to_string(),
                span,
            )))
        }
        Rule::integer => {
            let span = InputSpan::from_pair(&compound_variable_index);
            let value = compound_variable_index.as_str().parse::<i64>();
            if value.is_err() {
                return err_unexpected_token!(
                    "Expected integer but got: {}",
                    compound_variable_index
                );
            }
            Ok(PreExp::Primitive(Spanned::new(
                Primitive::Integer(value.unwrap()),
                span,
            )))
        }
        Rule::tagged_exp => parse_exp(compound_variable_index),
        _ => err_unexpected_token!(
            "Expected compound variable index but got: {}",
            compound_variable_index
        ),
    }
}

pub fn parse_block_function_type(
    block_function_type: &Pair<Rule>,
) -> Result<BlockFunctionKind, CompilationError> {
    match block_function_type.as_str().parse() {
        Ok(kind) => Ok(kind),
        Err(_) => err_unexpected_token!(
            "Unknown block function \"{}\", expected one of \"{}\"",
            block_function_type,
            BlockFunctionKind::kinds_to_string().join(", ")
        ),
    }
}

pub fn parse_scoped_block_function_type(
    scoped_block_function_type: &Pair<Rule>,
) -> Result<BlockScopedFunctionKind, CompilationError> {
    match scoped_block_function_type.as_str().parse() {
        Ok(kind) => Ok(kind),
        Err(_) => err_unexpected_token!(
            "Unknown scoped block function \"{}\", expected one of \"{}\"",
            scoped_block_function_type,
            BlockScopedFunctionKind::kinds_to_string().join(", ")
        ),
    }
}

pub fn parse_function_call(function_call: &Pair<Rule>) -> Result<FunctionCall, CompilationError> {
    match function_call.as_rule() {
        Rule::function => {
            let span = function_call.as_span();
            let inner = function_call.clone().into_inner();
            let name = inner.find_first_tagged("function_name");
            let args = inner.find_first_tagged("function_pars");
            match (name, args) {
                (Some(name), Some(args)) => Ok(FunctionCall::new(
                    parse_parameters(&args)?,
                    name.as_str().to_string(),
                    span,
                )),
                _ => err_unexpected_token!("Expected function call but got: {}", function_call),
            }
        }
        _ => err_unexpected_token!("Expected function call but got: {}", function_call),
    }
}

pub fn parse_parameters(pars: &Pair<Rule>) -> Result<Vec<PreExp>, CompilationError> {
    match pars.as_rule() {
        Rule::function_pars => {
            let inner = pars.clone().into_inner();
            let pars = inner
                .map(parse_parameter)
                .collect::<Result<Vec<PreExp>, CompilationError>>()?;
            Ok(pars)
        }
        _ => err_unexpected_token!("Expected function args but got: {}", pars),
    }
}

pub fn parse_parameter(arg: Pair<Rule>) -> Result<PreExp, CompilationError> {
    match arg.as_rule() {
        Rule::tagged_exp => parse_exp(arg),
        _ => err_unexpected_token!("Expected function arg but got: {}", arg),
    }
}

pub fn parse_set_iterator(range: &Pair<Rule>) -> Result<IterableSet, CompilationError> {
    match range.as_rule() {
        Rule::iteration_declaration => {
            let inner = range.clone().into_inner();
            let vars_tuple = inner
                .find_first_tagged("tuple")
                .map(|t| parse_variable_type(&t));
            let iterator = inner.find_first_tagged("iterator").map(|f| {
                let span = InputSpan::from_pair(range);
                parse_iterator(&f).map(|i| Spanned::new(i, span))
            });
            match (vars_tuple, iterator) {
                (Some(vars_tuple), Some(iterator)) => {
                    let span = InputSpan::from_pair(range);
                    Ok(IterableSet::new(vars_tuple?, iterator?, span))
                }
                _ => err_unexpected_token!("Expected set iterator but got: {}", range),
            }
        }
        _ => err_unexpected_token!("Expected set iterator but got: {}", range),
    }
}

pub fn parse_variable_type(tuple: &Pair<Rule>) -> Result<VariableKind, CompilationError> {
    match tuple.as_rule() {
        Rule::tuple => {
            let inner = tuple.clone().into_inner();
            let inner = inner
                .map(|i| {
                    let span = InputSpan::from_pair(&i);
                    match i.as_rule() {
                        Rule::simple_variable | Rule::no_par => {
                            Ok(Spanned::new(i.as_str().to_string(), span))
                        }
                        Rule::escaped_compound_variable => {
                            Ok(Spanned::new(i.as_str()[1..].to_string(), span))
                        }
                        _ => err_unexpected_token!("Expected variable but got: {}", i),
                    }
                })
                .collect::<Result<Vec<_>, CompilationError>>()?;
            Ok(VariableKind::Tuple(inner))
        }
        Rule::simple_variable | Rule::escaped_compound_variable => {
            let span = InputSpan::from_pair(tuple);
            Ok(VariableKind::Single(Spanned::new(
                tuple.as_str().to_string(),
                span,
            )))
        }
        _ => err_unexpected_token!("Expected tuple but got: {}", tuple),
    }
}

pub fn parse_iterator(iterator: &Pair<Rule>) -> Result<PreExp, CompilationError> {
    match iterator.as_rule() {
        Rule::iterator => {
            let mut inner = iterator.clone().into_inner();
            let first: Option<Rule> = inner.next().map(|i| i.as_rule());
            match first {
                Some(Rule::range_iterator) => {
                    let inner = iterator.clone().into_inner();
                    let from = inner.find_first_tagged("from").map(parse_parameter);
                    let to = inner.find_first_tagged("to").map(parse_parameter);
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
                            let span = InputSpan::from_pair(iterator);
                            let to_inclusive = PreExp::Primitive(Spanned::new(
                                Primitive::Boolean(to_inclusive),
                                span.clone(),
                            ));
                            Ok(PreExp::FunctionCall(
                                span,
                                FunctionCall::new(
                                    vec![from?, to?, to_inclusive],
                                    "range".to_string(),
                                    iterator.as_span(),
                                ),
                            ))
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
                    let function = parse_parameter(first.unwrap())?;
                    Ok(function)
                }

                _ => err_unexpected_token!("Expected range or parameter but got: {}", iterator),
            }
        }
        _ => err_unexpected_token!("Expected iterator but got: {}", iterator),
    }
}

pub fn parse_array_access(
    array_access: &Pair<Rule>,
) -> Result<AddressableAccess, CompilationError> {
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
                .map(parse_parameter)
                .collect::<Result<Vec<_>, CompilationError>>()?;
            Ok(AddressableAccess::new(name, accesses))
        }
        _ => err_unexpected_token!("Expected array access but got: {}", array_access),
    }
}
