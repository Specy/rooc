use pest::iterators::Pair;

use crate::{parser::{parser::Rule, pre_parsed_problem::PreExp}, utils::{CompilationError, InputSpan, Spanned, ParseError}, math::operators::{BinOp, UnOp}, err_unexpected_token, primitives::primitive::Primitive, bail_missing_token};

use super::other_parser::{parse_function_call, parse_block_function, parse_block_scoped_function, parse_primitive, parse_array_access, parse_compound_variable};




pub fn parse_exp(exp_to_parse: &Pair<Rule>) -> Result<PreExp, CompilationError> {
    let mut output_queue: Vec<PreExp> = Vec::new();
    let mut operator_stack: Vec<BinOp> = Vec::new();
    let mut last_token: Option<Rule> = None;
    let exp_list = exp_to_parse.clone().into_inner();
    let mut last_op_span = InputSpan::default();

    for exp in exp_list.into_iter() {
        let rule = exp.as_rule();
        let span = InputSpan::from_pair(&exp);
        match rule {
            Rule::binary_op => {
                last_op_span = span.clone();
                let op = parse_bin_operator(&exp)?;
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
                let op = parse_bin_operator(&exp)?;
                match op {
                    BinOp::Sub => {
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
                let fun_exp = parse_block_function(&exp)?;
                let sum =
                    englobe_if_multiplied_by_constant(&last_token, &mut output_queue, fun_exp)?;
                output_queue.push(sum);
            }
            Rule::block_scoped_function => {
                let fun_exp = parse_block_scoped_function(&exp)?;
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


fn should_unwind(operator_stack: &[BinOp], op: &BinOp) -> bool {
    match operator_stack.last() {
        Some(top) => top.precedence() >= op.precedence(),
        None => false,
    }
}

fn parse_bin_operator(operator: &Pair<Rule>) -> Result<BinOp, CompilationError> {
    match operator.as_rule() {
        //TODO add separate unary operators?
        Rule::binary_op | Rule::unary_op => match operator.as_str().parse() {
            Ok(op) => Ok(op),
            Err(_) => {
                err_unexpected_token!("found {}, expected", operator)
            }
        },
        _ => err_unexpected_token!("found {}, expected op", operator),
    }
}
fn parse_un_operator(operator: &Pair<Rule>) -> Result<UnOp, CompilationError> {
    match operator.as_rule() {
        //TODO add separate unary operators?
        Rule::unary_op => match operator.as_str().parse() {
            Ok(op) => Ok(op),
            Err(_) => {
                err_unexpected_token!("found {}, expected op", operator)
            }
        },
        _ => err_unexpected_token!("found {}, expected op", operator),
    }
}

//if the previous token was a number, englobe the rhs in a multiplication, this is to implement the implicit multiplication
pub fn englobe_if_multiplied_by_constant(
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
                Spanned::new(BinOp::Mul, span.clone()),
                last_number.to_boxed(),
                rhs.to_boxed(),
            );
            Ok(exp)
        }
        _ => Ok(rhs),
    }
}
