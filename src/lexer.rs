use std::str::FromStr;

use logos::Logos;
/*
// Example problem to lex:

max 3x + |x - 2| + 5 4(x1 + 8)
s.t.
    x1 + x2 <= 10
    x1 + 2x2 >= 5
    |2x1 - 5x2| <= 10
    x1, x2 >= 0
 */
#[derive(Debug, PartialEq, Clone)]
pub enum Operation{
    Add,
    Subtract,
    Multiply,
    Divide,
}
impl Operation{
    pub fn get_precedence(&self) -> u8{
        match self{
            Operation::Add => 1,
            Operation::Subtract => 1,
            Operation::Multiply => 2,
            Operation::Divide => 2,
        }
    }
}


impl FromStr for Operation{
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(Operation::Add),
            "-" => Ok(Operation::Subtract),
            "*" => Ok(Operation::Multiply),
            "/" => Ok(Operation::Divide),
            _ => Err(()),
        }
    }
} 
#[derive(Debug, PartialEq, Clone)]
pub enum ObjectiveType{
    Min,
    Max,
}
impl FromStr for ObjectiveType{
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "min" => Ok(ObjectiveType::Min),
            "max" => Ok(ObjectiveType::Max),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConstraintType{
    Lower,
    LowerOrEqual,
    Upper,
    UpperOrEqual,
    Equal,
}
impl FromStr for ConstraintType{
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "<" => Ok(ConstraintType::Lower),
            "<=" => Ok(ConstraintType::LowerOrEqual),
            ">" => Ok(ConstraintType::Upper),
            ">=" => Ok(ConstraintType::UpperOrEqual),
            "=" => Ok(ConstraintType::Equal),
            _ => Err(()),
        }
    }
}


#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t]+")]
pub enum Token {
    #[regex(r"max|min", |lex| lex.slice().parse())]
    Objective(ObjectiveType),
    #[regex(r"s\.t\.")]
    St,
    #[regex(r"[a-zA-Z][a-zA-Z0-9]*", |lex| lex.slice().to_string())]
    Variable(String),
    #[regex(r"[0-9]+(\.[0-9])*", |lex| lex.slice().parse().ok())]
    Number(f64),
    #[regex(r"[+*/-]", |lex| lex.slice().parse().ok())]
    Operation(Operation),
    #[regex(r"=|<=|<|>|>=", |lex| lex.slice().parse().ok())]
    Relation(ConstraintType),
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("|")]
    Modulus,
    #[token(",")]
    Comma,
    #[regex(r"\n")]
    Newline,
}


pub struct Lexer<'a> {
    lexer: logos::Lexer<'a, Token>,
    problem: String,
    tokens: Vec<Result<Token, ()>>,
}

impl Lexer<'_> {
    pub fn new(problem: &str) -> Lexer {
        let lexer = Token::lexer(problem.trim());
        let tokens: Vec<Result<Token, ()>> = lexer.clone().collect();
        Lexer {
            lexer,
            problem: problem.to_string(),
            tokens: tokens,
        }
    }

    pub fn get_tokens(&self) -> Result<Vec<Token>,()> {
        let mut tokens: Vec<Token> = Vec::new();
        for token in self.tokens.clone() {
            match token {
                Ok(token) => tokens.push(token),
                Err(_) => return Err(()),
            }
        }
        Ok(tokens)
    }
}
