use std::fmt::Debug;

use dyn_clone::DynClone;
use erased_serde::serialize_trait_object;
use pest::iterators::Pair;

use crate::parser::il::il_exp::PreExp;
use crate::parser::model_transformer::transform_error::TransformError;
use crate::parser::model_transformer::transformer_context::TransformerContext;
use crate::traits::latex::{escape_latex, ToLatex};
use crate::{
    parser::parser::Rule,
    primitives::primitive::{Primitive, PrimitiveKind},
    type_checker::type_checker_context::{TypeCheckable, WithType},
    utils::InputSpan,
};

/*TODO
the PreExp should have only the function call which includes the parameters expressions and the function name
the actual function (which can be kept like this trait, but implemented as a struct, only the `call` method should be implemented)
should be saved inside the transformer context, this way there can also be user defined functions, and functions builtin
perhaps i could add instance functions to objects too
 */
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
        format!("{}({})", escape_latex(&self.get_function_name()), pars)
    }

    fn get_function_name(&self) -> String;

    fn get_parameters(&self) -> &Vec<PreExp>;
    fn get_span(&self) -> &InputSpan;
}

serialize_trait_object!(FunctionCall);
