#[allow(unused_imports)]
use crate::prelude::*;
use indexmap::IndexMap;
use serde::Serialize;

use crate::math::VariableType;
use crate::parser::domain_declaration::VariablesDomainDeclaration;
use crate::parser::il::AddressableAccess;
use crate::parser::model_transformer::transform_error::TransformError;
use crate::primitives::Constant;
use crate::primitives::{Primitive, PrimitiveKind};
use crate::runtime_builtin::check_if_reserved_token;
use crate::type_checker::type_checker_context::FunctionContext;
use crate::utils::{InputSpan, Spanned};

/// Represents a single scope frame containing variable bindings.
/// Used to implement variable scoping and shadowing.
#[derive(Debug)]
pub struct Frame<T> {
    pub variables: IndexMap<String, T>,
}

impl<T> Frame<T> {
    /// Creates a new empty frame.
    pub fn new() -> Self {
        Self {
            variables: IndexMap::new(),
        }
    }

    /// Creates a new frame initialized with the given variable bindings.
    ///
    /// # Arguments
    /// * `constants` - Initial variable bindings to populate the frame with
    pub fn from_map(constants: IndexMap<String, T>) -> Self {
        Self {
            variables: constants,
        }
    }

    /// Looks up the value of a variable in this frame.
    ///
    /// # Arguments
    /// * `name` - Name of the variable to look up
    ///
    /// # Returns
    /// * `Some(&T)` if the variable exists in this frame
    /// * `None` if the variable is not found
    pub fn value(&self, name: &str) -> Option<&T> {
        self.variables.get(name)
    }

    /// Declares a new variable in this frame.
    ///
    /// # Arguments
    /// * `name` - Name of the variable to declare
    /// * `value` - Value to bind to the variable
    ///
    /// # Returns
    /// * `Ok(())` if declaration succeeds
    /// * `Err(TransformError)` if variable already exists
    pub fn declare_variable(&mut self, name: &str, value: T) -> Result<(), TransformError> {
        if self.has_variable(name) {
            return Err(TransformError::AlreadyDeclaredVariable(name.to_string()));
        }
        self.variables.insert(name.to_string(), value);
        Ok(())
    }

    /// Updates the value of an existing variable.
    ///
    /// # Arguments
    /// * `name` - Name of variable to update
    /// * `value` - New value to assign
    ///
    /// # Returns
    /// * `Ok(())` if update succeeds
    /// * `Err(TransformError)` if variable doesn't exist
    pub fn update_variable(&mut self, name: &str, value: T) -> Result<(), TransformError> {
        if !self.has_variable(name) {
            return Err(TransformError::UndeclaredVariable(name.to_string()));
        }
        self.variables.insert(name.to_string(), value);
        Ok(())
    }

    /// Checks if a variable exists in this frame.
    ///
    /// # Arguments
    /// * `name` - Name of variable to check
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// Removes a variable from this frame and returns its value.
    ///
    /// # Arguments
    /// * `name` - Name of variable to remove
    ///
    /// # Returns
    /// * `Ok(T)` containing the removed value if successful
    /// * `Err(TransformError)` if variable doesn't exist
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

/// Represents a variable in the domain of a model, tracking its type and usage.
#[derive(Debug, Clone, Serialize)]
pub struct DomainVariable {
    as_type: VariableType,
    span: InputSpan,
    usage_count: usize,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
pub const IModel: &'static str = r#"
export interface DomainVariable {
    as_type: VariableType;
    span: InputSpan;
    usage_count: number;
}
"#;

impl DomainVariable {
    /// Creates a new domain variable with the given type and source location.
    ///
    /// # Arguments
    /// * `as_type` - The variable's type
    /// * `span` - Source code location information
    pub fn new(as_type: VariableType, span: InputSpan) -> Self {
        Self {
            as_type,
            span,
            usage_count: 0,
        }
    }

    /// Increments the usage count of this variable.
    pub fn increment_usage(&mut self) {
        self.usage_count += 1;
    }

    /// Returns the number of times this variable has been used.
    pub fn usage_count(&self) -> usize {
        self.usage_count
    }

    /// Returns whether this variable has been used at least once.
    pub fn is_used(&self) -> bool {
        self.usage_count > 0
    }

    /// Returns the source location information for this variable.
    pub fn span(&self) -> &InputSpan {
        &self.span
    }

