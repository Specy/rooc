use core::fmt;

#[allow(unused_imports)]
use crate::prelude::*;
use serde::Serialize;

use crate::math::VariableType;
use crate::math::{BinOp, UnOp};
use crate::parser::il::PreExp;
use crate::primitives::PrimitiveKind;
use crate::runtime_builtin::TokenType;
use crate::utils::{InputSpan, Spanned};

/// Represents errors that can occur during model transformation.
///
/// This enum contains various error types that may arise when transforming
/// a pre-model into a complete optimization model, including variable declaration errors,
/// type mismatches, function errors, and operator errors.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum TransformError {
    /// Error when a variable is used but not declared
    UndeclaredVariable(String),

    /// Error when a variable's domain is referenced but not declared
    UndeclaredVariableDomain(String),

    /// Error when attempting to declare a variable that already exists
    AlreadyDeclaredVariable(String),

    /// Error when attempting to declare a domain for a variable that already has one.
    /// Contains a vector of (variable name, (count, variable type)) tuples.
    AlreadyDeclaredDomainVariable(Vec<(String, (i32, Spanned<VariableType>))>),

    /// Error when a value is outside its valid range
    OutOfBounds(String),

    /// Error when an argument has the wrong type
    WrongArgument {
        /// The type that was provided
        got: PrimitiveKind,
        /// The type that was expected
        expected: PrimitiveKind,
    },

    /// Error when an argument's type doesn't match any of the allowed types
    WrongExpectedArgument {
        /// The type that was provided
        got: PrimitiveKind,
        /// List of allowed types
        one_of: Vec<PrimitiveKind>,
    },

    /// Error with source location information
    SpannedError {
        /// The underlying error with location info
        spanned_error: Spanned<Box<TransformError>>,
        /// Optional additional context
        value: Option<String>,
    },

    /// Error when referencing a function that doesn't exist
    NonExistentFunction(String),

    /// Error when calling a function with incorrect argument types
    WrongFunctionSignature {
        /// Expected parameter types
        signature: Vec<(String, PrimitiveKind)>,
        /// Actual argument types provided
        got: Vec<PrimitiveKind>,
    },

    /// Error when calling a function with wrong number of arguments
    WrongNumberOfArguments {
        /// Expected parameter types
        signature: Vec<(String, PrimitiveKind)>,
        /// Actual arguments provided
        args: Vec<PreExp>,
    },

    /// Error when binary operator cannot be applied to given types
    BinOpError {
        /// The binary operator
        operator: BinOp,
        /// Type of left operand
        lhs: PrimitiveKind,
        /// Type of right operand
        rhs: PrimitiveKind,
    },

    /// Error when unary operator cannot be applied to given type
    UnOpError {
        /// The unary operator
        operator: UnOp,
        /// Type of the operand
        exp: PrimitiveKind,
    },

    /// Error when attempting to spread a type that cannot be spread
    Unspreadable(PrimitiveKind),

    /// Error when spreading a type into incompatible variables
    SpreadError {
        /// Type being spread
        to_spread: PrimitiveKind,
        /// Names of target variables
        in_variables: Vec<String>,
    },

    /// Error when attempting to define something that already exists
    AlreadyDefined {
        /// Name of the item
        name: String,
        /// Type of the token
        kind: TokenType,
    },

    /// Error when a value exceeds maximum allowed size
    TooLarge {
        /// Description of the error
        message: String,
        /// Actual value
        got: i64,
        /// Maximum allowed value
        max: i64,
    },

    /// Generic error with custom message
    Other(String),
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
pub const ITransformError: &'static str = r#"
export type SerializedTransformError = {
    type: "UndeclaredVariable",
    value: string
} | {
    type: "AlreadyDeclaredVariable",
    value: string
} | {
    type: "OutOfBounds",
    value: string
} | {
    type: "WrongArgument",
    value: string
} | {
    type: "SpannedError",
    value: {
        spanned_error: SerializedSpanned<SerializedTransformError>,
        value?: string
    }
} | {
    type: "Unspreadable",
    value: string
} | {
    type: "Other",
    value: string
} | {
    type: "WrongNumberOfArguments",
    value: {
        signature: SerializedPrimitiveKind[],
        got: SerializedPrimitiveKind[]
    }
} | {
    type: "BinOpError",
    value: {
        operator: BinOp,
        lhs: SerializedPrimitiveKind,
        rhs: SerializedPrimitiveKind
    }
} | {
    type: "UnOpError",
    value: {
        operator: UnOp,
        exp: SerializedPrimitiveKind
    }
} | {
    type: "WrongExpectedArgument",
    value: {
        got: SerializedPrimitiveKind,
        one_of: SerializedPrimitiveKind[]
    }
} | {
    type: "WrongFunctionSignature",
    value: {
        signature: SerializedPrimitiveKind[],
        got: SerializedPrimitiveKind[]
    }
} | {
    type: "SpreadError",
    value: {
        to_spread: SerializedPrimitiveKind,
        in_variables: string[]
    }
} | {
    type: "AlreadyDefined",
    value: {
        name: string,
        kind: SerializedTokenType
    }
} | {
    type: "AlreadyDeclaredDomainVariable",
    value: {
        variables: [string, SerializedVariableKind][]
    }
} | {
    type: "UndeclaredVariableDomain",
    value: string
} | {
    type: "TooLarge",
    value: {
        message: string,
        got: number,
        max: number
    }
} | {
    type: "NonExistentFunction",
    value: string
}
"#;

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TransformError::UndeclaredVariable(name) => format!(
                "[UndeclaredVariable] Variable \"{}\" was not declared",
                name
            ),
            TransformError::AlreadyDeclaredVariable(name) => {
                format!(
                    "[AlreadyDeclaredVariable] Variable {} was already declared",
                    name
                )
            }
            TransformError::NonExistentFunction(name) => {
                format!("[NonExistentFunction] Function \"{}\" does not exist", name)
            }
            TransformError::UndeclaredVariableDomain(name) => {
                format!(
                    "[UndeclaredVariableDomain] The domain of variable \"{}\" was not defined",
                    name
                )
            }
            TransformError::WrongFunctionSignature { signature, got } => {
                format!(
                    "[WrongFunctionSignature] Wrong number of arguments, expected \"{}\", got \"{}\"",
                    signature
                        .iter()
                        .map(|x| format!("{}: {}", x.0, x.1))
                        .collect::<Vec<_>>()
                        .join(", "),
                    got.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            TransformError::WrongNumberOfArguments { args, signature } => {
                format!(
                    "[WrongNumberOfArguments] Wrong number of arguments, expected signature \"{}\", got parameters \"{}\"",
                    signature
                        .iter()
                        .map(|x| format!("{}: {}", x.0, x.1))
                        .collect::<Vec<_>>()
                        .join(", "),
                    args
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            TransformError::OutOfBounds(name) => format!("[OutOfBounds] {}", name),
            TransformError::WrongArgument { expected, got } => {
                format!("[WrongArgument] expected \"{}\", got \"{}\"", expected, got)
            }
            TransformError::WrongExpectedArgument { got, one_of } => {
                format!(
                    "[WrongExpectedArgument] expected one of \"{}\", got \"{}\"",
                    one_of
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    got
                )
            }
            TransformError::TooLarge { message, got, max } => {
                format!("[TooLarge] {}: got {}, max {}", message, got, max)
            }
            TransformError::AlreadyDefined { name, kind } => {
                format!(
                    "[AlreadyDefined] name \"{}\" is already defined as a \"{}\"",
                    name, kind
                )
            }
            TransformError::AlreadyDeclaredDomainVariable(variables) => {
                format!(
                    "[AlreadyDeclaredDomainVariable] There are some variables whose domain is already declared:\n    {}",
                    variables
                        .iter()
                        .map(|(name, kind)| format!("{}: {} ({} duplicates)", name, *kind.1, kind.0))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            TransformError::SpreadError {
                to_spread,
                in_variables,
            } => format!(
                "[SpreadError] type \"{}\" cannot be spread in \"{}\"",
                to_spread,
                in_variables
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            TransformError::Other(name) => format!("[Other] {}", name),
            TransformError::Unspreadable(kind) => {
                format!("[Unspreadable] type \"{}\" is not spreadable", kind)
            }
            TransformError::SpannedError {
                spanned_error: span,
                ..
            } => span.to_string(),
            TransformError::BinOpError { operator, lhs, rhs } => {
                format!(
                    "[BinOpError] operator \"{}\" cannot be applied to \"{}\" and \"{}\"",
                    operator, lhs, rhs
                )
            }
            TransformError::UnOpError { operator, exp } => {
                format!(
                    "[UnOpError] operator \"{}\" cannot be applied to \"{}\"",
                    operator, exp
                )
            }
        };
        f.write_str(&s)
    }
}

/// Provides utility methods for handling and formatting transform errors.
impl TransformError {
    /// Creates a detailed error message with stack trace information.
    ///
    /// # Returns
    /// A formatted string containing the error message followed by the stack trace
    pub fn traced_error(&self) -> String {
        let error = self.to_string();
        let trace = self.trace();
        let trace = trace
            .iter()
            .map(|(span, origin)| {
                let origin = if let Some(o) = origin {
                    format!(" ({})", o)
                } else {
                    "".to_string()
                };
                format!(
                    "\tat {}:{}\"{}\"",
                    span.start_line, span.start_column, origin
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("{}\n{}", error, trace)
    }

    /// Creates a type mismatch error with source location information.
    ///
    /// # Arguments
    /// * `expected` - The expected primitive type
    /// * `got` - The actual primitive type received
    /// * `span` - Location information for the error
    pub fn from_wrong_type(expected: PrimitiveKind, got: PrimitiveKind, span: InputSpan) -> Self {
        TransformError::WrongArgument { got, expected }.add_span(&span)
    }

    /// Creates a binary operator error with source location information.
    ///
    /// # Arguments
    /// * `operator` - The binary operator that failed
    /// * `lhs` - Type of the left-hand operand
    /// * `rhs` - Type of the right-hand operand
    /// * `span` - Location information for the error
    pub fn from_wrong_binop(
        operator: BinOp,
        lhs: PrimitiveKind,
        rhs: PrimitiveKind,
        span: InputSpan,
    ) -> Self {
        TransformError::BinOpError { operator, lhs, rhs }.add_span(&span)
    }

    /// Creates a unary operator error with source location information.
    ///
    /// # Arguments
    /// * `operator` - The unary operator that failed
    /// * `exp` - Type of the operand
    /// * `span` - Location information for the error
    pub fn from_wrong_unop(operator: UnOp, exp: PrimitiveKind, span: InputSpan) -> Self {
        TransformError::UnOpError { operator, exp }.add_span(&span)
    }

    /// Adds source location information to an existing error.
    ///
    /// # Arguments
    /// * `span` - Location information to add
    ///
    /// # Returns
    /// A new error with the added span information
    pub fn add_span(self, span: &InputSpan) -> TransformError {
        TransformError::SpannedError {
            spanned_error: Spanned::new(Box::new(self), span.clone()),
            value: None,
        }
    }

    /// Gets the stack trace of nested errors.
    ///
    /// # Returns
    /// A vector of spans and optional origin strings representing the error trace
    pub fn trace(&self) -> Vec<(InputSpan, Option<String>)> {
        match self {
            TransformError::SpannedError {
                spanned_error: span,
                value,
            } => {
                let mut trace = vec![(span.span().clone(), value.clone())];
                let mut last_error = span;
                while let TransformError::SpannedError {
                    spanned_error: ref span,
                    ref value,
                } = **last_error.value()
                {
                    let current_span = span.span().clone();
                    //don't add if the last span is the same as the current one
                    if let Some((last_span, _)) = trace.last() {
                        if last_span == &current_span {
                            last_error = span;
                            continue;
                        }
                    }
                    trace.push((current_span, value.clone()));
                    last_error = span;
                }
                trace.reverse();
                trace
            }
            _ => Vec::new(),
        }
    }

    /// Gets the source location of the original error.
    ///
    /// # Returns
    /// The span of the first error in the trace, if any
    pub fn origin_span(&self) -> Option<InputSpan> {
        let trace = self.trace();
        trace.first().map(|(span, _)| span.clone())
    }

    /// Gets the root error, stripping any span information.
    ///
    /// # Returns
    /// A reference to the underlying error without span information
    pub fn base_error(&self) -> &TransformError {
        match self {
            TransformError::SpannedError {
                spanned_error: span,
                ..
            } => span.base_error(),
            _ => self,
        }
    }

    /// Creates a detailed error message with source code snippets.
    ///
    /// # Arguments
    /// * `source` - The source code text
    ///
    /// # Returns
    /// A formatted error message with relevant code snippets, or an error if the spans are invalid
    pub fn trace_from_source(&self, source: &str) -> Result<String, String> {
        let trace = self.trace();
        let trace = trace
            .into_iter()
            .map(|(span, _)| {
                let text = span.span_text(source)?;
                Ok(format!(
                    "at {}:{} \"{}\"",
                    span.start_line, span.start_column, text,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let join = trace.join("\n\t");
        Ok(format!("{}\n\t{}", self, join))
    }
}
