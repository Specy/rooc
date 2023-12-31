use core::fmt;
use std::{collections::HashMap, str::FromStr};

use crate::{
    enum_with_variants_to_string,
    math::math_enums::{Comparison, OptimizationType},
    math::operators::BinOp,
    primitives::{
        functions::function_traits::FunctionCall,
        graph::{Graph, GraphEdge, GraphNode},
        iterable::IterableKind,
        primitive::{Primitive, PrimitiveKind},
    },
    type_checker::type_checker_context::{TypeCheckable, TypeCheckerContext, WithType},
    utils::{InputSpan, Spanned},
    wrong_argument,
};
use wasm_bindgen::prelude::*;

use super::{
    recursive_set_resolver::recursive_set_resolver,
    transformer::{Exp, Frame, TransformError, TransformerContext, VariableType},
};
use crate::math::operators::UnOp;
use crate::primitives::primitive_traits::ApplyOp;
use serde::ser::{SerializeStruct, Serializer};
use serde::{de, Deserialize, Serialize};
use wasm_bindgen::prelude::*;

enum_with_variants_to_string! {
    pub enum BlockScopedFunctionKind derives[Debug, Clone] with_wasm {
        Sum,
        Prod,
        Min,
        Max,
        Avg,
    }
}
impl fmt::Display for BlockScopedFunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Sum => "sum".to_string(),
            Self::Prod => "prod".to_string(),
            Self::Min => "min".to_string(),
            Self::Max => "max".to_string(),
            Self::Avg => "avg".to_string(),
        };
        f.write_str(&s)
    }
}
impl FromStr for BlockScopedFunctionKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sum" => Ok(Self::Sum),
            "prod" => Ok(Self::Prod),
            "min" => Ok(Self::Min),
            "max" => Ok(Self::Max),
            "avg" => Ok(Self::Avg),
            _ => Err(()),
        }
    }
}

enum_with_variants_to_string! {
    pub enum BlockFunctionKind derives[Debug, Clone] with_wasm {
        Min,
        Max,
        Avg,
    }
}

impl FromStr for BlockFunctionKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "min" => Ok(Self::Min),
            "max" => Ok(Self::Max),
            "avg" => Ok(Self::Avg),
            _ => Err(()),
        }
    }
}

