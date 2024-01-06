use dyn_clone::DynClone;
use std::fmt::Debug;

use crate::{
    parser::{
        parser::Rule,
        pre_parsed_problem::PreExp,
        transformer::{TransformError, TransformerContext},
    },
    primitives::primitive::{Primitive, PrimitiveKind},
    utils::{CompilationError, InputSpan},    wrong_argument, type_checker::type_checker_context::{TypeCheckable, WithType},
};
use erased_serde::serialize_trait_object;
use pest::iterators::Pair;
pub trait FunctionCall: Debug + DynClone + erased_serde::Serialize + WithType + TypeCheckable {
    fn from_parameters(
        args: Vec<PreExp>,
        origin_rule: &Pair<Rule>,
    ) -> Self
    where
        Self: Sized;
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError>;

    fn get_type_signature(&self) -> Vec<PrimitiveKind>;

    fn to_string(&self) -> String;

    fn get_function_name(&self) -> String;

    fn get_parameters(&self) -> &Vec<PreExp>;
    fn get_span(&self) -> &InputSpan;
}

serialize_trait_object!(FunctionCall);
