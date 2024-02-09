#[macro_export]
macro_rules! err_unexpected_token {
    ($s:literal, $arg:ident $(, $x:expr )*) => {
        Err(CompilationError::from_pair(
            ParseError::UnexpectedToken(format!($s, $arg.as_str(), $($x),*)),
            &$arg,
            false,
        ))
    };
}
#[macro_export]
macro_rules! wrong_argument {
    ($expected_type: expr, $current_arg:expr) => {
        TransformError::WrongArgument {
            expected: $expected_type,
            got: $current_arg.get_type(),
        }
    };
}

#[macro_export]
macro_rules! bail_incorrect_type_signature_of_fn {
    ($self:expr, $context:expr) => {
        bail_incorrect_type_signature!(
            $self.get_type_signature(),
            $self
                .get_parameters()
                .iter()
                .map(|a| a.get_type($context))
                .collect::<Vec<_>>(),
            $self.get_span()
        )
    };
}

#[macro_export]
macro_rules! bail_incorrect_type_signature {
    ($expected:expr, $current:expr, $span:expr) => {
        Err((TransformError::WrongFunctionSignature {
            signature: $expected,
            got: $current,
        })
        .add_span($span))
    };
    ($expected:expr, $current:expr) => {
        Err(TransformError::WrongFunctionSignature {
            signature: $expected,
            got: $current,
        })
    };
}

#[macro_export]
macro_rules! bail_wrong_number_of_arguments {
    ($function:expr) => {
        Err(TransformError::WrongNumberOfArguments {
            signature: $function.get_type_signature(),
            args: $function.get_parameters().clone(),
        })
    };
}

#[macro_export]
macro_rules! bail_wrong_argument {
    ($expected_type: expr, $current_arg:expr) => {
        Err(wrong_argument!($expected_type, $current_arg))
    };
}

#[macro_export]
macro_rules! match_or_bail {
    ($expected:expr, $($enum:ident:: $variant:ident($($var:pat),+) => $expr:expr),+ ; ($value:expr)) => {
        match $value {
            $(
                $enum::$variant($($var),+) => $expr,
            )+
            _ => bail_wrong_argument!($expected, $value),
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
    ($vis:vis enum $name:ident derives[$($derive:tt)+] with_wasm { $($variant:ident),* $(,)? }) => {
        #[derive($($derive)*, Serialize)]
        #[serde(tag = "type")]
        #[wasm_bindgen]
        $vis enum $name {
            $($variant),*
        }
        impl $name {
            pub fn kinds() -> Vec<$name> {
                vec![$(Self::$variant),*]
            }

            pub fn kinds_to_string() -> Vec<String> {
                Self::kinds().iter().map(|k| k.to_string()).collect()
            }
        }

    };
}
