use indexmap::IndexMap;
use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::math::math_enums::PreVariableType;
use crate::parser::il::il_exp::PreExp;
use crate::parser::il::il_problem::AddressableAccess;
use crate::parser::model_transformer::transform_error::TransformError;
use crate::parser::model_transformer::transformer_context::Frame;
use crate::utils::Spanned;
use crate::{
    primitives::primitive::PrimitiveKind,
    runtime_builtin::reserved_tokens::check_if_reserved_token, utils::InputSpan,
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

pub struct StaticVariableType {
    pub value: PreVariableType,
    pub span: InputSpan,
}

impl StaticVariableType {
    pub fn new(value: PreVariableType, span: InputSpan) -> Self {
        Self { value, span }
    }
    pub fn new_spanned(value: Spanned<PreVariableType>) -> Self {
        let (value, span) = value.into_tuple();
        Self { value, span }
    }
}

pub struct TypeCheckerContext {
    frames: Vec<Frame<PrimitiveKind>>,
    static_domain: IndexMap<String, StaticVariableType>,
    token_map: IndexMap<u32, TypedToken>,
}

impl Default for TypeCheckerContext {
    fn default() -> Self {
        let primitives = IndexMap::new();
        let token_map = IndexMap::new();
        let static_domain = IndexMap::new();
        Self::new(primitives, token_map, static_domain)
    }
}

impl TypeCheckerContext {
    pub fn new(
        primitives: IndexMap<String, PrimitiveKind>,
        token_map: IndexMap<u32, TypedToken>,
        static_domain: IndexMap<String, StaticVariableType>,
    ) -> Self {
        let frame = Frame::from_map(primitives);
        Self {
            frames: vec![frame],
            token_map,
            static_domain,
        }
    }

    pub fn into_token_map(self) -> IndexMap<u32, TypedToken> {
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
            self.declare_variable(val, value.clone(), true)
                .map_err(|e| e.add_span(&span))?;
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
            self.declare_variable(val, value.clone(), true)
                .unwrap_or(());
        }
        let token = TypedToken::new(span, value, identifier);
        self.token_map.insert(start, token);
    }
    pub fn set_static_domain(&mut self, domain: Vec<(String, Spanned<PreVariableType>)>) {
        self.static_domain = IndexMap::from_iter(domain.into_iter().map(|(k, v)| {
            let (v, span) = v.into_tuple();
            (k, StaticVariableType::new(v, span))
        }));
    }
    pub fn get_static_domain_variable(&self, name: &str) -> Option<&StaticVariableType> {
        self.static_domain.get(name)
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
    pub fn check_compound_variable(
        &mut self,
        compound_indexes: &[PreExp],
    ) -> Result<(), TransformError> {
        for index in compound_indexes {
            index.type_check(self)?;
            let value = match index {
                PreExp::Variable(v) => self.get_value(v).cloned(),
                PreExp::Primitive(p) => Some(p.value.get_type()),
                _ => Some(index.get_type(self)),
            };
            if value.is_none() {
                return Err(TransformError::UndeclaredVariable(index.to_string()));
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
                    }
                    .add_span(index.get_span()));
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
                            access.get_type(self),
                            access
                        )));
                    }
                    match last_value {
                        PrimitiveKind::Iterable(i) => {
                            last_value = i
                        }
                        _ => return Err(TransformError::Other(format!(
                            "Expected value of type \"Iterable\" to index array, got \"{}\", check the definition of \"{}\"",
                            last_value,
                            access
                        )).add_span(access.get_span()))
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
