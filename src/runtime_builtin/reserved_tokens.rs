use std::{collections::HashMap, fmt::Display};

use lazy_static::lazy_static;
use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    parser::{
        pre_parsed_problem::{BlockFunctionKind, BlockScopedFunctionKind},
        transformer::TransformError,
    },
    primitives::functions::function_traits::FunctionCall,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum TokenType {
    Function,
    Literal,
    Variable,
    Keyword,
    Type,
}
#[wasm_bindgen(typescript_custom_section)]
const TOKEN_TYPE: &'static str = r#"
export type SerializedTokenType = "Function" | "Literal" | "Variable" | "Keyword" | "Type";
"#;
impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Function => write!(f, "Function"),
            TokenType::Literal => write!(f, "Literal"),
            TokenType::Variable => write!(f, "Variable"),
            TokenType::Keyword => write!(f, "Keyword"),
            TokenType::Type => write!(f, "Type"),
        }
    }
}

lazy_static! {
    static ref RESERVED_TOKEN: HashMap<String, TokenType> = {
        let mut m = HashMap::new();
        m.insert("min".to_string(), TokenType::Keyword);
        m.insert("max".to_string(), TokenType::Keyword);
        m.insert("s.t.".to_string(), TokenType::Keyword);
        m.insert("where".to_string(), TokenType::Keyword);
        m.insert("in".to_string(), TokenType::Keyword);
        m.insert("for".to_string(), TokenType::Keyword);
        m.insert("as".to_string(), TokenType::Keyword);
        m.insert("if".to_string(), TokenType::Keyword);
        m.insert("else".to_string(), TokenType::Keyword);

        m.insert("true".to_string(), TokenType::Literal);
        m.insert("false".to_string(), TokenType::Literal);

        m.insert("Graph".to_string(), TokenType::Type);

        for v in BlockFunctionKind::kinds_to_string() {
            m.insert(v, TokenType::Function);
        }
        for v in BlockScopedFunctionKind::kinds_to_string() {
            m.insert(v, TokenType::Function);
        }

        m.insert("nodes".to_string(), TokenType::Function);
        m.insert("neigh_edges".to_string(), TokenType::Function);
        m.insert("neigh_edges_of".to_string(), TokenType::Function);
        m.insert("range".to_string(), TokenType::Function);
        m.insert("edges".to_string(), TokenType::Function);
        m.insert("len".to_string(), TokenType::Function);
        m.insert("enumerate".to_string(), TokenType::Function);
        m
    };
}

pub fn check_if_reserved_token(token: &str) -> Result<(), TransformError> {
    match RESERVED_TOKEN.get(token) {
        Some(kind) => Err(TransformError::AlreadyDefined {
            name: token.to_string(),
            kind: kind.clone(),
        }),
        None => Ok(()),
    }
}
