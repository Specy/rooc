use pest::iterators::Pair;

use crate::{ parser::{Rule, Objective, Error, Condition, Bounds, Exp, Comparison, Operator, OptimizationType}};

pub fn parse_objective(objective: Pair<'_, Rule>) -> Result<Objective, Error> {
    match objective.as_rule() {
        Rule::objective => {
            let mut pairs = objective.into_inner();

            let (objective_type, rhs) = (pairs.next(), pairs.next());
            if objective_type.is_none() {
                return Err(Error::MissingToken("Missing min/max ".to_string()));
            }
            let objective_type = objective_type.unwrap().as_str();
            let objective_type = match objective_type {
                "min" => OptimizationType::Min,
                "max" => OptimizationType::Max,
                _ => {
                    return Err(Error::UnexpectedToken(format!(
                        "(1) found {}, expected min/max",
                        objective_type
                    )))
                }
            };
            match rhs {
                Some(rhs) => Ok(Objective::new(
                    objective_type,
                    parse_exp_list(rhs)?,
                )),
                None => Err(Error::UnexpectedToken(
                    "(2) Missing objective function".to_string(),
                )),
            }
        }
        _ => {
            return Err(Error::UnexpectedToken(format!(
                "(3) found {}, expected objective",
                objective.to_string()
            )))
        }
    }
}

pub fn parse_condition_list(condition_list: Pair<'_, Rule>) -> Result<Vec<Condition>, Error> {
    match condition_list.as_rule() {
        Rule::condition_list => condition_list
            .into_inner()
            .map(|c| parse_condition(c))
            .collect::<Result<Vec<Condition>, Error>>(),
        _ => Err(Error::UnexpectedToken("(15) Expected condition list".to_string())),
    }
}
pub fn parse_bounds_list(bounds_list: Pair<'_, Rule>) -> Result<Vec<Bounds>, Error> {
    match bounds_list.as_rule() {
        Rule::bounds_list => bounds_list
            .into_inner()
            .map(|c| parse_bounds(c))
            .collect::<Result<Vec<Bounds>, Error>>(),
        _ => Err(Error::UnexpectedToken("(16) Expected bounds list".to_string())),
    }
}

fn parse_comma_separated_vars(vars: Pair<'_, Rule>) -> Result<Vec<String>, Error> {
    match vars.as_rule() {
        Rule::comma_separated_vars => vars
            .into_inner()
            .map(|c| parse_var(c))
            .collect::<Result<Vec<String>, Error>>(),
        _ => Err(Error::UnexpectedToken("(17) Expected comma separated vars".to_string())),
    }
}
fn parse_var(var: Pair<'_, Rule>) -> Result<String, Error> {
    match var.as_rule() {
        Rule::variable => Ok(var.as_str().to_string()),
        _ => Err(Error::UnexpectedToken("(18) Expected var".to_string())),
    }
}
fn parse_condition(condition: Pair<'_, Rule>) -> Result<Condition, Error> {
    match condition.as_rule() {
        Rule::condition => {
            let mut inner = condition.into_inner();
            let (rhs, condition_type, lhs) = (
                inner.next(),
                inner.next(),
                inner.next(),
            );
            match (rhs, condition_type, lhs) {
                
                (Some(rhs), Some(condition_type), Some(lhs)) => Ok(Condition::new(
                    parse_exp_list(lhs)?,
                    parse_comparison(condition_type)?,
                    parse_exp_list(rhs)?
                )),
                _ => Err(Error::MissingToken(
                    "(19) Missing condition".to_string(),
                )),
            }
        }
        _ => Err(Error::UnexpectedToken("(20) Expected condition".to_string())),
    }
}
fn parse_bounds(bounds: Pair<'_, Rule>) -> Result<Bounds, Error> {
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
                _ => Err(Error::UnexpectedToken(
                    "(21) Missing bounds".to_string(),
                )),
            }
        }
        _ => Err(Error::UnexpectedToken("(22) Expected bounds".to_string())),
    }    
}

fn parse_number(number: Pair<'_, Rule>) -> Result<f64, Error> {
    match number.as_rule() {
        Rule::number => {
            let number = number.as_str();
            let number = match number.parse::<f64>() {
                Ok(number) => number,
                Err(_) => {
                    return Err(Error::UnexpectedToken(format!(
                        "(23) found {}, expected number",
                        number
                    )))
                }
            };
            Ok(number)
        }
        _ => Err(Error::UnexpectedToken("(24) Expected number".to_string())),
    }
}

