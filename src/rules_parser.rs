use pest::iterators::{Pair, Pairs};

use crate::consts::{Comparison, Constant, ConstantValue, Operator, OptimizationType, ParseError};
use crate::parser::{
    PreAccess, PreArrayAccess, PreCondition, PreExp, PreLenOf, PreObjective, PreRange,
    PreRangeValue, Rule,
};

//done
pub fn parse_objective(objective: Pair<Rule>) -> Result<PreObjective, ParseError> {
    match objective.as_rule() {
        Rule::objective => {
            let pairs = objective.into_inner();
            let objective_type = pairs.find_first_tagged("objective_type");
            let objective_body = pairs.find_first_tagged("objective_body");
            if objective_type.is_none() {
                return Err(ParseError::MissingToken(format!(
                    "Missing min/max in objective"
                )));
            }
            let objective_type = objective_type.unwrap().as_str();
            let objective_type = match objective_type {
                "min" => OptimizationType::Min,
                "max" => OptimizationType::Max,
                _ => {
                    return Err(ParseError::UnexpectedToken(format!(
                        "found {}, expected min/max",
                        objective_type
                    )))
                }
            };
            match objective_body {
                Some(rhs) => Ok(PreObjective::new(
                    objective_type,
                    parse_exp_list(&rhs.into_inner())?,
                )),
                None => Err(ParseError::UnexpectedToken(format!(
                    "Missing objective function"
                ))),
            }
        }
        _ => {
            return Err(ParseError::UnexpectedToken(format!(
                "found {}, expected objective",
                objective
            )))
        }
    }
}
//done
pub fn parse_consts_declaration(
    consts_declarations: Pair<Rule>,
) -> Result<Vec<Constant>, ParseError> {
    match consts_declarations.as_rule() {
        Rule::consts_declaration => consts_declarations
            .into_inner()
            .map(|c| parse_const_declaration(c))
            .collect(),
        _ => Err(ParseError::UnexpectedToken(format!(
            "Expected consts declaration but got: {}",
            consts_declarations.as_str()
        ))),
    }
}
//done
pub fn parse_const_declaration(const_declaration: Pair<Rule>) -> Result<Constant, ParseError> {
    match const_declaration.as_rule() {
        Rule::const_declaration => {
            let str = const_declaration.as_str();
            let pairs = const_declaration.into_inner();
            let name = pairs
                .find_first_tagged("name")
                .map(|n| n.as_str().to_string());
            let value = pairs
                .find_first_tagged("value")
                .map(|v| parse_const_value(&v));
            if name.is_none() || value.is_none() {
                return Err(ParseError::UnexpectedToken(format!(
                    "Expected constant declaration but got: {}",
                    str
                )));
            }
            let name = name.unwrap();
            let value = value.unwrap()?;
            Ok(Constant::new(name, value))
        }
        _ => Err(ParseError::UnexpectedToken(format!(
            "Expected constant declaration but got: {}",
            const_declaration.as_str()
        ))),
    }
}
//done
pub fn parse_const_value(const_value: &Pair<Rule>) -> Result<ConstantValue, ParseError> {
    match const_value.as_rule() {
        Rule::number => Ok(ConstantValue::Number(parse_number(const_value)?)),
        Rule::array => {
            let values = const_value
                .clone()
                .into_inner()
                .map(|v| parse_array_value(&v))
                .collect::<Result<Vec<ArrayValue>, ParseError>>()?;
            //make sure they are all the same type
            let first = values.first();
            if first.is_none() {
                return Ok(ConstantValue::OneDimArray(vec![]));
            }
            let first = first.unwrap();
            match first {
                ArrayValue::Number(_) => {
                    let values = values
                        .into_iter()
                        .map(|v| match v {
                            ArrayValue::Number(n) => Ok(n),
                            ArrayValue::Array(_) | ArrayValue::Empty => {
                                Err(ParseError::UnexpectedToken(format!(
                                    "Expected number but got an array"
                                )))
                            }
                        })
                        .collect::<Result<Vec<f64>, ParseError>>()?;
                    Ok(ConstantValue::OneDimArray(values))
                }
                ArrayValue::Array(_) => {
                    let values = values
                        .into_iter()
                        .map(|v| match v {
                            ArrayValue::Array(n) => Ok(n),
                            ArrayValue::Number(_) | ArrayValue::Empty => {
                                Err(ParseError::UnexpectedToken(format!(
                                    "Expected array but got a number"
                                )))
                            }
                        })
                        .collect::<Result<Vec<Vec<f64>>, ParseError>>()?;
                    //make sure all the arrays are the same length
                    let lengths = values.iter().map(|v| v.len()).collect::<Vec<usize>>();
                    let first = lengths.first();
                    if first.is_none() {
                        return Ok(ConstantValue::TwoDimArray(vec![]));
                    }
                    let max = lengths.iter().max().unwrap();
                    if lengths.iter().any(|l| l != max) {
                        return Err(ParseError::UnexpectedToken(format!(
                            "Arrays must all have the same size, using the biggest one of: {}",
                            max
                        )));
                    }
                    Ok(ConstantValue::TwoDimArray(values))
                }
                ArrayValue::Empty => Ok(ConstantValue::OneDimArray(vec![])),
            }
        }
        _ => Err(ParseError::UnexpectedToken(format!(
            "Expected constant value but got: {}",
            const_value.as_str()
        ))),
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
pub fn parse_array_value(array_value: &Pair<Rule>) -> Result<ArrayValue, ParseError> {
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
                .collect::<Result<Vec<f64>, ParseError>>()?;
            if values.is_empty() {
                return Ok(ArrayValue::Empty);
            }
            Ok(ArrayValue::Array(values))
        }
        _ => Err(ParseError::UnexpectedToken(format!(
            "Expected array value but got: {}",
            array_value.as_str()
        ))),
    }
}
//done
pub fn parse_condition_list(condition_list: &Pair<Rule>) -> Result<Vec<PreCondition>, ParseError> {
    match condition_list.as_rule() {
        Rule::condition_list => condition_list
            .clone()
            .into_inner()
            .map(|c| parse_condition(&c))
            .collect(),
        _ => Err(ParseError::UnexpectedToken(format!(
            "Expected condition list but got: {}",
            condition_list.as_str()
        ))),
    }
}
/*
pub fn parse_bounds_list(bounds_list: Pair<Rule>) -> Result<Vec<Bounds>, ParseError> {
    match bounds_list.as_rule() {
        Rule::bounds_list => bounds_list
            .into_inner()
            .map(|c| parse_bounds(c))
            .collect::<Result<Vec<Bounds>, ParseError>>(),
        _ => Err(ParseError::UnexpectedToken("Expected bounds list".to_string())),
    }
}
fn parse_comma_separated_vars(vars: Pair<Rule>) -> Result<Vec<String>, ParseError> {
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
fn parse_var(var: Pair<Rule>) -> Result<String, ParseError> {
    match var.as_rule() {
        Rule::variable => Ok(var.as_str().to_string()),
        _ => Err(ParseError::UnexpectedToken("Expected var".to_string())),
    }
}
*/
//done
fn parse_condition(condition: &Pair<Rule>) -> Result<PreCondition, ParseError> {
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
                        Some(iteration) => Some(parse_range(&iteration)?),
                        None => None,
                    };
                    Ok(PreCondition::new(
                        parse_exp_list(&lhs.into_inner())?,
                        parse_comparison(&relation_type)?,
                        parse_exp_list(&rhs.into_inner())?,
                        iteration,
                    ))
                }
                _ => Err(ParseError::MissingToken(format!("Missing condition body"))),
            }
        }
        _ => Err(ParseError::UnexpectedToken(format!(
            "Expected condition but got: {}",
            condition.as_str()
        ))),
    }
}

