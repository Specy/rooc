#[allow(unused_imports)]
use crate::prelude::*;
use indexmap::IndexMap;
use serde::Serialize;

use crate::math::PreVariableType;
use crate::parser::il::AddressableAccess;
use crate::parser::il::PreExp;
use crate::parser::model_transformer::Frame;
use crate::parser::model_transformer::TransformError;
use crate::runtime_builtin::RoocFunction;
use crate::utils::Spanned;
use crate::{
    primitives::PrimitiveKind, runtime_builtin::check_if_reserved_token, utils::InputSpan,
};

/// Trait for types that can be type checked within a context.
pub trait TypeCheckable {
    /// Performs type checking on the implementing type.
    ///
    /// # Arguments
    /// * `context` - Mutable reference to the type checker context
    /// * `fn_context` - Reference to the function context
    ///
    /// # Returns
    /// * `Ok(())` if type checking succeeds
    /// * `Err(TransformError)` if type checking fails
    fn type_check(
        &self,
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError>;

    /// Populates the token type map with type information.
    ///
    /// # Arguments
    /// * `context` - Mutable reference to the type checker context
    /// * `fn_context` - Reference to the function context
    fn populate_token_type_map(
        &self,
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    );
}

/// Trait for types that have an associated primitive type.
pub trait WithType {
    /// Gets the primitive type of the implementing type.
    ///
    /// # Arguments
    /// * `context` - Reference to the type checker context
    /// * `fn_context` - Reference to the function context
    ///
    /// # Returns
    /// The primitive kind associated with this type
    fn get_type(&self, context: &TypeCheckerContext, fn_context: &FunctionContext)
    -> PrimitiveKind;
}

/// Represents a token with type information and optional identifier.
#[derive(Debug, Serialize)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct TypedToken {
    span: InputSpan,
    value: PrimitiveKind,
    identifier: Option<String>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
const ITypedToken: &'static str = r#"
export type SerializedTypedToken = {
    span: InputSpan,
    value: SerializedPrimitiveKind,
    identifier?: string
}
"#;

impl TypedToken {
    /// Creates a new TypedToken.
    ///
    /// # Arguments
    /// * `span` - The input span of the token
    /// * `value` - The primitive type of the token
    /// * `identifier` - Optional identifier name
    pub fn new(span: InputSpan, value: PrimitiveKind, identifier: Option<String>) -> Self {
        Self {
            span,
            value,
            identifier,
        }
    }
}

/// Represents a variable type with associated source location information.
pub struct StaticVariableType {
    pub value: PreVariableType,
    pub span: InputSpan,
}

impl StaticVariableType {
    /// Creates a new StaticVariableType.
    ///
    /// # Arguments
    /// * `value` - The pre-variable type
    /// * `span` - The input span where this type was declared
    pub fn new(value: PreVariableType, span: InputSpan) -> Self {
        Self { value, span }
    }

    /// Creates a new StaticVariableType from a spanned pre-variable type.
    ///
    /// # Arguments
    /// * `value` - The spanned pre-variable type
    pub fn new_spanned(value: Spanned<PreVariableType>) -> Self {
        let (value, span) = value.into_tuple();
        Self { value, span }
    }
}

/// Context for function resolution during type checking.
pub struct FunctionContext<'a> {
    functions: &'a IndexMap<String, Box<dyn RoocFunction>>,
    builtin_functions: &'a IndexMap<String, Box<dyn RoocFunction>>,
}

impl<'a> FunctionContext<'a> {
    /// Creates a new FunctionContext.
    ///
    /// # Arguments
    /// * `functions` - Map of user-defined functions
    /// * `builtin_functions` - Map of built-in functions
    pub fn new(
        functions: &'a IndexMap<String, Box<dyn RoocFunction>>,
        builtin_functions: &'a IndexMap<String, Box<dyn RoocFunction>>,
    ) -> Self {
        Self {
            functions,
            builtin_functions,
        }
    }

    /// Looks up a function by name, checking built-ins first then user-defined functions.
    ///
    /// # Arguments
    /// * `name` - Name of the function to look up
    ///
    /// # Returns
    /// Reference to the function if found, None otherwise
    pub fn function(&self, name: &str) -> Option<&dyn RoocFunction> {
        match self.builtin_functions.get(name).map(|f| f.as_ref()) {
            Some(f) => Some(f),
            None => self.functions.get(name).map(|f| f.as_ref()),
        }
    }
}

