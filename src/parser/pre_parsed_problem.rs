use super::{
    recursive_set_resolver::recursive_set_resolver,
    transformer::{Exp, TransformError, TransformerContext, VariableType},
};
use crate::math::operators::ApplyOp;
use crate::{
    bail_wrong_argument_spanned, enum_with_variants_to_string, match_or_bail_spanned,
    math::math_enums::{Comparison, OptimizationType},
    math::operators::Op,
    primitives::{
        functions::function_traits::FunctionCall,
        graph::{Graph, GraphEdge, GraphNode},
        iterable::IterableKind,
        primitive::Primitive,
    },
    utils::{InputSpan, Spanned},
    wrong_argument,
};

enum_with_variants_to_string! {
    pub enum BlockScopedFunctionKind derives[Debug] {
        Sum,
        Prod,
        Min,
        Max,
        Avg,
    }
}
impl BlockScopedFunctionKind {
    pub fn to_string(&self) -> String {
        match self {
            Self::Sum => "sum".to_string(),
            Self::Prod => "prod".to_string(),
            Self::Min => "min".to_string(),
            Self::Max => "max".to_string(),
            Self::Avg => "avg".to_string(),
        }
    }
}
enum_with_variants_to_string! {
    pub enum BlockFunctionKind derives[Debug] {
        Min,
        Max,
        Avg,
    }
}

