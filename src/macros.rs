#[macro_export]
macro_rules! err_unexpected_token {
    ($s:literal, $arg:ident) => {
        Err(CompilationError::from_pair(
            ParseError::UnexpectedToken(format!($s, $arg.as_str())),
            &$arg,
            false,
        ))
    };
}
#[macro_export]
macro_rules! wrong_argument {
    ($expected_type: literal, $current_arg:expr, $evauluated_in:expr) => {
        TransformError::WrongArgument(format!(
            "Expected {}, got {:?} evaluating {:?}",
            $expected_type,
            $current_arg,
            $evauluated_in.to_string()
        ))
    };
    ($expected_type: literal, $evauluated_in:expr) => {
        TransformError::WrongArgument(format!(
            "Expected {}, got {:?}",
            $expected_type,
            $evauluated_in.to_string()
        ))
    };
}

#[macro_export]
macro_rules! bail_wrong_argument {
    ($expected_type: literal, $current_arg:expr, $evauluated_in:expr) => {
        Err(wrong_argument!(
            $expected_type,
            $current_arg,
            $evauluated_in
        ))
    };
    ($expected_type: literal, $evauluated_in:expr) => {
        Err(wrong_argument!($expected_type, $evauluated_in))
    };
}
#[macro_export]
macro_rules! bail_wrong_argument_spanned {
    ($expected_type: literal, $current_arg:expr, $evauluated_in:expr) => {
        Err(wrong_argument!(
            $expected_type,
            $current_arg,
            $evauluated_in
        ).to_spanned_error($evauluated_in.as_span()))
    };
    ($expected_type: literal, $evauluated_in:expr) => {
        Err(wrong_argument!($expected_type, $evauluated_in).to_spanned_error($evauluated_in.as_span()))
    };
}

#[macro_export]
macro_rules! match_or_bail {
    ($expected:expr, $($enum:ident:: $variant:ident($($var:pat),+) => $expr:expr),+ ; ($value:expr, $self:expr)) => {
        match $value {
            $(
                $enum::$variant($($var),+) => $expr,
            )+
            _ => bail_wrong_argument!($expected, $value, $self),
        }
    };
}
#[macro_export]
macro_rules! match_or_bail_spanned {
    ($expected:expr, $($enum:ident:: $variant:ident($($var:pat),+) => $expr:expr),+ ; ($value:expr, $self:expr)) => {
        match $value {
            $(
                $enum::$variant($($var),+) => $expr,
            )+
            _ => Err(wrong_argument!($expected, $value, $self).to_spanned_error($self.as_span())),
        }
    };
}

#[macro_export]
macro_rules! bail_missing_token {
    ($s: literal, $arg:ident) => {
        Err(CompilationError::from_pair(
            ParseError::MissingToken(format!($s)),
            &$arg,
            true,
        ))
    };
}

#[macro_export]
macro_rules! bail_semantic_error {
    ($s: literal, $arg:ident) => {
        Err(CompilationError::from_pair(
            ParseError::SemanticError(format!($s)),
            &$arg,
            true,
        ))
    };
}
