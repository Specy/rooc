/*
_ => Err(ParseError::UnexpectedToken(format!(
    "Expected condition list but got: {}",
    condition_list.as_str()
))),

*/



#[macro_export]
macro_rules! err_unexpected_token {
    ($s:literal, $arg:ident) => {
        Err(CompilationError::from_span(
            ParseError::UnexpectedToken(format!($s, $arg.as_str())),
            &$arg.as_span(),
            false,
        ))
    };
}

#[macro_export]
macro_rules! err_missing_token {
    ($s: literal, $arg:ident) => {
        Err(CompilationError::from_span(
            ParseError::MissingToken(format!($s)),
            &$arg.as_span(),
            true,
        ))
    }
}

#[macro_export]
macro_rules! err_semantic_error {
    ($s: literal, $arg:ident) => {
        Err(CompilationError::from_span(
            ParseError::SemanticError(format!($s)),
            &$arg.as_span(),
            true,
        ))
    }
}

