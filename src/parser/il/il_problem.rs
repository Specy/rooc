use core::fmt;

use serde::ser::SerializeStruct;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::{
    math::math_enums::{Comparison, OptimizationType},
    primitives::{functions::function_traits::FunctionCall, primitive::Primitive},
    type_checker::type_checker_context::{TypeCheckable, TypeCheckerContext, WithType},
    utils::InputSpan,
};
use crate::parser::il::il_exp::PreExp;
use crate::parser::il::iterable_set::IterableSet;
use crate::parser::model_transformer::transform_error::TransformError;
use crate::parser::model_transformer::transformer_context::TransformerContext;
use crate::primitives::primitive_traits::ApplyOp;
use crate::traits::latex::ToLatex;

#[derive(Debug, Serialize, Clone)]
pub struct AddressableAccess {
    pub name: String,
    pub accesses: Vec<PreExp>,
}

#[wasm_bindgen(typescript_custom_section)]
const IAddressableAccess: &'static str = r#"
export type SerializedAddressableAccess = {
    name: string,
    accesses: SerializedPreExp[],
}
"#;

impl AddressableAccess {
    pub fn new(name: String, accesses: Vec<PreExp>) -> Self {
        Self { name, accesses }
    }
}

impl ToLatex for AddressableAccess {
    fn to_latex(&self) -> String {
        let rest = self
            .accesses
            .iter()
            .map(|a| format!("[{}]", a.to_latex()))
            .collect::<Vec<String>>()
            .join("");
        format!("{}{}", self.name, rest)
    }
}

impl fmt::Display for AddressableAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rest = self
            .accesses
            .iter()
            .map(|a| format!("[{}]", a))
            .collect::<Vec<String>>()
            .join("");
        write!(f, "{}{}", self.name, rest)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct CompoundVariable {
    pub name: String,
    pub indexes: Vec<PreExp>,
}

#[wasm_bindgen(typescript_custom_section)]
const ICompoundVariable: &'static str = r#"
export type SerializedCompoundVariable = {
    name: string,
    indexes: SerializedPreExp[],
}
"#;

impl CompoundVariable {
    pub fn new(name: String, indexes: Vec<PreExp>) -> Self {
        Self { name, indexes }
    }

    pub fn compute_indexes(
        &self,
        context: &TransformerContext,
    ) -> Result<Vec<Primitive>, TransformError> {
        self.indexes
            .iter()
            .map(|i| i.as_primitive(context))
            .collect::<Result<Vec<Primitive>, TransformError>>()
    }
}

impl ToLatex for CompoundVariable {
    fn to_latex(&self) -> String {
        let indexes = self
            .indexes
            .iter()
            .map(|i| match i {
                PreExp::Primitive(p) => {
                    if p.get_type().is_numeric() {
                        format!("{}", i.to_latex())
                    } else {
                        format!("({})", i.to_latex())
                    }
                }
                PreExp::Variable(name) => {
                    let name = name.get_span_value().clone();
                    format!("{}", name)
                }
                _ => format!("({})", i.to_latex()),
            })
            .collect::<Vec<String>>();
        format!("{}_{{{}}}", self.name, indexes.join(""))
    }
}

impl fmt::Display for CompoundVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indexes = self
            .indexes
            .iter()
            .map(|i| match i {
                PreExp::Primitive(p) => match p.get_span_value() {
                    Primitive::Number(n) => format!("{}", n),
                    _ => format!("{{{}}}", i),
                },
                PreExp::Variable(name) => name.get_span_value().clone(),
                _ => format!("{{{}}}", i),
            })
            .collect::<Vec<String>>();
        write!(f, "{}_{}", self.name, indexes.join("_"))
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct PreObjective {
    pub objective_type: OptimizationType,
    pub rhs: PreExp,
}

impl ToLatex for PreObjective {
    fn to_latex(&self) -> String {
        let rhs = self.rhs.to_latex();
        let opt_name = self.objective_type.to_latex();
        format!("{} \\ {}", opt_name, rhs)
    }
}

#[wasm_bindgen(typescript_custom_section)]
const IPreObjective: &'static str = r#"
export type SerializedPreObjective = {
    objective_type: OptimizationType,
    rhs: SerializedPreExp,
}
"#;

