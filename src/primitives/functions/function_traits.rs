use dyn_clone::DynClone;
use std::fmt::Debug;

use crate::{
    parser::{
        parser::Rule,
        pre_parsed_problem::PreExp,
        transformer::{TransformError, TransformerContext, TypeCheckerContext},
    },
    primitives::primitive::{Primitive, PrimitiveKind},
    utils::CompilationError,
};
use erased_serde::serialize_trait_object;
use pest::iterators::Pair;
pub trait FunctionCall: Debug + DynClone + erased_serde::Serialize {
    fn from_parameters(
        pars: Vec<PreExp>,
        origin_rule: &Pair<Rule>,
    ) -> Result<Self, CompilationError>
    where
        Self: Sized;
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError>;

    fn get_parameters_types(&self) -> Vec<PrimitiveKind>;
    fn get_return_type(&self, context: &TypeCheckerContext) -> PrimitiveKind;
    fn type_check(&self, context: &TypeCheckerContext) -> Result<(), TransformError>;

    fn to_string(&self) -> String;
}

serialize_trait_object!(FunctionCall);
