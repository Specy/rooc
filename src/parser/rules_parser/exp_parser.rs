use pest::iterators::Pair;
use pest::pratt_parser::PrattParser;

use crate::{
    err_unexpected_token,
    math::operators::{BinOp, UnOp},
    parser::{parser::Rule, pre_parsed_problem::PreExp},
    utils::{CompilationError, InputSpan, ParseError, Spanned},
};

use super::other_parser::{
    parse_array_access, parse_block_function, parse_block_scoped_function, parse_compound_variable,
    parse_function_call, parse_primitive,
};

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        PrattParser::new()
            .op(Op::infix(Rule::add, Left) | Op::infix(Rule::sub, Left))
            .op(Op::infix(Rule::mul, Left) | Op::infix(Rule::div, Left))
            //.op(Op::infix(Rule::pow, Right)) TODO should i add this?
            //.op(Op::infix(Rule::fac, Left)) TODO should i add this?
            .op(Op::prefix(Rule::neg))
    };
}
//TODO add implicit multiplication: 2x = 2 * x, should this be as a preprocessor? or part of the grammar?
pub fn parse_exp(exp_to_parse: Pair<Rule>) -> Result<PreExp, CompilationError> {
    PRATT_PARSER
        .map_primary(parse_exp_leaf)
        .map_infix(|lhs, op, rhs| {
            let span = InputSpan::from_pair(&op);
            let op = match op.as_rule() {
                Rule::add => BinOp::Add,
                Rule::sub => BinOp::Sub,
                Rule::mul => BinOp::Mul,
                Rule::div => BinOp::Div,
                _ => return err_unexpected_token!("found {}, expected op", op),
            };
            Ok(PreExp::BinaryOperation(
                Spanned::new(op, span),
                lhs?.to_boxed(),
                rhs?.to_boxed(),
            ))
        })
        .map_prefix(|op, rhs| {
            let span = InputSpan::from_pair(&op);
            let op = match op.as_rule() {
                Rule::neg => UnOp::Neg,
                _ => return err_unexpected_token!("found {}, expected op", op),
            };
            Ok(PreExp::UnaryOperation(
                Spanned::new(op, span),
                rhs?.to_boxed(),
            ))
        })
        .parse(exp_to_parse.into_inner())
}

pub fn parse_exp_leaf(exp: Pair<Rule>) -> Result<PreExp, CompilationError> {
    let span = InputSpan::from_pair(&exp);
    match exp.as_rule() {
        Rule::function => {
            let fun = parse_function_call(&exp)?;
            Ok(PreExp::FunctionCall(span, fun))
        }
        Rule::simple_variable => {
            let variable = exp.as_str().to_string();
            Ok(PreExp::Variable(Spanned::new(variable, span)))
        }
        Rule::compound_variable => {
            let variable = parse_compound_variable(&exp)?;
            Ok(PreExp::CompoundVariable(Spanned::new(variable, span)))
        }
        Rule::block_function => parse_block_function(&exp),
        Rule::block_scoped_function => parse_block_scoped_function(&exp),
        //also adding number since the implicit multiplication rule uses it without being part of the primitive
        Rule::primitive | Rule::float | Rule::integer => {
            let prim = parse_primitive(&exp)?;
            let spanned = Spanned::new(prim, span);
            Ok(PreExp::Primitive(spanned))
        }
        Rule::parenthesis => parse_exp(exp),
        Rule::modulo => {
            let exp = parse_exp(exp)?;
            Ok(PreExp::Mod(span, Box::new(exp)))
        }
        Rule::implicit_mul => {
            let exps = exp.clone().into_inner().map(parse_exp_leaf).collect::<Result<Vec<_>, _>>()?;
            if exps.len() < 2 {
                return err_unexpected_token!("implicit multiplication must have at least 2 operands, got {}", exp);
            }
            let mut iter = exps.into_iter();
            let first = iter.next().unwrap();
            let mut res = PreExp::BinaryOperation(
                Spanned::new(BinOp::Mul, span.clone()),
                first.to_boxed(),
                iter.next().unwrap().to_boxed(),
            );
            for exp in iter {
                res = PreExp::BinaryOperation(
                    Spanned::new(BinOp::Mul, span.clone()),
                    res.to_boxed(),
                    exp.to_boxed(),
                );
            }
            Ok(res)
        }
        Rule::array_access => {
            let access = parse_array_access(&exp)?;
            Ok(PreExp::ArrayAccess(Spanned::new(access, span)))
        }
        _ => err_unexpected_token!(
            "found \"{}\"({:?}), expected exp, binary_op, unary_op, len, variable, sum, primitive, parenthesis, array_access, min, max, block function or scoped function",
            exp, exp.as_rule()
        ),
    }
}
