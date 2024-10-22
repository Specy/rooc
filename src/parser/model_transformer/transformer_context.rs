use indexmap::IndexMap;
use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::math::math_enums::VariableType;
use crate::parser::domain_declaration::VariablesDomainDeclaration;
use crate::parser::il::il_problem::AddressableAccess;
use crate::parser::model_transformer::transform_error::TransformError;
use crate::primitives::consts::Constant;
use crate::primitives::primitive::{Primitive, PrimitiveKind};
use crate::runtime_builtin::reserved_tokens::check_if_reserved_token;
use crate::utils::{InputSpan, Spanned};

#[derive(Debug)]
pub struct Frame<T> {
    pub variables: IndexMap<String, T>,
}

impl<T> Frame<T> {
    pub fn new() -> Self {
        Self {
            variables: IndexMap::new(),
        }
    }
    pub fn from_map(constants: IndexMap<String, T>) -> Self {
        Self {
            variables: constants,
        }
    }

    pub fn get_value(&self, name: &str) -> Option<&T> {
        self.variables.get(name)
    }
    pub fn declare_variable(&mut self, name: &str, value: T) -> Result<(), TransformError> {
        if self.has_variable(name) {
            return Err(TransformError::AlreadyDeclaredVariable(name.to_string()));
        }
        self.variables.insert(name.to_string(), value);
        Ok(())
    }
    pub fn update_variable(&mut self, name: &str, value: T) -> Result<(), TransformError> {
        if !self.has_variable(name) {
            return Err(TransformError::UndeclaredVariable(name.to_string()));
        }
        self.variables.insert(name.to_string(), value);
        Ok(())
    }
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }
    pub fn drop_variable(&mut self, name: &str) -> Result<T, TransformError> {
        if !self.variables.contains_key(name) {
            return Err(TransformError::UndeclaredVariable(name.to_string()));
        }
        let value = self.variables.shift_remove(name).unwrap();
        Ok(value)
    }
}

impl<T> Default for Frame<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DomainVariable {
    as_type: VariableType,
    span: InputSpan,
    usage_count: usize,
}
#[wasm_bindgen(typescript_custom_section)]
pub const IModel: &'static str = r#"
export interface DomainVariable {
    as_type: VariableType;
    span: InputSpan;
    usage_count: number;
}
"#;

impl DomainVariable {
    pub fn new(as_type: VariableType, span: InputSpan) -> Self {
        Self {
            as_type,
            span,
            usage_count: 0,
        }
    }
    pub fn increment_usage(&mut self) {
        self.usage_count += 1;
    }
    pub fn get_usage_count(&self) -> usize {
        self.usage_count
    }
    pub fn is_used(&self) -> bool {
        self.usage_count > 0
    }
    pub fn get_span(&self) -> &InputSpan {
        &self.span
    }
    pub fn get_type(&self) -> &VariableType {
        &self.as_type
    }
}

#[derive(Debug)]
pub struct TransformerContext {
    frames: Vec<Frame<Primitive>>,
    domain: IndexMap<String, DomainVariable>,
}

impl Default for TransformerContext {
    fn default() -> Self {
        let primitives = IndexMap::new();
        let domain = IndexMap::new();
        Self::new(primitives, domain)
    }
}

impl TransformerContext {
    pub fn new(
        primitives: IndexMap<String, Primitive>,
        domain: IndexMap<String, DomainVariable>,
    ) -> Self {
        let frame = Frame::from_map(primitives);
        Self {
            frames: vec![frame],
            domain,
        }
    }
    pub fn new_from_constants(
        constants: Vec<Constant>,
        domain: Vec<VariablesDomainDeclaration>,
    ) -> Result<Self, TransformError> {
        let mut context = Self::default();
        for constant in constants {
            let value = constant.as_primitive(&context)?;
            let name = constant.name.get_span_value();
            context.declare_variable(name, value, true)?; //TODO should this be strict or allow for redeclaration?
        }
        let computed_domain = domain
            .into_iter()
            .map(|d| d.compute_domain(&mut context))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        assert_no_duplicates_in_domain(&computed_domain)?;
        let computed_domain = computed_domain
            .into_iter()
            .map(|(name, as_type)| {
                let (as_type, span) = as_type.into_tuple();
                (name, DomainVariable::new(as_type, span))
            })
            .collect::<Vec<_>>();
        context.domain = IndexMap::from_iter(computed_domain);
        Ok(context)
    }