    /// Returns the type of this variable.
    pub fn get_type(&self) -> &VariableType {
        &self.as_type
    }
}

/// Maintains the context for transforming a model, including variable scopes and domains.
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
    /// Creates a new transformer context with initial variable bindings and domains.
    ///
    /// # Arguments
    /// * `primitives` - Initial variable bindings
    /// * `domain` - Initial variable domains
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

    /// Creates a new transformer context from constants and domain declarations.
    ///
    /// # Arguments
    /// * `constants` - List of constants to initialize
    /// * `domain` - List of domain declarations
    /// * `fn_context` - Function context for evaluating expressions
    ///
    /// # Returns
    /// * `Ok(Self)` if initialization succeeds
    /// * `Err(TransformError)` if there are duplicate or invalid declarations
    pub fn new_from_constants<'a>(
        constants: Vec<Constant>,
        domain: Vec<VariablesDomainDeclaration>,
        fn_context: &FunctionContext,
    ) -> Result<Self, TransformError> {
        let mut context = Self::default();

        for constant in constants {
            let value = constant.as_primitive(&context, &fn_context)?;
            let name = constant.name.value();
            context.declare_variable(name, value, true)?; //TODO should this be strict or allow for redeclaration?
        }
        let computed_domain = domain
            .into_iter()
            .map(|d| d.compute_domain(&mut context, &fn_context))
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

    /// Flattens a list of primitive values into a single string identifier.
    ///
    /// # Arguments
    /// * `compound_indexes` - List of primitive values to flatten
    ///
    /// # Returns
    /// * `Ok(String)` containing the flattened identifier
    /// * `Err(TransformError)` if any values have invalid types
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
                Primitive::GraphNode(v) => Ok(v.name().clone()),
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

    /// Adds a new scope frame with existing variable bindings.
    ///
    /// # Arguments
    /// * `frame` - Frame containing variable bindings to add
    pub fn add_populated_scope(&mut self, frame: Frame<Primitive>) {
        self.frames.push(frame);
    }

    /// Replaces the current scope frame with a new one.
    ///
    /// # Arguments
    /// * `frame` - New frame to replace the current one with
    pub fn replace_last_frame(&mut self, frame: Frame<Primitive>) {
        self.frames.pop();
        self.frames.push(frame);
    }

    /// Adds a new empty scope frame.
    pub fn add_scope(&mut self) {
        let frame = Frame::new();
        self.frames.push(frame);
    }

    /// Removes and returns the current scope frame.
    ///
    /// # Returns
    /// * `Ok(Frame)` containing the removed frame if successful
    /// * `Err(TransformError)` if there is only one frame remaining
    pub fn pop_scope(&mut self) -> Result<Frame<Primitive>, TransformError> {
        if self.frames.len() <= 1 {
            return Err(TransformError::Other("Missing frame to pop".to_string()));
        }
        Ok(self.frames.pop().unwrap())
    }

    /// Looks up a variable's value across all scope frames.
    ///
    /// # Arguments
    /// * `name` - Name of variable to look up
    ///
    /// # Returns
    /// * `Some(&Primitive)` if variable is found
    /// * `None` if variable doesn't exist
    pub fn value(&self, name: &str) -> Option<&Primitive> {
        for frame in self.frames.iter().rev() {
            match frame.value(name) {
                Some(value) => return Some(value),
                None => continue,
            }
        }
        None
    }

    /// Gets the domain type of a variable.
    ///
    /// # Arguments
    /// * `name` - Name of variable to look up
    ///
    /// # Returns
    /// * `Some(&VariableType)` if variable has a domain
    /// * `None` if variable has no domain
    pub fn variable_domain(&self, name: &str) -> Option<&VariableType> {
        self.domain.get(name).map(|v| &v.as_type)
    }

    /// Increments the usage count for a domain variable.
    ///
    /// # Arguments
    /// * `name` - Name of variable to increment
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(TransformError)` if variable has no domain
    pub fn increment_domain_variable_usage(&mut self, name: &str) -> Result<(), TransformError> {
        match self.domain.get_mut(name) {
            Some(v) => {
                v.increment_usage();
                Ok(())
            }
            None => Err(TransformError::UndeclaredVariableDomain(name.to_string())),
        }
    }

    /// Resets the usage count for all domain variables to zero.
    pub fn reset_domain(&mut self) {
        for (_, v) in self.domain.iter_mut() {
            v.usage_count = 0;
        }
    }

    /// Returns a list of all used domain variables and their types.
    pub fn used_domain_variables(&self) -> Vec<(&String, &VariableType)> {
        self.domain
            .iter()
            .filter(|(_, v)| v.is_used())
            .map(|(k, v)| (k, &v.as_type))
            .collect()
    }

    /// Checks if a variable exists in any scope frame.
    ///
    /// # Arguments
    /// * `name` - Name of variable to check
    /// * `strict` - If true, checks all frames; if false, only checks current frame
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

    /// Declares a new variable in the current scope frame.
    ///
    /// # Arguments
    /// * `name` - Name of variable to declare
    /// * `value` - Value to bind to the variable
    /// * `strict` - If true, fails if variable exists in any frame
    ///
    /// # Returns
    /// * `Ok(())` if declaration succeeds
    /// * `Err(TransformError)` if variable already exists or name is reserved
    pub fn declare_variable(
        &mut self,
        name: &str,
        value: Primitive,
        strict: bool,
    ) -> Result<(), TransformError> {
        if name == "_" {
            return Ok(());
        }
        if strict && self.value(name).is_some() {
            return Err(TransformError::AlreadyDeclaredVariable(name.to_string()));
        }
        check_if_reserved_token(name)?;
        let frame = self.frames.last_mut().unwrap();
        frame.declare_variable(name, value)
    }

    /// Updates an existing variable in any scope frame.
    ///
    /// # Arguments
    /// * `name` - Name of variable to update
    /// * `value` - New value to assign
    ///
    /// # Returns
    /// * `Ok(())` if update succeeds
    /// * `Err(TransformError)` if variable doesn't exist
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

    /// Removes a variable from any scope frame.
    ///
    /// # Arguments
    /// * `name` - Name of variable to remove
    ///
    /// # Returns
    /// * `Ok(Primitive)` containing the removed value if successful
    /// * `Err(TransformError)` if variable doesn't exist
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

    /// Creates a flattened variable name from a base name and list of indexes.
    ///
    /// # Arguments
    /// * `name` - Base variable name
    /// * `indexes` - List of index values to append
    ///
    /// # Returns
    /// * `Ok(String)` containing the flattened name
    /// * `Err(TransformError)` if flattening fails
    pub fn flatten_compound_variable(
        &self,
        name: &String,
        indexes: &[Primitive],
    ) -> Result<String, TransformError> {
        let names: String = self.flatten_variable_name(indexes)?;
        let name = format!("{}_{}", name, names);
        Ok(name)
    }

    /// Gets the value of an addressable variable access.
    ///
    /// # Arguments
    /// * `addressable_access` - The variable access to evaluate
    /// * `fn_context` - Function context for evaluating expressions
    ///
    /// # Returns
    /// * `Ok(Primitive)` containing the accessed value if successful
    /// * `Err(TransformError)` if access fails
    pub fn addressable_value(
        &self,
        addressable_access: &AddressableAccess,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        //TODO add support for object access like G["a"] or g.a
        match self.value(&addressable_access.name) {
            Some(a) => {
                let accesses = addressable_access
                    .accesses
                    .iter()
                    .map(|access| access.as_usize_cast(self, fn_context))
                    .collect::<Result<Vec<_>, TransformError>>()?;
                let value = a.as_iterator()?.read(accesses)?;
                Ok(value)
            }
            None => Err(TransformError::UndeclaredVariable(
                addressable_access.name.to_string(),
            )),
        }
    }

    /// Consumes the context and returns its domain map.
    pub fn into_components(self) -> IndexMap<String, DomainVariable> {
        self.domain
    }
}

/// Checks for duplicate variable declarations in a domain.
///
/// # Arguments
/// * `domain` - List of variable declarations to check
///
/// # Returns
/// * `Ok(())` if no duplicates found
/// * `Err(TransformError)` if duplicates exist
pub(crate) fn assert_no_duplicates_in_domain(
    domain: &[(String, Spanned<VariableType>)],
) -> Result<(), TransformError> {
    let acc: IndexMap<String, (i32, Spanned<VariableType>)> = IndexMap::new();
    let duplicates = domain
        .iter()
        .fold(acc, |mut acc, (name, as_type)| {
            if let Some((count, saved_type)) = acc.get_mut(name) {
                //ignore the type if it's the same
                if saved_type.value() == as_type.value() {
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
        let first_span = duplicates.first().unwrap().1 .1.span().clone();
        Err(TransformError::AlreadyDeclaredDomainVariable(duplicates).add_span(&first_span))
    } else {
        Ok(())
    }
}
