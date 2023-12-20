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
            "Expected argument of type \"{}\", got \"{}\" evaluating \"{}\"",
            $expected_type,
            $current_arg.get_type().to_string(),
            $evauluated_in.to_string()
        ))
    };
    ($expected_type: literal, $evauluated_in:expr) => {
        TransformError::WrongArgument(format!(
            "Expected argument of type {}, got \"{}\"",
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
        ).to_spanned_error($evauluated_in.get_span()))
    };
    ($expected_type: literal, $evauluated_in:expr) => {
        Err(wrong_argument!($expected_type, $evauluated_in).to_spanned_error($evauluated_in.get_span()))
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
            _ => Err(wrong_argument!($expected, $value, $self).to_spanned_error($self.get_span())),
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

#[macro_export]
macro_rules! bail_wrong_number_of_arguments {
    ($n:expr, $expected:ident, [$($arg:literal),+]) => {
        Err(CompilationError::from_pair(
            ParseError::WrongNumberOfArguments($n, vec![$($arg.to_string()),+]),
            &$expected,
            true,
        ))
    };
}

#[macro_export]
macro_rules! check_bounds {
    ($i:expr, $v:expr, $self:expr, $mapper:expr) => {
        if $i < $v.len() {
            $mapper
        } else {
            return Err(TransformError::OutOfBounds(format!(
                "cannot access index {} of {}",
                $i,
                $self.to_string()
            )));
        }
    };
}

#[macro_export]
macro_rules! enum_with_variants_to_string {
    ($vis:vis enum $name:ident derives[$($derive:tt)+] { $($variant:ident),* $(,)? }) => {
        #[derive($($derive)*)]
        $vis enum $name {
            $($variant),*
        }

        impl $name {
            pub fn kinds() -> Vec<Self> {
                vec![$(Self::$variant),*]
            }

            pub fn kinds_to_string() -> Vec<String> {
                Self::kinds().iter().map(|k| k.to_string()).collect()
            }
        }
    };
}