    pub fn flatten_variable_name(
        &self,
        compound_indexes: &[Primitive],
    ) -> Result<String, TransformError> {
        let flattened = compound_indexes
            .iter()
            .map(|value| match value {
                Primitive::Number(value) => Ok(value.to_string()),
                Primitive::Integer(value) => Ok(value.to_string()),
                Primitive::PositiveInteger(value) => Ok(value.to_string()),
                Primitive::Boolean(value) => Ok(if *value { "T" } else { "F" }.to_string()),
                Primitive::String(value) => Ok(value.clone()),
                Primitive::GraphNode(v) => Ok(v.get_name().clone()),
                _ => Err(TransformError::WrongExpectedArgument {
                    got: value.get_type(),
                    one_of: vec![
                        PrimitiveKind::Number,
                        PrimitiveKind::Integer,
                        PrimitiveKind::PositiveInteger,
                        PrimitiveKind::String,
                        PrimitiveKind::GraphNode,
                    ],
                }),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(flattened.join("_"))
    }

    pub fn add_populated_scope(&mut self, frame: Frame<Primitive>) {
        self.frames.push(frame);
    }
    pub fn replace_last_frame(&mut self, frame: Frame<Primitive>) {
        self.frames.pop();
        self.frames.push(frame);
    }
    pub fn add_scope(&mut self) {
        let frame = Frame::new();
        self.frames.push(frame);
    }
    pub fn pop_scope(&mut self) -> Result<Frame<Primitive>, TransformError> {
        if self.frames.len() <= 1 {
            return Err(TransformError::Other("Missing frame to pop".to_string()));
        }
        Ok(self.frames.pop().unwrap())
    }
    pub fn get_value(&self, name: &str) -> Option<&Primitive> {
        for frame in self.frames.iter().rev() {
            match frame.get_value(name) {
                Some(value) => return Some(value),
                None => continue,
            }
        }
        None
    }
    pub fn get_variable_domain(&self, name: &str) -> Option<&VariableType> {
        self.domain.get(name).map(|v| &v.as_type)
    }
    pub fn increment_domain_variable_usage(&mut self, name: &str) -> Result<(), TransformError> {
        match self.domain.get_mut(name) {
            Some(v) => {
                v.increment_usage();
                Ok(())
            }
            None => Err(TransformError::UndeclaredVariableDomain(name.to_string())),
        }
    }
    pub fn reset_domain(&mut self) {
        for (_, v) in self.domain.iter_mut() {
            v.usage_count = 0;
        }
    }
    pub fn get_used_domain_variables(&self) -> Vec<(&String, &VariableType)> {
        self.domain
            .iter()
            .filter(|(_, v)| v.is_used())
            .map(|(k, v)| (k, &v.as_type))
            .collect()
    }
    pub fn exists_variable(&self, name: &str, strict: bool) -> bool {
        if strict {
            for frame in self.frames.iter().rev() {
                if frame.has_variable(name) {
                    return true;
                }
            }
        } else {
            return match self.frames.last() {
                Some(frame) => frame.has_variable(name),
                None => false,
            };
        }
        false
    }
    pub fn declare_variable(
        &mut self,
        name: &str,
        value: Primitive,
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
    pub fn update_variable(&mut self, name: &str, value: Primitive) -> Result<(), TransformError> {
        if name == "_" {
            return Ok(());
        }
        for frame in self.frames.iter_mut().rev() {
            if frame.has_variable(name) {
                return frame.update_variable(name, value);
            }
        }
        Err(TransformError::UndeclaredVariable(name.to_string()))
    }
    pub fn remove_variable(&mut self, name: &str) -> Result<Primitive, TransformError> {
        if name == "_" {
            return Ok(Primitive::Undefined);
        }
        for frame in self.frames.iter_mut().rev() {
            if frame.has_variable(name) {
                return frame.drop_variable(name);
            }
        }
        Err(TransformError::UndeclaredVariable(name.to_string()))
    }

    pub fn flatten_compound_variable(
        &self,
        name: &String,
        indexes: &[Primitive],
    ) -> Result<String, TransformError> {
        let names: String = self.flatten_variable_name(indexes)?;
        let name = format!("{}_{}", name, names);
        Ok(name)
    }

    pub fn get_addressable_value(
        &self,
        addressable_access: &AddressableAccess,
    ) -> Result<Primitive, TransformError> {
        //TODO add support for object access like G["a"] or g.a
        match self.get_value(&addressable_access.name) {
            Some(a) => {
                let accesses = addressable_access
                    .accesses
                    .iter()
                    .map(|access| access.as_usize_cast(self))
                    .collect::<Result<Vec<_>, TransformError>>()?;
                let value = a.as_iterator()?.read(accesses)?;
                Ok(value)
            }
            None => Err(TransformError::UndeclaredVariable(
                addressable_access.name.to_string(),
            )),
        }
    }
    pub fn into_components(self) -> IndexMap<String, DomainVariable> {
        self.domain
    }
}

pub fn assert_no_duplicates_in_domain(
    domain: &[(String, Spanned<VariableType>)],
) -> Result<(), TransformError> {
    let acc: IndexMap<String, (i32, Spanned<VariableType>)> = IndexMap::new();
    let duplicates = domain
        .iter()
        .fold(acc, |mut acc, (name, as_type)| {
            if let Some((count, saved_type)) = acc.get_mut(name) {
                //ignore the type if it's the same
                if saved_type.get_span_value() == as_type.get_span_value() {
                    return acc;
                }
                *count += 1;
            } else {
                acc.insert(name.clone(), (1, as_type.clone()));
            }
            acc
        })
        .into_iter()
        .filter(|(_, (count, _))| *count > 1)
        .collect::<Vec<_>>();
    if !duplicates.is_empty() {
        let first_span = duplicates.first().unwrap().1 .1.get_span().clone();
        Err(TransformError::AlreadyDeclaredDomainVariable(duplicates).add_span(&first_span))
    } else {
        Ok(())
    }
}