impl TypeCheckable for PreObjective {
    fn type_check(&self, context: &mut TypeCheckerContext) -> Result<(), TransformError> {
        self.rhs.type_check(context)
    }
    fn populate_token_type_map(&self, context: &mut TypeCheckerContext) {
        self.rhs.populate_token_type_map(context)
    }
}

impl PreObjective {
    pub fn new(objective_type: OptimizationType, rhs: PreExp) -> Self {
        Self {
            objective_type,
            rhs,
        }
    }
}

impl fmt::Display for PreObjective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.objective_type, self.rhs)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct PreCondition {
    pub lhs: PreExp,
    pub condition_type: Comparison,
    pub rhs: PreExp,
    pub iteration: Vec<IterableSet>,
    pub span: InputSpan,
}

#[wasm_bindgen(typescript_custom_section)]
const IPreCondition: &'static str = r#"
export type SerializedPreCondition = {
    lhs: SerializedPreExp,
    condition_type: Comparison,
    rhs: SerializedPreExp,
    iteration: SerializedVariableType[],
    span: InputSpan,
}
"#;

impl PreCondition {
    pub fn new(
        lhs: PreExp,
        condition_type: Comparison,
        rhs: PreExp,
        iteration: Vec<IterableSet>,
        span: InputSpan,
    ) -> Self {
        Self {
            lhs,
            condition_type,
            rhs,
            iteration,
            span,
        }
    }
}

impl TypeCheckable for PreCondition {
    fn type_check(&self, context: &mut TypeCheckerContext) -> Result<(), TransformError> {
        for iter in &self.iteration {
            iter.iterator
                .type_check(context)
                .map_err(|e| e.to_spanned_error(iter.iterator.get_span()))?;
            context.add_scope();
            let types = iter.get_variable_types(context)?;
            for (name, t) in types {
                context.add_token_type(
                    t,
                    name.get_span().clone(),
                    Some(name.get_span_value().clone()),
                )?;
            }
        }
        match (self.lhs.type_check(context), self.rhs.type_check(context)) {
            (Ok(()), Ok(())) => (),
            (Err(e), _) | (_, Err(e)) => {
                for _ in &self.iteration {
                    context.pop_scope()?;
                }
                return Err(e);
            }
        }
        let lhs_type = self.lhs.get_type(context);
        let rhs_type = self.rhs.get_type(context);
        for _ in &self.iteration {
            context.pop_scope()?;
        }
        if !lhs_type.is_numeric() || !rhs_type.is_numeric() {
            let err = TransformError::Other(format!(
                "Expected comparison of \"Number\", got \"{}\" {} \"{}\"",
                lhs_type.to_string(),
                self.condition_type,
                rhs_type.to_string()
            ))
            .to_spanned_error(&self.span);
            return Err(err);
        }
        Ok(())
    }
    fn populate_token_type_map(&self, context: &mut TypeCheckerContext) {
        for iter in &self.iteration {
            context.add_scope();
            iter.populate_token_type_map(context);
        }
        self.lhs.populate_token_type_map(context);
        self.rhs.populate_token_type_map(context);
        for _ in &self.iteration {
            let _ = context.pop_scope();
        }
    }
}

impl ToLatex for PreCondition {
    fn to_latex(&self) -> String {
        let lhs = self.lhs.to_latex();
        let rhs = self.rhs.to_latex();
        let condition = self.condition_type.to_latex();
        let iterations = self
            .iteration
            .iter()
            .map(|i| format!("\\forall{{{}}}", i.to_latex()))
            .collect::<Vec<String>>();
        if iterations.is_empty() {
            format!("{} \\ &{} \\ {}", lhs, condition, rhs)
        } else {
            format!(
                "{} \\ &{} \\ {} \\qquad {}",
                lhs,
                condition,
                rhs,
                iterations.join(",\\")
            )
        }
    }
}

impl fmt::Display for PreCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        s.push_str(&format!(
            "{} {} {}",
            self.lhs, self.condition_type, self.rhs
        ));
        if !self.iteration.is_empty() {
            s.push_str(" for ");
            s.push_str(
                &self
                    .iteration
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
            );
        }
        f.write_str(&s)
    }
}