impl fmt::Display for BlockFunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Min => "min".to_string(),
            Self::Max => "max".to_string(),
            Self::Avg => "avg".to_string(),
        };
        f.write_str(&s)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct BlockScopedFunction {
    kind: BlockScopedFunctionKind,
    iters: Vec<IterableSet>,
    exp: Box<PreExp>,
}
#[wasm_bindgen(typescript_custom_section)]
const IBlockScopedFunction: &'static str = r#"
export type SerializedBlockScopedFunction = {
    kind: BlockScopedFunctionKind,
    iters: SerializedIterableSet[],
    exp: SerializedPreExp,
}
"#;
impl BlockScopedFunction {
    pub fn new(kind: BlockScopedFunctionKind, iters: Vec<IterableSet>, exp: Box<PreExp>) -> Self {
        Self { kind, iters, exp }
    }
    pub fn get_body_span(&self) -> InputSpan {
        self.exp.get_span().clone()
    }
}
impl fmt::Display for BlockScopedFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.kind.to_string();
        write!(
            f,
            "{}({}){{ {} }}",
            name,
            self.iters
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.exp
        )
    }
}
#[derive(Debug, Serialize, Clone)]
pub struct BlockFunction {
    kind: BlockFunctionKind,
    exps: Vec<PreExp>,
}
#[wasm_bindgen(typescript_custom_section)]
const IBlockFunction: &'static str = r#"
export type SerializedBlockFunction = {
    kind: BlockFunctionKind,
    exps: SerializedPreExp[],
}
"#;
impl BlockFunction {
    pub fn new(kind: BlockFunctionKind, exps: Vec<PreExp>) -> Self {
        Self { kind, exps }
    }
}
impl fmt::Display for BlockFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.kind.to_string();
        write!(
            f,
            "{}{{ {} }}",
            name,
            self.exps
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
#[derive(Debug)]

pub enum PreExp {
    Primitive(Spanned<Primitive>),
    Mod(InputSpan, Box<PreExp>),
    BlockFunction(Spanned<BlockFunction>),
    Variable(Spanned<String>),
    CompoundVariable(Spanned<CompoundVariable>),
    ArrayAccess(Spanned<AddressableAccess>),
    BlockScopedFunction(Spanned<BlockScopedFunction>),
    FunctionCall(InputSpan, Box<dyn FunctionCall>),
    BinaryOperation(Spanned<BinOp>, Box<PreExp>, Box<PreExp>),
    UnaryOperation(Spanned<UnOp>, Box<PreExp>),
}
#[wasm_bindgen(typescript_custom_section)]
const IPreExp: &'static str = r#"
export type SerializedFunctionCall = {
    //TODO

}
export type SerializedPreExp = {span: InputSpan} & (
    {type: "Primitive", value: SerializedPrimitive} |
    {type: "Mod", value: SerializedPreExp} |
    {type: "BlockFunction", value: SerializedBlockFunction} |
    {type: "Variable", value: string} |
    {type: "CompoundVariable", value: SerializedCompoundVariable} |
    {type: "ArrayAccess", value: SerializedAddressableAccess} |
    {type: "BlockScopedFunction", value: SerializedBlockScopedFunction} |
    {type: "FunctionCall", value: SerializedFunctionCall} |
    {type: "BinaryOperation", value: {
        op: BinOp,
        lhs: SerializedPreExp,
        rhs: SerializedPreExp,
    }} |
    {type: "UnaryOperation", value: {
        op: UnOp,
        exp: SerializedPreExp,
    }}
)
"#;

impl Clone for PreExp {
    fn clone(&self) -> Self {
        match self {
            Self::Primitive(p) => Self::Primitive(p.clone()),
            Self::Mod(span, exp) => Self::Mod(span.clone(), exp.clone()),
            Self::BlockFunction(f) => Self::BlockFunction(f.clone()),
            Self::Variable(name) => Self::Variable(name.clone()),
            Self::CompoundVariable(c) => Self::CompoundVariable(c.clone()),
            Self::ArrayAccess(array_access) => Self::ArrayAccess(array_access.clone()),
            Self::BlockScopedFunction(f) => Self::BlockScopedFunction(f.clone()),
            Self::FunctionCall(span, f) => {
                Self::FunctionCall(span.clone(), dyn_clone::clone_box(f.as_ref()))
            }
            Self::BinaryOperation(op, lhs, rhs) => {
                Self::BinaryOperation(op.clone(), lhs.clone(), rhs.clone())
            }
            Self::UnaryOperation(op, exp) => Self::UnaryOperation(op.clone(), exp.clone()),
        }
    }
}

impl Serialize for PreExp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Primitive(p) => {
                let mut state = serializer.serialize_struct("Primitive", 3)?;
                state.serialize_field("type", &"Primitive")?;
                state.serialize_field("value", &p.get_span_value())?;
                state.serialize_field("span", &p.get_span())?;
                state.end()
            }
            Self::Mod(span, exp) => {
                let mut state = serializer.serialize_struct("Mod", 3)?;
                state.serialize_field("type", &"Mod")?;
                state.serialize_field("value", &exp)?;
                state.serialize_field("span", &span)?;
                state.end()
            }
            Self::BlockFunction(f) => {
                let mut state = serializer.serialize_struct("BlockFunction", 3)?;
                state.serialize_field("type", &"BlockFunction")?;
                state.serialize_field("value", &f.get_span_value())?;
                state.serialize_field("span", &f.get_span())?;
                state.end()
            }
            Self::Variable(name) => {
                let mut state = serializer.serialize_struct("Variable", 3)?;
                state.serialize_field("type", &"Variable")?;
                state.serialize_field("value", &name.get_span_value())?;
                state.serialize_field("span", &name.get_span())?;
                state.end()
            }
            Self::CompoundVariable(c) => {
                let mut state = serializer.serialize_struct("CompoundVariable", 3)?;
                state.serialize_field("type", &"CompoundVariable")?;
                state.serialize_field("value", &c.get_span_value())?;
                state.serialize_field("span", &c.get_span())?;
                state.end()
            }
            Self::ArrayAccess(array_access) => {
                let mut state = serializer.serialize_struct("ArrayAccess", 3)?;
                state.serialize_field("type", &"ArrayAccess")?;
                state.serialize_field("value", &array_access.get_span_value())?;
                state.serialize_field("span", &array_access.get_span())?;
                state.end()
            }
            Self::BlockScopedFunction(f) => {
                let mut state = serializer.serialize_struct("BlockScopedFunction", 3)?;
                state.serialize_field("type", &"BlockScopedFunction")?;
                state.serialize_field("value", &f.get_span_value())?;
                state.serialize_field("span", &f.get_span())?;
                state.end()
            }
            Self::FunctionCall(span, f) => {
                let mut state = serializer.serialize_struct("FunctionCall", 3)?;
                state.serialize_field("type", &"FunctionCall")?;
                state.serialize_field("value", &f)?;
                state.serialize_field("span", &span)?;
                state.end()
            }
            Self::BinaryOperation(op, lhs, rhs) => {
                let mut state = serializer.serialize_struct("BinaryOperation", 3)?;
                state.serialize_field("type", &"BinaryOperation")?;
                state.serialize_field(
                    "value",
                    &TempBinOp {
                        op: **op,
                        lhs: *lhs.clone(),
                        rhs: *rhs.clone(),
                    },
                )?;
                state.serialize_field("span", &op.get_span())?;
                state.end()
            }
            Self::UnaryOperation(op, exp) => {
                let mut state = serializer.serialize_struct("UnaryOperation", 3)?;
                state.serialize_field("type", &"UnaryOperation")?;
                state.serialize_field(
                    "value",
                    &TempUnOp {
                        op: **op,
                        exp: *exp.clone(),
                    },
                )?;
                state.serialize_field("span", &op.get_span())?;
                state.end()
            }
        }
    }
}
#[derive(Serialize)]
struct TempBinOp {
    op: BinOp,
    lhs: PreExp,
    rhs: PreExp,
}

