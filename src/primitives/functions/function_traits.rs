use dyn_clone::DynClone;
use erased_serde::serialize_trait_object;
use pest::iterators::Pair;
use std::fmt::Debug;

use crate::traits::latex::{escape_latex, ToLatex};
use crate::{
    parser::{
        parser::Rule,
        pre_parsed_problem::PreExp,
        transformer::{TransformError, TransformerContext},
    },
    primitives::primitive::{Primitive, PrimitiveKind},
    type_checker::type_checker_context::{TypeCheckable, WithType},
    utils::InputSpan,
};

pub trait FunctionCall:
    Debug + DynClone + erased_serde::Serialize + WithType + TypeCheckable + Send + Sync
{
    fn from_parameters(args: Vec<PreExp>, origin_rule: &Pair<Rule>) -> Self
    where
        Self: Sized;
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError>;

    fn get_type_signature(&self) -> Vec<(String, PrimitiveKind)>;

    fn to_string(&self) -> String;
    fn to_latex(&self) -> String {
        let pars = self
            .get_parameters()
            .iter()
            .map(|p| p.to_latex())
            .collect::<Vec<String>>()
            .join(",\\");
        format!(
            "{}({})",
            escape_latex(&self.get_function_name()),
            pars
        )
    }

    fn get_function_name(&self) -> String;

    fn get_parameters(&self) -> &Vec<PreExp>;
    fn get_span(&self) -> &InputSpan;
}

serialize_trait_object!(FunctionCall);
