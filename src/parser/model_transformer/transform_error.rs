use core::fmt;

use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::math::math_enums::VariableType;
use crate::math::operators::{BinOp, UnOp};
use crate::parser::il::il_exp::PreExp;
use crate::primitives::primitive::PrimitiveKind;
use crate::runtime_builtin::reserved_tokens::TokenType;
use crate::utils::{InputSpan, Spanned};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum TransformError {
    UndeclaredVariable(String),
    UndeclaredVariableDomain(String),
    AlreadyDeclaredVariable(String),
    AlreadyDeclaredDomainVariable(Vec<(String, (i32, Spanned<VariableType>))>),
    OutOfBounds(String),
    WrongArgument {
        got: PrimitiveKind,
        expected: PrimitiveKind,
    },
    WrongExpectedArgument {
        got: PrimitiveKind,
        one_of: Vec<PrimitiveKind>,
    },
    SpannedError {
        spanned_error: Spanned<Box<TransformError>>,
        value: Option<String>,
    },
    WrongFunctionSignature {
        signature: Vec<(String, PrimitiveKind)>,
        got: Vec<PrimitiveKind>,
    },
    WrongNumberOfArguments {
        signature: Vec<(String, PrimitiveKind)>,
        args: Vec<PreExp>,
    },
    BinOpError {
        operator: BinOp,
        lhs: PrimitiveKind,
        rhs: PrimitiveKind,
    },
    UnOpError {
        operator: UnOp,
        exp: PrimitiveKind,
    },
    Unspreadable(PrimitiveKind),
    SpreadError {
        to_spread: PrimitiveKind,
        in_variables: Vec<String>,
    },
    AlreadyDefined {
        name: String,
        kind: TokenType,
    },
    Other(String),
}

#[wasm_bindgen(typescript_custom_section)]
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

impl TransformError {
    pub fn get_traced_error(&self) -> String {
        let error = self.to_string();
        let trace = self.get_trace();
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
    pub fn from_wrong_type(expected: PrimitiveKind, got: PrimitiveKind, span: InputSpan) -> Self {
        TransformError::WrongArgument { got, expected }.add_span(&span)
    }
    pub fn from_wrong_binop(
        operator: BinOp,
        lhs: PrimitiveKind,
        rhs: PrimitiveKind,
        span: InputSpan,
    ) -> Self {
        TransformError::BinOpError { operator, lhs, rhs }.add_span(&span)
    }
    pub fn from_wrong_unop(operator: UnOp, exp: PrimitiveKind, span: InputSpan) -> Self {
        TransformError::UnOpError { operator, exp }.add_span(&span)
    }
    pub fn add_span(self, span: &InputSpan) -> TransformError {
        TransformError::SpannedError {
            spanned_error: Spanned::new(Box::new(self), span.clone()),
            value: None,
        }
    }
    pub fn get_trace(&self) -> Vec<(InputSpan, Option<String>)> {
        match self {
            TransformError::SpannedError {
                spanned_error: span,
                value,
            } => {
                let mut trace = vec![(span.get_span().clone(), value.clone())];
                let mut last_error = span;
                while let TransformError::SpannedError {
                    spanned_error: ref span,
                    ref value,
                } = **last_error.get_span_value()
                {
                    let current_span = span.get_span().clone();
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
    pub fn get_origin_span(&self) -> Option<InputSpan> {
        let trace = self.get_trace();
        trace.first().map(|(span, _)| span.clone())
    }
    pub fn get_base_error(&self) -> &TransformError {
        match self {
            TransformError::SpannedError {
                spanned_error: span,
                ..
            } => span.get_base_error(),
            _ => self,
        }
    }
    pub fn get_trace_from_source(&self, source: &str) -> Result<String, String> {
        let trace = self.get_trace();
        let trace = trace
            .into_iter()
            .map(|(span, _)| {
                let text = span.get_span_text(source)?;
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