#[derive(Serialize)]
struct TempUnOp {
    op: UnOp,
    exp: PreExp,
}
impl TypeCheckable for PreExp {
    fn populate_token_type_map(&self, context: &mut TypeCheckerContext) {
        match self {
            Self::FunctionCall(span, fun) => {
                fun.populate_token_type_map(context);
                context.add_token_type(
                    fun.get_type(context),
                    span.clone(),
                    None, //Some(fun.get_function_name()) should i add this?
                )
            }
            Self::Mod(_, exp) => {
                exp.populate_token_type_map(context);
            }
            Self::Primitive(p) => {
                context.add_token_type(p.get_span_value().get_type(), p.get_span().clone(), None)
            }
            Self::Variable(name) => match context.get_value(name) {
                Some(value) => context.add_token_type(
                    value.clone(),
                    name.get_span().clone(),
                    Some(name.get_span_value().clone()),
                ),
                None => context.add_token_type(
                    PrimitiveKind::Number, //TODO we assume undeclared variables are numbers, make this configurable with assignments
                    name.get_span().clone(),
                    Some(name.get_span_value().clone()),
                ),
            },
            Self::CompoundVariable(c) => context.add_token_type(
                PrimitiveKind::Number, //every compound variable must be a number
                c.get_span().clone(),
                Some(c.to_string()),
            ),
            Self::BinaryOperation(_, lhs, rhs) => {
                lhs.populate_token_type_map(context);
                rhs.populate_token_type_map(context);
            }
            Self::UnaryOperation(_, exp) => {
                exp.populate_token_type_map(context);
            }
            Self::ArrayAccess(array_access) => context.add_token_type(
                context
                    .get_addressable_value(&array_access)
                    .unwrap_or(PrimitiveKind::Undefined),
                array_access.get_span().clone(),
                Some(array_access.to_string()),
            ),
            Self::BlockFunction(f) => {
                for exp in &f.exps {
                    exp.populate_token_type_map(context);
                }
            }
            Self::BlockScopedFunction(f) => {
                for iter in &f.iters {
                    iter.populate_token_type_map(context);
                }
                f.exp.populate_token_type_map(context);
            }
        }
    }
    //TODO improve spans
    fn type_check(&self, context: &mut TypeCheckerContext) -> Result<(), TransformError> {
        match self {
            Self::FunctionCall(span, fun) => {
                for arg in fun.get_parameters() {
                    arg.type_check(context)
                        .map_err(|e| e.to_spanned_error(span))?;
                }
                fun.type_check(context)
                    .map_err(|e| e.to_spanned_error(span))
            }
            Self::BinaryOperation(op, lhs, rhs) => {
                lhs.type_check(context)?;
                rhs.type_check(context)?;
                let lhs_type = lhs.get_type(context);
                let rhs_type = rhs.get_type(context);
                if !lhs_type.can_apply_binary_op(**op, rhs_type.clone()) {
                    Err(TransformError::from_wrong_binop(
                        **op,
                        lhs_type,
                        rhs_type,
                        op.get_span().clone(),
                    ))
                } else {
                    Ok(())
                }
            }
            Self::UnaryOperation(op, exp) => {
                exp.type_check(context)
                    .map_err(|e| e.to_spanned_error(exp.get_span()))?;
                let exp_type = exp.get_type(context);
                if !exp_type.can_apply_unary_op(**op) {
                    Err(TransformError::from_wrong_unop(
                        **op,
                        exp_type,
                        op.get_span().clone(),
                    ))
                } else {
                    Ok(())
                }
            }
            Self::Primitive(_) => Ok(()),
            Self::Mod(_, exp) => {
                exp.type_check(context)
                    .map_err(|e| e.to_spanned_error(exp.get_span()))?;
                let exp_type = exp.get_type(context);
                if !exp_type.is_numeric() {
                    return Err(TransformError::from_wrong_type(
                        exp_type,
                        PrimitiveKind::Number,
                        exp.get_span().clone(),
                    ));
                }
                Ok(())
            }
            Self::Variable(name) => Ok(()),
            Self::CompoundVariable(c) => context
                .check_compound_variable(&c.indexes)
                .map_err(|e| e.to_spanned_error(c.get_span())),
            Self::BlockFunction(f) => {
                for exp in &f.exps {
                    exp.type_check(context)
                        .map_err(|e| e.to_spanned_error(f.get_span()))?;
                    let exp_type = exp.get_type(context);
                    if !exp_type.is_numeric() {
                        return Err(TransformError::from_wrong_type(
                            PrimitiveKind::Number,
                            exp_type,
                            f.get_span().clone(),
                        ));
                    }
                }
                Ok(())
            }
            Self::BlockScopedFunction(f) => {
                for iter in &f.iters {
                    iter.iterator
                        .type_check(context)
                        .map_err(|e| e.to_spanned_error(f.get_span()))?;
                    let types = iter
                        .get_variable_types(context)
                        .map_err(|e| e.to_spanned_error(f.get_span()))?;
                    let map: HashMap<String, PrimitiveKind> = HashMap::from_iter(types);
                    let frame = Frame::from_map(map);
                    context.add_frame(frame);
                }
                let res = f.exp.type_check(context);
                let exp_type = f.exp.get_type(context);
                for _ in &f.iters {
                    context
                        .pop_scope()
                        .map_err(|e| e.to_spanned_error(f.get_span()))?;
                }
                if let Err(e) = res {
                    return Err(e.to_spanned_error(f.get_span()));
                }
                if !exp_type.is_numeric() {
                    let err = TransformError::from_wrong_type(
                        PrimitiveKind::Number,
                        exp_type,
                        f.get_span().clone(),
                    );
                    return Err(err);
                }
                Ok(())
            }
            Self::ArrayAccess(array_access) => context
                .get_addressable_value(array_access)
                .map(|_| ())
                .map_err(|e| e.to_spanned_error(array_access.get_span())),
        }
    }
}