/// Main context for type checking, maintaining variable scopes and type information.
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
    /// Creates a new TypeCheckerContext.
    ///
    /// # Arguments
    /// * `primitives` - Initial primitive type mappings
    /// * `token_map` - Initial token type mappings
    /// * `static_domain` - Initial static variable types
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

    /// Consumes the context and returns the token type map.
    pub fn into_token_map(self) -> IndexMap<u32, TypedToken> {
        self.token_map
    }

    /// Adds a new scope frame to the context.
    pub fn add_scope(&mut self) {
        let frame = Frame::new();
        self.frames.push(frame);
    }

    /// Adds a typed token to the context.
    ///
    /// # Arguments
    /// * `value` - The primitive type of the token
    /// * `span` - The input span of the token
    /// * `identifier` - Optional identifier name
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(TransformError)` if there's an error declaring the variable
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

    /// Adds a typed token to the context, ignoring declaration errors.
    ///
    /// # Arguments
    /// * `value` - The primitive type of the token
    /// * `span` - The input span of the token
    /// * `identifier` - Optional identifier name
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

    /// Sets the static domain for the context.
    ///
    /// # Arguments
    /// * `domain` - Vector of name/type pairs to set as the static domain
    pub fn set_static_domain(&mut self, domain: Vec<(String, Spanned<PreVariableType>)>) {
        self.static_domain = IndexMap::from_iter(domain.into_iter().map(|(k, v)| {
            let (v, span) = v.into_tuple();
            (k, StaticVariableType::new(v, span))
        }));
    }

    /// Gets a static variable type by name.
    ///
    /// # Arguments
    /// * `name` - Name of the static variable
    ///
    /// # Returns
    /// Reference to the StaticVariableType if found, None otherwise
    pub fn static_domain_variable_of(&self, name: &str) -> Option<&StaticVariableType> {
        self.static_domain.get(name)
    }

    /// Removes and returns the top scope frame.
    ///
    /// # Returns
    /// * `Ok(Frame)` containing the popped frame
    /// * `Err(TransformError)` if there are no frames to pop
    pub fn pop_scope(&mut self) -> Result<Frame<PrimitiveKind>, TransformError> {
        if self.frames.len() <= 1 {
            return Err(TransformError::Other("Missing frame to pop".to_string()));
        }
        Ok(self.frames.pop().unwrap())
    }

    /// Looks up a variable's type by name, searching through all scopes.
    ///
    /// # Arguments
    /// * `name` - Name of the variable to look up
    ///
    /// # Returns
    /// Reference to the PrimitiveKind if found, None otherwise
    pub fn value_of(&self, name: &str) -> Option<&PrimitiveKind> {
        for frame in self.frames.iter().rev() {
            match frame.value(name) {
                Some(value) => return Some(value),
                None => continue,
            }
        }
        None
    }

    /// Type checks compound variable indexes.
    ///
    /// # Arguments
    /// * `compound_indexes` - Slice of expressions used as indexes
    /// * `fn_context` - Reference to the function context
    ///
    /// # Returns
    /// * `Ok(())` if all indexes are valid
    /// * `Err(TransformError)` if any index is invalid
    pub fn check_compound_variable(
        &mut self,
        compound_indexes: &[PreExp],
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        for index in compound_indexes {
            index.type_check(self, fn_context)?;
            let value = match index {
                PreExp::Variable(v) => self.value_of(v).cloned(),
                PreExp::Primitive(p) => Some(p.value.get_type()),
                _ => Some(index.get_type(self, fn_context)),
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
                    .add_span(index.span()));
                }
            }
        }
        Ok(())
    }

    /// Declares a new variable in the current scope.
    ///
    /// # Arguments
    /// * `name` - Name of the variable
    /// * `value` - Type of the variable
    /// * `strict` - Whether to error on redeclaration
    ///
    /// # Returns
    /// * `Ok(())` if declaration succeeds
    /// * `Err(TransformError)` if declaration fails
    pub fn declare_variable(
        &mut self,
        name: &str,
        value: PrimitiveKind,
        strict: bool,
    ) -> Result<(), TransformError> {
        if name == "_" {
            return Ok(());
        }
        if strict && self.value_of(name).is_some() {
            return Err(TransformError::AlreadyDeclaredVariable(name.to_string()));
        }
        check_if_reserved_token(name)?;
        let frame = self.frames.last_mut().unwrap();
        frame.declare_variable(name, value)
    }

    /// Gets the type of an addressable value (e.g. array access).
    ///
    /// # Arguments
    /// * `addressable_access` - The addressable access expression
    /// * `fn_context` - Reference to the function context
    ///
    /// # Returns
    /// * `Ok(PrimitiveKind)` with the resolved type
    /// * `Err(TransformError)` if type resolution fails
    pub fn get_addressable_value(
        &self,
        addressable_access: &AddressableAccess,
        fn_context: &FunctionContext,
    ) -> Result<PrimitiveKind, TransformError> {
        //TODO add support for object access like G["a"] or g.a
        match self.value_of(&addressable_access.name) {
            Some(v) => {
                let mut last_value = v;
                for access in addressable_access.accesses.iter() {
                    if !access.get_type(self, fn_context).is_numeric() {
                        //TODO this is a relaxed check, the runtime will check for the exact type
                        return Err(TransformError::Other(format!(
                            "Expected value of type \"Number\" to index array, got \"{}\", check the definition of \"{}\"",
                            access.get_type(self, fn_context),
                            access
                        )));
                    }
                    match last_value {
                        PrimitiveKind::Iterable(i) => {
                            last_value = i
                        }
                        _ => return Err(TransformError::Other(format!(
                            "Expected value of type \"Iterable\" to index, got \"{}\", check the definition of \"{}\"",
                            last_value,
                            addressable_access
                        )).add_span(access.span()))
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
