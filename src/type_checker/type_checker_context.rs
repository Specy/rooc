use std::collections::HashMap;

use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    parser::{
        pre_parsed_problem::AddressableAccess,
        transformer::{Frame, TransformError},
    },
    primitives::{consts::Constant, primitive::PrimitiveKind},
    runtime_builtin::reserved_tokens::check_if_reserved_token,
    utils::InputSpan,
};

pub trait TypeCheckable {
    fn type_check(&self, context: &mut TypeCheckerContext) -> Result<(), TransformError>;
    fn populate_token_type_map(&self, context: &mut TypeCheckerContext);
}

pub trait WithType {
    fn get_type(&self, context: &TypeCheckerContext) -> PrimitiveKind;
}

#[derive(Debug, Serialize)]
#[wasm_bindgen]
pub struct TypedToken {
    span: InputSpan,
    value: PrimitiveKind,
    identifier: Option<String>,
}
#[wasm_bindgen(typescript_custom_section)]
const ITypedToken: &'static str = r#"
export type SerializedTypedToken = {
    span: InputSpan,
    value: SerializedPrimitiveKind,
    identifier?: string
}
"#;

impl TypedToken {
    pub fn new(span: InputSpan, value: PrimitiveKind, identifier: Option<String>) -> Self {
        Self {
            span,
            value,
            identifier,
        }
    }
}

pub struct TypeCheckerContext {
    frames: Vec<Frame<PrimitiveKind>>,
    token_map: HashMap<usize, TypedToken>,
}
impl Default for TypeCheckerContext {
    fn default() -> Self {
        let primitives = HashMap::new();
        let token_map = HashMap::new();
        Self::new(primitives, token_map)
    }
}
impl TypeCheckerContext {
    pub fn new(
        primitives: HashMap<String, PrimitiveKind>,
        token_map: HashMap<usize, TypedToken>,
    ) -> Self {
        let frame = Frame::from_map(primitives);
        Self {
            frames: vec![frame],
            token_map,
        }
    }

    pub fn into_token_map(self) -> HashMap<usize, TypedToken> {
        self.token_map
    }
    pub fn add_scope(&mut self) {
        let frame = Frame::new();
        self.frames.push(frame);
    }
    pub fn add_token_type(
        &mut self,
        value: PrimitiveKind,
        span: InputSpan,
        identifier: Option<String>,
    ) -> Result<(), TransformError> {
        let start = span.start;
        if let Some(val) = &identifier {
            self.declare_variable(&val, value.clone(), false)
                .map_err(|e| e.to_spanned_error(&span))?;
        }
        let token = TypedToken::new(span, value, identifier);
        self.token_map.insert(start, token);
        Ok(())
    }
    pub fn add_token_type_or_undefined(
        &mut self,
        value: PrimitiveKind,
        span: InputSpan,
        identifier: Option<String>,
    ) {
        let start = span.start;
        if let Some(val) = &identifier {
            self.declare_variable(&val, value.clone(), false)
                .unwrap_or(());
        }
        let token = TypedToken::new(span, value, identifier);
        self.token_map.insert(start, token);
    }
    pub fn pop_scope(&mut self) -> Result<Frame<PrimitiveKind>, TransformError> {
        if self.frames.len() <= 1 {
            return Err(TransformError::Other("Missing frame to pop".to_string()));
        }
        Ok(self.frames.pop().unwrap())
    }
    pub fn get_value(&self, name: &str) -> Option<&PrimitiveKind> {
        for frame in self.frames.iter().rev() {
            match frame.get_value(name) {
                Some(value) => return Some(value),
                None => continue,
            }
        }
        None
    }
    pub fn check_compound_variable(&self, compound_name: &[String]) -> Result<(), TransformError> {
        for name in compound_name {
            let value = self.get_value(name);
            if value.is_none() {
                return Err(TransformError::UndeclaredVariable(name.to_string()));
            }
            let value = value.unwrap();
            match value {
                PrimitiveKind::Number
                | PrimitiveKind::Integer
                | PrimitiveKind::PositiveInteger
                | PrimitiveKind::String
                | PrimitiveKind::GraphNode => {}
                _ => {
                    return Err(TransformError::WrongExpectedArgument {
                        got: value.clone(),
                        one_of: vec![
                            PrimitiveKind::Number,
                            PrimitiveKind::Integer,
                            PrimitiveKind::PositiveInteger,
                            PrimitiveKind::String,
                            PrimitiveKind::GraphNode,
                        ],
                    })
                }
            }
        }
        Ok(())
    }
    pub fn declare_variable(
        &mut self,
        name: &str,
        value: PrimitiveKind,
        strict: bool,
    ) -> Result<(), TransformError> {
        if name == "_" {
            return Ok(());
        }
        if strict && self.get_value(name).is_some() {
            return Err(TransformError::AlreadyDeclaredVariable(name.to_string()));
        }
        check_if_reserved_token(name)?;
        let frame = self.frames.last_mut().unwrap();
        frame.declare_variable(name, value)
    }
    pub fn get_addressable_value(
        &self,
        addressable_access: &AddressableAccess,
    ) -> Result<PrimitiveKind, TransformError> {
        //TODO add support for object access like G["a"] or g.a
        match self.get_value(&addressable_access.name) {
            Some(v) => {
                let mut last_value = v;
                for access in addressable_access.accesses.iter() {
                    if !access.get_type(self).is_numeric() {
                        //TODO this is a relaxed check, the runtime will check for the exact type
                        return Err(TransformError::Other(format!(
                            "Expected value of type \"Number\" to index array, got \"{}\", check the definition of \"{}\"",
                            access.get_type(self).to_string(),
                            access.to_string()
                        )));
                    }
                    match last_value {
                        PrimitiveKind::Iterable(i) => {
                            last_value = i
                        }
                        _ => return Err(TransformError::Other(format!(
                            "Expected value of type \"Iterable\" to index array, got \"{}\", check the definition of \"{}\"",
                            last_value.to_string(),
                            access.to_string()
                        )).to_spanned_error(access.get_span()))
                    }
                }
                Ok(last_value.clone())
            }
            None => Err(TransformError::UndeclaredVariable(
                addressable_access.name.to_string(),
            )),
        }
    }
}