impl WithType for PreExp {
    fn get_type(&self, context: &TypeCheckerContext) -> PrimitiveKind {
        match self {
            Self::Primitive(p) => p.get_span_value().get_type(),
            Self::FunctionCall(_, fun) => fun.get_type(context),
            Self::Variable(name) => context
                .get_value(name)
                .map(|e| e.clone())
                .unwrap_or(PrimitiveKind::Number), //TODO add assignments for
            Self::BinaryOperation(_, lhs, _) => lhs.get_type(context),
            Self::UnaryOperation(_, exp) => exp.get_type(context),
            Self::Mod(_, exp) => exp.get_type(context),
            Self::ArrayAccess(a) => context
                .get_addressable_value(a)
                .unwrap_or(PrimitiveKind::Undefined),
            Self::BlockFunction(f) => PrimitiveKind::Number, //TODO check if this is true always
            Self::BlockScopedFunction(_) => PrimitiveKind::Number, //TODO check if this is true always
            Self::CompoundVariable(_) => PrimitiveKind::Number, //TODO check if this is true always
        }
    }
}

impl PreExp {
    pub fn to_boxed(self) -> Box<PreExp> {
        Box::new(self)
    }
    pub fn get_span(&self) -> &InputSpan {
        match self {
            Self::Primitive(n) => n.get_span(),
            Self::Mod(span, _) => span,
            Self::BlockFunction(f) => f.get_span(),
            Self::Variable(name) => name.get_span(),
            Self::CompoundVariable(c) => c.get_span(),
            Self::BinaryOperation(op, _, _) => op.get_span(),
            Self::UnaryOperation(op, _) => op.get_span(),
            Self::ArrayAccess(array_access) => array_access.get_span(),
            Self::BlockScopedFunction(function) => function.get_span(),
            Self::FunctionCall(span, _) => span,
        }
    }
    pub fn into_exp(&self, context: &mut TransformerContext) -> Result<Exp, TransformError> {
        match self {
            Self::BinaryOperation(op, lhs, rhs) => {
                let lhs = lhs
                    .into_exp(context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                let rhs = rhs
                    .into_exp(context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                Ok(Exp::BinOp(**op, lhs.to_box(), rhs.to_box()))
            }
            Self::Primitive(n) => match n.as_number_cast() {
                Ok(n) => Ok(Exp::Number(n)),
                Err(e) => Err(e.to_spanned_error(self.get_span())),
            },
            Self::Mod(span, exp) => {
                let inner = exp
                    .into_exp(context)
                    .map_err(|e| e.to_spanned_error(span))?;
                Ok(Exp::Mod(inner.to_box()))
            }
            Self::BlockFunction(f) => {
                let mut parsed_exp = f
                    .exps
                    .iter()
                    .map(|exp| exp.into_exp(context))
                    .collect::<Result<Vec<Exp>, TransformError>>()
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                match f.kind {
                    BlockFunctionKind::Min => Ok(Exp::Min(parsed_exp)),
                    BlockFunctionKind::Max => Ok(Exp::Max(parsed_exp)),
                    BlockFunctionKind::Avg => {
                        let len = parsed_exp.len();
                        let mut sum = parsed_exp.pop().unwrap_or(Exp::Number(0.0));
                        for exp in parsed_exp.into_iter().rev() {
                            sum = Exp::BinOp(BinOp::Add, exp.to_box(), sum.to_box());
                        }
                        Ok(Exp::BinOp(
                            BinOp::Div,
                            sum.to_box(),
                            Exp::Number(len as f64).to_box(),
                        ))
                    }
                }
            }

            Self::UnaryOperation(op, exp) => {
                let inner = exp
                    .into_exp(context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                Ok(Exp::UnOp(**op, inner.to_box()))
            }
            Self::Variable(name) => {
                let value = context.get_value(name).map(|v| match v.as_number_cast() {
                    Ok(n) => Ok(Exp::Number(n)),
                    Err(e) => Err(e.to_spanned_error(self.get_span())),
                });
                match value {
                    Some(value) => Ok(value?),
                    None => Ok(Exp::Variable(name.get_span_value().clone())),
                }
            }
            Self::CompoundVariable(c) => {
                let parsed_indexes = context
                    .flatten_variable_name(&c.indexes)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                Ok(Exp::Variable(format!("{}_{}", c.name, parsed_indexes)))
            }
            Self::ArrayAccess(array_access) => {
                let value = context
                    .get_addressable_value(array_access)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                match value.as_number_cast() {
                    Ok(n) => Ok(Exp::Number(n)),
                    Err(e) => Err(e.to_spanned_error(self.get_span())),
                }
            }
            Self::BlockScopedFunction(f) => {
                let mut results = Vec::new();
                recursive_set_resolver(&f.iters, context, &mut results, 0, &|context| {
                    let inner = f
                        .exp
                        .into_exp(context)
                        .map_err(|e| e.to_spanned_error(self.get_span()))?;
                    Ok(inner)
                })
                .map_err(|e| e.to_spanned_error(self.get_span()))?;
                match f.kind {
                    BlockScopedFunctionKind::Sum => {
                        let mut sum = results.pop().unwrap_or(Exp::Number(0.0));
                        for result in results.into_iter().rev() {
                            sum = Exp::BinOp(BinOp::Add, result.to_box(), sum.to_box());
                        }
                        Ok(sum)
                    }
                    BlockScopedFunctionKind::Prod => {
                        let mut prod = results.pop().unwrap_or(Exp::Number(1.0));
                        for result in results.into_iter().rev() {
                            prod = Exp::BinOp(BinOp::Mul, result.to_box(), prod.to_box());
                        }
                        Ok(prod)
                    }
                    BlockScopedFunctionKind::Min => Ok(Exp::Min(results)),
                    BlockScopedFunctionKind::Max => Ok(Exp::Max(results)),
                    BlockScopedFunctionKind::Avg => {
                        let len = results.len();
                        let mut sum = results.pop().unwrap_or(Exp::Number(0.0));
                        for result in results.into_iter().rev() {
                            sum = Exp::BinOp(BinOp::Add, result.to_box(), sum.to_box());
                        }
                        Ok(Exp::BinOp(
                            BinOp::Div,
                            sum.to_box(),
                            Exp::Number(len as f64).to_box(),
                        ))
                    }
                }
            }
            Self::FunctionCall(span, function_call) => {
                //TODO improve this, what other types of functions can there be?
                let value = function_call
                    .call(context)
                    .map_err(|e| e.to_spanned_error(span))?;
                match value.as_number_cast() {
                    Ok(n) => Ok(Exp::Number(n)),
                    Err(e) => Err(e.to_spanned_error(self.get_span())),
                }
            }
        }
    }

    pub fn as_primitive(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        match self {
            PreExp::Primitive(p) => Ok(p.get_span_value().clone()),
            PreExp::Variable(s) => match context.get_value(s) {
                Some(value) => Ok(value.clone()),
                None => Err(TransformError::MissingVariable(s.get_span_value().clone())),
            },
            PreExp::CompoundVariable(c) => {
                let name = context.flatten_compound_variable(&c.name, &c.indexes)?;
                match context.get_value(&name) {
                    Some(value) => Ok(value.clone()),
                    None => Err(TransformError::MissingVariable(name)),
                }
            }
            PreExp::FunctionCall(_, f) => {
                let value = f
                    .call(context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                Ok(value)
            }
            PreExp::ArrayAccess(a) => {
                let value = context.get_addressable_value(a)?;
                Ok(value)
            }
            PreExp::UnaryOperation(op, v) => {
                let value = v.as_primitive(context)?;
                match value.apply_unary_op(**op) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(TransformError::from_wrong_unop(
                        **op,
                        value.get_type(),
                        op.get_span().clone(),
                    )),
                }
            }
            PreExp::BinaryOperation(op, lhs, rhs) => {
                let lhs = lhs.as_primitive(context)?;
                let rhs = rhs.as_primitive(context)?;
                match lhs.apply_binary_op(**op, &rhs) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(TransformError::from_wrong_binop(
                        **op,
                        lhs.get_type(),
                        rhs.get_type(),
                        op.get_span().clone(),
                    )),
                }
            }
            PreExp::Mod(_, _) | PreExp::BlockFunction(_) | PreExp::BlockScopedFunction(_) => {
                //TODO is this correct?
                Err(TransformError::WrongArgument {
                    got: PrimitiveKind::Undefined,
                    expected: PrimitiveKind::Any,
                })
            }
        }
    }
    //TODO make this a macro
    pub fn as_number(&self, context: &TransformerContext) -> Result<f64, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_number())
            .map_err(|e| e.to_spanned_error(self.get_span()))?
    }
    pub fn as_number_cast(&self, context: &TransformerContext) -> Result<f64, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_number_cast())
            .map_err(|e| e.to_spanned_error(self.get_span()))?
    }
    pub fn as_integer(&self, context: &TransformerContext) -> Result<i64, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_integer())
            .map_err(|e| e.to_spanned_error(self.get_span()))?
    }
    pub fn as_integer_cast(&self, context: &TransformerContext) -> Result<i64, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_integer_cast())
            .map_err(|e| e.to_spanned_error(self.get_span()))?
    }
    pub fn as_positive_integer(&self, context: &TransformerContext) -> Result<u64, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_positive_integer())
            .map_err(|e| e.to_spanned_error(self.get_span()))?
    }
    pub fn as_usize(&self, context: &TransformerContext) -> Result<usize, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_usize().map(|v| v.to_owned()))
            .map_err(|e| e.to_spanned_error(self.get_span()))?
    }
    pub fn as_usize_cast(&self, context: &TransformerContext) -> Result<usize, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_usize_cast().map(|v| v.to_owned()))
            .map_err(|e| e.to_spanned_error(self.get_span()))?
    }
    pub fn as_string(&self, context: &TransformerContext) -> Result<String, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_string().map(|v| v.to_owned()))
            .map_err(|e| e.to_spanned_error(self.get_span()))?
    }
    pub fn as_boolean(&self, context: &TransformerContext) -> Result<bool, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_boolean())
            .map_err(|e| e.to_spanned_error(self.get_span()))?
    }
    pub fn as_graph(&self, context: &TransformerContext) -> Result<Graph, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_graph().map(|v| v.to_owned()))
            .map_err(|e| e.to_spanned_error(self.get_span()))?
    }
    pub fn as_node(&self, context: &TransformerContext) -> Result<GraphNode, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_graph_node().map(|v| v.to_owned()))
            .map_err(|e| e.to_spanned_error(self.get_span()))?
    }
    pub fn as_edge(&self, context: &TransformerContext) -> Result<GraphEdge, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_graph_edge().map(|v| v.to_owned()))
            .map_err(|e| e.to_spanned_error(self.get_span()))?
    }

    pub fn as_iterator(
        &self,
        context: &TransformerContext,
    ) -> Result<IterableKind, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_iterator().map(|v| v.to_owned()))
            .map_err(|e| e.to_spanned_error(self.get_span()))?
    }

    fn is_leaf(&self) -> bool {
        match self {
            Self::BinaryOperation(_, _, _) => false,
            Self::UnaryOperation(_, _) => false,
            _ => true,
        }
    }
    fn to_string_with_precedence(&self, previous_precedence: u8) -> String {
        match self {
            Self::BinaryOperation(op, lhs, rhs) => {
                let lhs = lhs.to_string_with_precedence(op.precedence());
                let rhs = rhs.to_string_with_precedence(op.precedence());
                if op.precedence() < previous_precedence {
                    format!("({} {} {})", lhs, op.to_string(), rhs)
                } else {
                    format!("{} {} {}", lhs, op.to_string(), rhs)
                }
            }
            _ => self.to_string(),
        }
    }
}
impl fmt::Display for PreExp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::ArrayAccess(a) => a.to_string(),
            Self::BlockFunction(f) => f.to_string(),
            Self::BlockScopedFunction(f) => f.to_string(),
            Self::BinaryOperation(op, lhs, rhs) => {
                let rhs = rhs.to_string_with_precedence(op.precedence());
                let lhs = lhs.to_string_with_precedence(op.precedence());
                format!("{} {} {}", lhs, op.to_string(), rhs)
            }
            Self::CompoundVariable(c) => c.to_string(),
            Self::FunctionCall(_, f) => f.to_string(),
            Self::Mod(_, exp) => format!("|{}|", **exp),
            Self::Primitive(p) => p.to_string(),
            Self::UnaryOperation(op, exp) => {
                if self.is_leaf() {
                    format!("{}{}", **op, **exp)
                } else {
                    format!("{}({})", **op, **exp)
                }
            }
            Self::Variable(name) => name.to_string(),
        };
        f.write_str(&s)
    }
}