impl BlockFunctionKind {
    pub fn to_string(&self) -> String {
        match self {
            Self::Min => "min".to_string(),
            Self::Max => "max".to_string(),
            Self::Avg => "avg".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct BlockScopedFunction {
    kind: BlockScopedFunctionKind,
    iters: Vec<IterableSet>,
    exp: Box<PreExp>,
}
impl BlockScopedFunction {
    pub fn new(kind: BlockScopedFunctionKind, iters: Vec<IterableSet>, exp: Box<PreExp>) -> Self {
        Self { kind, iters, exp }
    }
    pub fn get_body_span(&self) -> InputSpan {
        self.exp.get_span().clone()
    }
    pub fn to_string(&self) -> String {
        let name = self.kind.to_string();
        format!(
            "{}({}){{{}}}",
            name,
            self.iters
                .iter()
                .map(|i| i.iterator.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.exp.to_string()
        )
    }
}
#[derive(Debug)]
pub struct BlockFunction {
    kind: BlockFunctionKind,
    exps: Vec<PreExp>,
}
impl BlockFunction {
    pub fn new(kind: BlockFunctionKind, exps: Vec<PreExp>) -> Self {
        Self { kind, exps }
    }
    pub fn to_string(&self) -> String {
        let name = self.kind.to_string();
        format!(
            "{}{{{}}}",
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
    Mod(Spanned<Box<PreExp>>),
    BlockFunction(Spanned<BlockFunction>),
    Variable(Spanned<String>),
    CompoundVariable(Spanned<CompoundVariable>),
    ArrayAccess(Spanned<ArrayAccess>),
    BlockScopedFunction(Spanned<BlockScopedFunction>),
    FunctionCall(Spanned<Box<dyn FunctionCall>>),

    BinaryOperation(Spanned<Op>, Box<PreExp>, Box<PreExp>),
    UnaryOperation(Spanned<Op>, Spanned<Box<PreExp>>),
}

impl PreExp {
    pub fn to_boxed(self) -> Box<PreExp> {
        Box::new(self)
    }
    pub fn get_span(&self) -> &InputSpan {
        match self {
            Self::Primitive(n) => n.get_span(),
            Self::Mod(exp) => exp.get_span(),
            Self::BlockFunction(f) => f.get_span(),
            Self::Variable(name) => name.get_span(),
            Self::CompoundVariable(c) => c.get_span(),
            Self::BinaryOperation(op, _, _) => op.get_span(),
            Self::UnaryOperation(op, _) => op.get_span(),
            Self::ArrayAccess(array_access) => array_access.get_span(),
            Self::BlockScopedFunction(function) => function.get_span(),
            Self::FunctionCall(function_call) => function_call.get_span(),
        }
    }
    pub fn into_exp(&self, context: &mut TransformerContext) -> Result<Exp, TransformError> {
        let exp = match self {
            Self::BinaryOperation(op, lhs, rhs) => {
                let lhs = lhs
                    .into_exp(context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                let rhs = rhs
                    .into_exp(context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                match (op, lhs, rhs) {
                    (op, lhs, rhs) => Ok(Exp::BinOp(**op, lhs.to_box(), rhs.to_box())),
                }
            }
            Self::Primitive(n) => match **n {
                Primitive::Number(n) => Ok(Exp::Number(n)),
                _ => {
                    let err = TransformError::WrongArgument(format!(
                        "Expected \"Number\", got \"{}\"",
                        n.get_type_string()
                    ));
                    Err(err.to_spanned_error(self.get_span()))
                }
            },
            Self::Mod(exp) => {
                let inner = exp
                    .into_exp(context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
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
                            sum = Exp::BinOp(Op::Add, exp.to_box(), sum.to_box());
                        }
                        Ok(Exp::BinOp(
                            Op::Div,
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
                let value = context.get_value(&*name).map(|v| match v {
                    Primitive::Number(n) => Ok(Exp::Number(n.clone())),
                    _ => {
                        let err = TransformError::WrongArgument(format!(
                            "Expected \"Number\", got \"{}\"",
                            v.get_type_string()
                        ));
                        Err(err.to_spanned_error(self.get_span()))
                    }
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
                    .get_array_value(array_access)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                match value {
                    Primitive::Number(n) => Ok(Exp::Number(n.clone())),
                    _ => {
                        let err = TransformError::WrongArgument(format!(
                            "Expected \"Number\", got \"{}\"",
                            value.get_type_string()
                        ));
                        Err(err.to_spanned_error(self.get_span()))
                    }
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
                            sum = Exp::BinOp(Op::Add, result.to_box(), sum.to_box());
                        }
                        Ok(sum)
                    }
                    BlockScopedFunctionKind::Prod => {
                        let mut prod = results.pop().unwrap_or(Exp::Number(1.0));
                        for result in results.into_iter().rev() {
                            prod = Exp::BinOp(Op::Mul, result.to_box(), prod.to_box());
                        }
                        Ok(prod)
                    }
                    BlockScopedFunctionKind::Min => Ok(Exp::Min(results)),
                    BlockScopedFunctionKind::Max => Ok(Exp::Max(results)),
                    BlockScopedFunctionKind::Avg => {
                        let len = results.len();
                        let mut sum = results.pop().unwrap_or(Exp::Number(0.0));
                        for result in results.into_iter().rev() {
                            sum = Exp::BinOp(Op::Add, result.to_box(), sum.to_box());
                        }
                        Ok(Exp::BinOp(
                            Op::Div,
                            sum.to_box(),
                            Exp::Number(len as f64).to_box(),
                        ))
                    }
                }
            }
            Self::FunctionCall(function_call) => {
                //TODO improve this, what other types of functions can there be?
                let value = function_call
                    .call(&context)
                    .map_err(|e| e.to_spanned_error(self.get_span()))?;
                match value {
                    Primitive::Number(n) => Ok(Exp::Number(n)),
                    _ => {
                        let err = TransformError::WrongArgument(format!(
                            "Expected \"Number\", got \"{}\"",
                            value.get_type_string()
                        ));
                        Err(err.to_spanned_error(self.get_span()))
                    }
                }
            }
        };
        exp.map(|e: Exp| e.flatten())
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
            PreExp::FunctionCall(f) => {
                let value = f.call(context)?;
                Ok(value)
            }
            PreExp::ArrayAccess(a) => {
                let value = context.get_array_value(a)?;
                Ok(value.to_owned())
            }
            PreExp::UnaryOperation(op, v) => {
                let value = v.as_primitive(context)?;
                match value.apply_unary_op(**op) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        let err = TransformError::OperatorError(e.to_string());
                        Err(err.to_spanned_error(self.get_span()))
                    }
                }
            }
            PreExp::BinaryOperation(op, lhs, rhs) => {
                let lhs = lhs.as_primitive(context)?;
                let rhs = rhs.as_primitive(context)?;
                match lhs.apply_binary_op(**op, &rhs) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        let err = TransformError::OperatorError(e.to_string());
                        Err(err.to_spanned_error(self.get_span()))
                    }
                }
            }
            _ => Err(TransformError::WrongArgument(format!(
                "Expected \"Primitive\"" //TODO add the value of the primitive
            ))),
        }
    }
    //TODO make this a macro
    pub fn as_number(&self, context: &TransformerContext) -> Result<f64, TransformError> {
        let value = self
            .as_primitive(context)
            .map_err(|e| e.to_spanned_error(self.get_span()))?;
        match_or_bail_spanned!("Number", Primitive::Number(n) => Ok(n) ; (value, self))
    }
    pub fn as_string(&self, context: &TransformerContext) -> Result<String, TransformError> {
        let value = self
            .as_primitive(context)
            .map_err(|e| e.to_spanned_error(self.get_span()))?;
        match_or_bail_spanned!("String", Primitive::String(s) => Ok(s.to_owned()) ; (value, self))
    }
    pub fn as_integer(&self, context: &TransformerContext) -> Result<i64, TransformError> {
        let n = self
            .as_number(context)
            .map_err(|e| e.to_spanned_error(self.get_span()))?;
        if n.fract() != 0.0 {
            bail_wrong_argument_spanned!("Integer", self)
        } else {
            Ok(n as i64)
        }
    }
    pub fn as_usize(&self, context: &TransformerContext) -> Result<usize, TransformError> {
        let n = self
            .as_number(context)
            .map_err(|e| e.to_spanned_error(self.get_span()))?;
        if n.fract() != 0.0 {
            bail_wrong_argument_spanned!("Integer", self)
        } else if n < 0.0 {
            bail_wrong_argument_spanned!("Positive integer", self)
        } else {
            Ok(n as usize)
        }
    }
    pub fn as_boolean(&self, context: &TransformerContext) -> Result<bool, TransformError> {
        let value = self
            .as_primitive(context)
            .map_err(|e| e.to_spanned_error(self.get_span()))?;
        match_or_bail_spanned!("Boolean", Primitive::Boolean(b) => Ok(b) ; (value, self))
    }
    pub fn as_graph(&self, context: &TransformerContext) -> Result<Graph, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_graph().map(|v| v.to_owned()))
            .map_err(|e| e.to_spanned_error(self.get_span()))?
    }
    pub fn as_node(&self, context: &TransformerContext) -> Result<GraphNode, TransformError> {
        let node = self
            .as_primitive(context)
            .map_err(|e| e.to_spanned_error(self.get_span()))?;
        match_or_bail_spanned!("GraphNode", Primitive::GraphNode(n) => Ok(n.to_owned()) ; (node, self))
    }
    pub fn as_edge(&self, context: &TransformerContext) -> Result<GraphEdge, TransformError> {
        let edge = self
            .as_primitive(context)
            .map_err(|e| e.to_spanned_error(self.get_span()))?;
        match_or_bail_spanned!("GraphEdge", Primitive::GraphEdge(e) => Ok(e.to_owned()) ; (edge, self))
    }

    pub fn as_iterator(
        &self,
        context: &TransformerContext,
    ) -> Result<IterableKind, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_iterator().map(|v| v.to_owned()))
            .map_err(|e| e.to_spanned_error(self.get_span()))?
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::ArrayAccess(a) => a.to_string(),
            Self::BlockFunction(f) => f.to_string(),
            Self::BlockScopedFunction(f) => f.to_string(),
            Self::BinaryOperation(op, lhs, rhs) => format!(
                "({} {} {})",
                lhs.to_string(),
                op.to_string(),
                rhs.to_string()
            ),
            Self::CompoundVariable(c) => c.to_string(),
            Self::FunctionCall(f) => f.to_string(),
            Self::Mod(exp) => format!("|{}|", exp.to_string()),
            Self::Primitive(p) => p.to_string(),
            Self::UnaryOperation(op, exp) => format!("{}{}",op.to_string(), exp.to_string()),
            Self::Variable(name) => name.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct IterableSet {
    pub var: VariableType,
    pub iterator: Spanned<PreExp>,
    pub span: InputSpan,
}
impl IterableSet {
    pub fn new(var: VariableType, iterator: Spanned<PreExp>, span: InputSpan) -> Self {
        Self {
            var,
            iterator,
            span,
        }
    }
}

#[derive(Debug)]
pub struct ArrayAccess {
    pub name: String,
    pub accesses: Vec<PreExp>,
}
impl ArrayAccess {
    pub fn new(name: String, accesses: Vec<PreExp>) -> Self {
        Self { name, accesses }
    }
    pub fn to_string(&self) -> String {
        let rest = self
            .accesses
            .iter()
            .map(|a| format!("[{}]", a.to_string()))
            .collect::<Vec<String>>()
            .join("");
        format!("{}{}", self.name, rest)
    }
}

#[derive(Debug)]
pub struct CompoundVariable {
    pub name: String,
    pub indexes: Vec<String>,
}
impl CompoundVariable {
    pub fn new(name: String, indexes: Vec<String>) -> Self {
        Self { name, indexes }
    }
    pub fn to_string(&self) -> String {
        format!("{}_{}", self.name, self.indexes.join("_"))
    }
}

#[derive(Debug)]
pub struct PreObjective {
    pub objective_type: OptimizationType,
    pub rhs: PreExp,
}

impl PreObjective {
    pub fn new(objective_type: OptimizationType, rhs: PreExp) -> Self {
        Self {
            objective_type,
            rhs,
        }
    }
}

#[derive(Debug)]
pub struct PreCondition {
    pub lhs: PreExp,
    pub condition_type: Comparison,
    pub rhs: PreExp,
    pub iteration: Vec<IterableSet>,
    pub span: InputSpan,
}

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