fn parse_comparison(comparison: Pair<'_, Rule>) -> Result<Comparison, Error> {
    match comparison.as_rule() {
        Rule::comparison => {
            let comparison = comparison.as_str();
            let comparison = match comparison {
                "<=" => Comparison::LowerOrEqual,
                ">=" => Comparison::UpperOrEqual,
                "=" => Comparison::Equal,
                _ => {
                    return Err(Error::UnexpectedToken(format!(
                        "(25) found {}, expected comparison",
                        comparison
                    )))
                }
            };
            Ok(comparison)
        }
        _ => Err(Error::UnexpectedToken(
            "(26) Expected comparison".to_string(),
        )),
    }
}
fn parse_exp_list(exp_list: Pair<'_, Rule>) -> Result<Exp, Error> {
    match exp_list.as_rule() {
        Rule::exp_list => {
            //use shunting yard algorithm to parse expression list into a Exp tree
            let mut output_queue: Vec<Exp> = Vec::new();
            let mut operator_stack: Vec<Operator> = Vec::new();
            let mut last_token: Option<Rule> = None;
            for exp in exp_list.into_inner() {
                let rule = exp.as_rule();
                match rule {
                    Rule::op => {
                        let op = parse_operator(&exp)?;

                        while should_unwind(&operator_stack, &op) {
                            let op = operator_stack.pop();
                            let rhs = output_queue.pop();
                            let lhs = output_queue.pop();
                            if op.is_none() || rhs.is_none() || lhs.is_none() {
                                return Err(Error::UnexpectedToken(format!(
                                    "(4) found {}, expected exp",
                                    exp.as_str()
                                )));
                            }
                            let (op, rhs, lhs) = (op.unwrap(), rhs.unwrap(), lhs.unwrap());
                            output_queue.push(Exp::BinaryOperation(
                                op,
                                lhs.to_boxed(),
                                rhs.to_boxed(),
                            ));
                        }
                        //check if the operator is unary, if so, add a zero to the output queue
                        if last_token == Some(Rule::op) || last_token == None {
                            match op {
                                Operator::Sub => {
                                    output_queue.push(Exp::Number(0.0));
                                }
                                _ => {
                                    return Err(Error::UnexpectedToken(format!(
                                        "(5) Unexpected unary token {}, expected exp",
                                        exp.as_str()
                                    )))
                                }
                            }
                        }
                        operator_stack.push(op);
                    }
                    Rule::variable => {
                        let variable = Exp::Variable(exp.as_str().to_string());
                        let next = englobe_if_multiplied_by_constant(
                            &last_token,
                            &mut output_queue,
                            variable,
                        );
                        output_queue.push(next);
                    }
                    Rule::number => {
                        let num = exp.as_str().parse::<f64>();
                        match num {
                            Ok(num) => output_queue.push(Exp::Number(num)),
                            Err(_) => {
                                return Err(Error::UnexpectedToken(format!(
                                    "(6) found {}, expected number",
                                    exp.as_str()
                                )))
                            }
                        }
                    }
                    Rule::parenthesis => {
                        let first = exp.into_inner().next();
                        match first {
                            Some(inner) => {
                                let par = parse_exp_list(inner)?;
                                let par = englobe_if_multiplied_by_constant(
                                    &last_token,
                                    &mut output_queue,
                                    par,
                                );
                                output_queue.push(par);
                            }
                            None => {
                                return Err(Error::UnexpectedToken(format!(
                                    "(7) found, expected exp",
                                )))
                            }
                        }
                    }
                    Rule::modulo => {
                        let first = exp.into_inner().next();
                        match first {
                            Some(inner) => {
                                let exp = parse_exp_list(inner)?;
                                let modulo = Exp::Mod(Box::new(exp));
                                let modulo = englobe_if_multiplied_by_constant(
                                    &last_token,
                                    &mut output_queue,
                                    modulo,
                                );
                                output_queue.push(modulo)
                            }
                            None => {
                                return Err(Error::UnexpectedToken(format!(
                                    "(8) found, expected exp",
                                )))
                            }
                        }
                    }
                    Rule::min | Rule::max => {
                        let members = exp
                            .into_inner()
                            .map(|member| parse_exp_list(member))
                            .collect::<Result<Vec<Exp>, Error>>()?;
                        let fun = match rule {
                            Rule::min => Exp::Min(members),
                            Rule::max => Exp::Max(members),
                            _ => unreachable!(),
                        };

                        let fun =
                            englobe_if_multiplied_by_constant(&last_token, &mut output_queue, fun);
                        output_queue.push(fun);
                    }
                    _ => {
                        return Err(Error::UnexpectedToken(format!(
                            "(9) found {}, expected exp or op",
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
                    return Err(Error::UnexpectedToken(format!(
                        "(10) found, expected exp",
                    )));
                }
                let (op, rhs, lhs) = (op.unwrap(), rhs.unwrap(), lhs.unwrap());
                output_queue.push(Exp::BinaryOperation(op, lhs.to_boxed(), rhs.to_boxed()));
            }
            match output_queue.pop() {
                Some(exp) => Ok(exp),
                None => Err(Error::UnexpectedToken(format!("(11) Invalid expression"))),
            }
        }
        _ => Err(Error::UnexpectedToken(format!(
            "(12) found {}, expected exp_list",
            exp_list.as_str()
        ))),
    }
}

fn should_unwind(operator_stack: &Vec<Operator>, op: &Operator) -> bool {
    match operator_stack.last() {
        Some(top) => top.precedence() >= op.precedence(),
        None => false,
    }
}

fn parse_operator(operator: &Pair<'_, Rule>) -> Result<Operator, Error> {
    match operator.as_rule() {
        Rule::op => {
            let op = operator.as_str();
            match op {
                "+" => return Ok(Operator::Add),
                "-" => return Ok(Operator::Sub),
                "*" => return Ok(Operator::Mul),
                "/" => return Ok(Operator::Div),
                _ => {
                    return Err(Error::UnexpectedToken(format!(
                        "(13) found {}, expected +, -, *, /",
                        op
                    )))
                }
            }
        }
        _ => {
            return Err(Error::UnexpectedToken(format!(
                "(14) found {}, expected op",
                operator.as_str()
            )))
        }
    }
}

//if the previous token was a number, englobe the rhs in a multiplication, this is to implement the implicit multiplication
fn englobe_if_multiplied_by_constant(
    prev_token: &Option<Rule>,
    queue: &mut Vec<Exp>,
    rhs: Exp,
) -> Exp {
    match prev_token {
        Some(Rule::number) => {
            let last_number = queue.pop().unwrap();
            Exp::BinaryOperation(Operator::Mul, last_number.to_boxed(), rhs.to_boxed())
        }
        _ => rhs,
    }
}