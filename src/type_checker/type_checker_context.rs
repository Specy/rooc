use std::collections::HashMap;

use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    parser::{
        pre_parsed_problem::AddressableAccess,
        transformer::{Frame, TransformError},
    },
    primitives::{consts::Constant, primitive::PrimitiveKind},
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
    pub fn new_from_constants(constants: &Vec<Constant>) -> Self {
        let primitives = constants
            .into_iter()
            .map(|c| (c.name.get_span_value().clone(), c.value.get_type()))
            .collect::<HashMap<_, _>>();
        let token_map = constants
            .into_iter()
            .map(|c| {
                (
                    c.name.get_span().start,
                    TypedToken::new(
                        c.name.get_span().clone(),
                        c.value.get_type(),
                        Some(c.name.get_span_value().clone()),
                    ),
                )
            })
            .collect::<HashMap<_, _>>();
        Self::new(primitives, token_map)
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
    ) {
        let start = span.start;
        if let Some(val) = &identifier {
            let _ = self.declare_variable(&val, value.clone(), false);
        }
        let token = TypedToken::new(span, value, identifier);
        self.token_map.insert(start, token);
    }
    pub fn add_frame(&mut self, frame: Frame<PrimitiveKind>) {
        self.frames.push(frame);
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
                return Err(TransformError::MissingVariable(name.to_string()));
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
        if strict && self.get_value(name).is_some() {
            return Err(TransformError::AlreadyExistingVariable(name.to_string()));
        }
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
            None => Err(TransformError::MissingVariable(
                addressable_access.name.to_string(),
            )),
        }
    }
}