impl PreExp {
    pub fn get_span_wasm(&self) -> InputSpan {
        self.get_span().clone()
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct IterableSet {
    pub var: VariableType,
    pub iterator: Spanned<PreExp>,
    pub span: InputSpan,
}
#[wasm_bindgen(typescript_custom_section)]
const IIterableSet: &'static str = r#"
export type SerializedIterableSet = {
    var: SerializedVariableType,
    iterator: SerializedSpanned<SerializedPreExp>,
    span: InputSpan,
}
"#;

impl IterableSet {
    pub fn new(var: VariableType, iterator: Spanned<PreExp>, span: InputSpan) -> Self {
        Self {
            var,
            iterator,
            span,
        }
    }
    pub fn populate_token_type_map(&self, context: &mut TypeCheckerContext) {
        let iter_type = self.iterator.get_type(context);

        let iter_type = match iter_type {
            PrimitiveKind::Iterable(kind) => *kind,
            _ => PrimitiveKind::Undefined, //should this be undefined or any?
        };
        match &self.var {
            VariableType::Single(name) => context.add_token_type(
                iter_type,
                self.span.clone(),
                Some(name.get_span_value().clone()),
            ),
            VariableType::Tuple(vars) => match &iter_type {
                PrimitiveKind::Tuple(types) => {
                    for (i, v) in vars.iter().enumerate() {
                        context.add_token_type(
                            types.get(i).unwrap_or(&PrimitiveKind::Undefined).clone(),
                            self.span.clone(),
                            Some(v.get_span_value().clone()),
                        )
                    }
                }
                PrimitiveKind::Iterable(kind) => {
                    for v in vars {
                        context.add_token_type(
                            *kind.clone(),
                            self.span.clone(),
                            Some(v.get_span_value().clone()),
                        )
                    }
                }
                _ => {
                    for v in vars {
                        context.add_token_type(
                            PrimitiveKind::Undefined,
                            self.span.clone(),
                            Some(v.get_span_value().clone()),
                        )
                    }
                }
            },
        }
    }
    pub fn get_variable_types(
        &self,
        context: &TypeCheckerContext,
    ) -> Result<Vec<(String, PrimitiveKind)>, TransformError> {
        let iter_type = self.iterator.get_type(context);

        let iter_type = match iter_type {
            PrimitiveKind::Iterable(kind) => *kind,
            _ => {
                return Err(TransformError::from_wrong_type(
                    PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)),
                    iter_type,
                    self.span.clone(),
                ))
            }
        };
        match &self.var {
            VariableType::Single(name) => Ok(vec![(name.get_span_value().clone(), iter_type)]),
            VariableType::Tuple(vars) => {
                match &iter_type {
                    PrimitiveKind::Iterable(kind) => {
                        Ok(vars
                            .iter()
                            .map(|v| (v.get_span_value().clone(), *kind.clone()))
                            .collect::<Vec<_>>()) //we don't know at compile time how many variables there are, so we assume all of them exist
                    }
                    _ => {
                        let spreads_into = iter_type.can_spread_into()?;
                        if vars.len() > spreads_into.len() {
                            let err = TransformError::SpreadError {
                                to_spread: iter_type,
                                in_variables: vars
                                    .iter()
                                    .map(|v| v.get_span_value().clone())
                                    .collect::<Vec<_>>(),
                            }
                            .to_spanned_error(&self.span);
                            Err(err)
                        } else {
                            Ok(vars
                                .iter()
                                .zip(spreads_into.iter())
                                .map(|(v, t)| (v.get_span_value().clone(), t.clone()))
                                .collect::<Vec<_>>())
                        }
                    }
                }
            }
        }
    }
}
impl fmt::Display for IterableSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} in {}",
            self.var.to_string(),
            self.iterator.to_string()
        )
    }
}

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
    pub indexes: Vec<String>,
}
#[wasm_bindgen(typescript_custom_section)]
const ICompoundVariable: &'static str = r#"
export type SerializedCompoundVariable = {
    name: string,
    indexes: string[],
}
"#;
impl CompoundVariable {
    pub fn new(name: String, indexes: Vec<String>) -> Self {
        Self { name, indexes }
    }
}
impl fmt::Display for CompoundVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.name, self.indexes.join("_"))
    }
}
#[derive(Debug, Serialize, Clone)]
pub struct PreObjective {
    pub objective_type: OptimizationType,
    pub rhs: PreExp,
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
            let types = iter.get_variable_types(context)?;
            let map: HashMap<String, PrimitiveKind> = HashMap::from_iter(types);
            let frame = Frame::from_map(map);
            context.add_frame(frame);
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
            context.add_frame(Frame::default());
            iter.populate_token_type_map(context);
        }
        self.lhs.populate_token_type_map(context);
        self.rhs.populate_token_type_map(context);
        for _ in &self.iteration {
            let _ = context.pop_scope();
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