/*
fn parse_bounds(bounds: Pair<Rule>) -> Result<Bounds, ParseError> {
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
fn parse_number(number: &Pair<Rule>) -> Result<f64, ParseError> {
    match number.as_rule() {
        Rule::number => {
            let number = number.as_str();
            let number = match number.parse::<f64>() {
                Ok(number) => number,
                Err(_) => {
                    return Err(ParseError::UnexpectedToken(format!(
                        "found {}, expected number",
                        number
                    )))
                }
            };
            Ok(number)
        }
        _ => Err(ParseError::UnexpectedToken(format!(
            "Expected number but got: {}",
            number.as_str()
        ))),
    }
}

//done
fn parse_comparison(comparison: &Pair<Rule>) -> Result<Comparison, ParseError> {
    match comparison.as_rule() {
        Rule::comparison => {
            let comparison = comparison.as_str();
            let comparison = match comparison {
                "<=" => Comparison::LowerOrEqual,
                ">=" => Comparison::UpperOrEqual,
                "=" => Comparison::Equal,
                _ => {
                    return Err(ParseError::UnexpectedToken(format!(
                        "found {}, expected comparison",
                        comparison
                    )))
                }
            };
            Ok(comparison)
        }
        _ => Err(ParseError::UnexpectedToken(format!(
            "Expected comparison but got: {}",
            comparison.as_str()
        ))),
    }
}
//done
fn parse_exp_list(exp_list: &Pairs<Rule>) -> Result<PreExp, ParseError> {
    //use shunting yard algorithm to parse expression list into a PreExp tree
    let mut output_queue: Vec<PreExp> = Vec::new();
    let mut operator_stack: Vec<Operator> = Vec::new();
    let mut last_token: Option<Rule> = None;
    for exp in exp_list.clone().into_iter() {
        let rule = exp.as_rule();
        match rule {
            Rule::binary_op | Rule::unary_op => {
                let op = parse_operator(&exp)?;

                while should_unwind(&operator_stack, &op) {
                    let op = operator_stack.pop();
                    let rhs = output_queue.pop();
                    let lhs = output_queue.pop();
                    if op.is_none() || rhs.is_none() || lhs.is_none() {
                        return Err(ParseError::UnexpectedToken(format!(
                            "found {}, expected exp",
                            exp.as_str()
                        )));
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
                            return Err(ParseError::UnexpectedToken(format!(
                                "Unexpected unary token {}, expected exp",
                                exp.as_str()
                            )))
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
                    return Err(ParseError::UnexpectedToken(format!(
                        "found {}, expected compound variable",
                        exp.as_str()
                    )));
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
                    .map(|r| parse_range_list(&r.into_inner()));
                let body = inner
                    .find_first_tagged("body")
                    .map(|b| parse_exp_list(&b.into_inner()));
                if range.is_none() || body.is_none() {
                    return Err(ParseError::UnexpectedToken(format!(
                        "found {}, expected sum",
                        exp.as_str()
                    )));
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
                        return Err(ParseError::UnexpectedToken(format!(
                            "found {}, expected number",
                            exp.as_str()
                        )))
                    }
                }
            }
            Rule::parenthesis => {
                let par = parse_exp_list(&exp.into_inner())?;
                let par =
                    englobe_if_multiplied_by_constant(&last_token, &mut output_queue, par);
                output_queue.push(PreExp::Parenthesis(par.to_boxed()));
            }
            Rule::modulo => {
                let exp = parse_exp_list(&exp.into_inner())?;
                let modulo = PreExp::Mod(Box::new(exp));
                let modulo = englobe_if_multiplied_by_constant(
                    &last_token,
                    &mut output_queue,
                    modulo,
                );
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
                    .map(|member| parse_exp_list(&member.into_inner()))
                    .collect::<Result<Vec<PreExp>, ParseError>>()?;
                let fun = match rule {
                    Rule::min => PreExp::Min(members),
                    Rule::max => PreExp::Max(members),
                    _ => unreachable!(),
                };

                let fun = englobe_if_multiplied_by_constant(&last_token, &mut output_queue, fun);
                output_queue.push(fun);
            }
            _ => {
                return Err(ParseError::UnexpectedToken(format!(
                    "found {}, expected exp, binary_op, unary_op, len, variable, sum, number, parenthesis, array_access, min or max",
                    exp.as_str()
                )))
            }
        }
        last_token = Some(rule);
    }
    while !operator_stack.is_empty() {
        let op = operator_stack.pop();
        let rhs = output_queue.pop();
        let lhs = output_queue.pop();
        if op.is_none() || rhs.is_none() || lhs.is_none() {
            return Err(ParseError::UnexpectedToken(format!("expected exp")));
        }
        let (op, rhs, lhs) = (op.unwrap(), rhs.unwrap(), lhs.unwrap());
        output_queue.push(PreExp::BinaryOperation(op, lhs.to_boxed(), rhs.to_boxed()));
    }
    match output_queue.pop() {
        Some(exp) => Ok(exp),
        None => Err(ParseError::UnexpectedToken(format!(
            "Invalid empty expression"
        ))),
    }
}

//done
fn parse_range_list(range_list: &Pairs<Rule>) -> Result<Vec<PreRange>, ParseError> {
    range_list
        .clone()
        .map(|r| parse_range(&r))
        .collect::<Result<Vec<PreRange>, ParseError>>()
}

//done
fn parse_range(range: &Pair<Rule>) -> Result<PreRange, ParseError> {
    match range.as_rule() {
        Rule::range_declaration => {
            let inner = range.clone().into_inner();
            let name = inner
                .find_first_tagged("name")
                .map(|n| n.as_str().to_string());
            let from = inner
                .find_first_tagged("from")
                .map(|f| parse_range_value(&f));
            let to = inner.find_first_tagged("to").map(|t| parse_range_value(&t));
            if name.is_none() || from.is_none() || to.is_none() {
                return Err(ParseError::UnexpectedToken(format!(
                    "Expected range but got: {}",
                    range.as_str()
                )));
            }
            Ok(PreRange {
                name: name.unwrap(),
                from: from.unwrap()?,
                to: to.unwrap()?,
            })
        }
        _ => Err(ParseError::UnexpectedToken(format!(
            "Expected range but got: {}",
            range.as_str()
        ))),
    }
}

//done
fn parse_range_value(range_value: &Pair<Rule>) -> Result<PreRangeValue, ParseError> {
    match range_value.as_rule() {
        Rule::number => Ok(PreRangeValue::Number(
            range_value.as_str().parse::<i32>().unwrap(),
        )),
        Rule::len => Ok(PreRangeValue::LenOf(parse_len(&range_value)?)),
        _ => Err(ParseError::UnexpectedToken(format!(
            "Expected range value but got: {}",
            range_value.as_str()
        ))),
    }
}

//done
fn parse_len(len: &Pair<Rule>) -> Result<PreLenOf, ParseError> {
    match len.as_rule() {
        Rule::len => {
            let inner = len.clone().into_inner().next();
            if inner.is_none() {
                return Err(ParseError::UnexpectedToken(format!(
                    "Expected len but got: {}",
                    len.as_str()
                )));
            }
            let inner = inner.unwrap();
            match inner.as_rule() {
                Rule::simple_variable => Ok(PreLenOf::Array(inner.as_str().to_string())),
                Rule::array_access => {
                    let array_access = parse_array_access(&inner)?;
                    Ok(PreLenOf::ArrayAccess(array_access))
                }
                _ => Err(ParseError::UnexpectedToken(format!(
                    "Expected len but got: {}",
                    len.as_str()
                ))),
            }
        }
        _ => Err(ParseError::UnexpectedToken(format!(
            "Expected len but got: {}",
            len.as_str()
        ))),
    }
}
//done
fn parse_array_access(array_access: &Pair<Rule>) -> Result<PreArrayAccess, ParseError> {
    match array_access.as_rule() {
        Rule::array_access => {
            let inner = array_access.clone().into_inner();
            let name = inner
                .find_first_tagged("name")
                .map(|n| n.as_str().to_string());
            let accesses = inner.find_first_tagged("accesses");
            if name.is_none() || accesses.is_none() {
                return Err(ParseError::UnexpectedToken(format!(
                    "Expected array access but got: {}",
                    array_access.as_str()
                )));
            }
            let name = name.unwrap();
            let accesses = accesses.unwrap();
            let accesses = accesses
                .into_inner()
                .map(|a| parse_pointer_access(&a))
                .collect::<Result<Vec<PreAccess>, ParseError>>()?;
            Ok(PreArrayAccess { name, accesses })
        }
        _ => Err(ParseError::UnexpectedToken(format!(
            "Expected array access but got: {}",
            array_access.as_str()
        ))),
    }
}

//done
fn parse_pointer_access(pointer_access: &Pair<Rule>) -> Result<PreAccess, ParseError> {
    match pointer_access.as_rule() {
        Rule::number => Ok(PreAccess::Number(
            pointer_access.as_str().parse::<i32>().unwrap(),
        )),
        Rule::simple_variable => Ok(PreAccess::Variable(pointer_access.as_str().to_string())),
        _ => Err(ParseError::UnexpectedToken(format!(
            "Expected pointer access but got: {}",
            pointer_access.as_str()
        ))),
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
fn parse_operator(operator: &Pair<Rule>) -> Result<Operator, ParseError> {
    match operator.as_rule() {
        //TODO add separate unary operators?
        Rule::binary_op | Rule::unary_op => {
            let op = operator.as_str();
            match op {
                "+" => return Ok(Operator::Add),
                "-" => return Ok(Operator::Sub),
                "*" => return Ok(Operator::Mul),
                "/" => return Ok(Operator::Div),
                _ => {
                    return Err(ParseError::UnexpectedToken(format!(
                        "found {}, expected +, -, *, /",
                        op
                    )))
                }
            }
        }
        _ => {
            return Err(ParseError::UnexpectedToken(format!(
                "found {}, expected op",
                operator.as_str()
            )))
        }
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
