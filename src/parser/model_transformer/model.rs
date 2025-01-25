use crate::domain_declaration::{format_domain, Variable};
use crate::math::{BinOp, UnOp};
use crate::math::{Comparison, OptimizationType};
use crate::parser::il::PreExp;
use crate::parser::il::{PreConstraint, PreObjective};
use crate::parser::model_transformer::transform_error::TransformError;
use crate::parser::model_transformer::transformer_context::{DomainVariable, TransformerContext};
use crate::parser::pre_model::PreModel;
use crate::parser::recursive_set_resolver::recursive_set_resolver;
#[allow(unused_imports)]
use crate::prelude::*;
use crate::primitives::Constant;
use crate::runtime_builtin::{make_std, make_std_constants, RoocFunction};
use crate::traits::{escape_latex, ToLatex};
use crate::type_checker::type_checker_context::FunctionContext;
use crate::{primitives::Primitive, utils::Spanned};
use core::fmt;
use indexmap::IndexMap;
use serde::Serialize;
use std::cell::Cell;

/// Represents a mathematical expression in the optimization model.
///
/// This enum defines the possible forms an expression can take, including:
/// - Numeric literals
/// - Variables
/// - Absolute value expressions
/// - Min/max of multiple expressions
/// - Binary operations (add, subtract, multiply, divide)
/// - Unary operations (negation)
#[derive(Debug, Clone, Serialize)]
pub enum Exp {
    /// A numeric literal value
    Number(f64),
    /// A named variable
    Variable(String),
    /// Absolute value of an expression
    Abs(Box<Exp>),
    /// Minimum of multiple expressions
    Min(Vec<Exp>),
    /// Maximum of multiple expressions
    Max(Vec<Exp>),
    /// Binary operation between two expressions
    BinOp(BinOp, Box<Exp>, Box<Exp>),
    /// Unary operation on an expression
    UnOp(UnOp, Box<Exp>),
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
pub const IExp: &'static str = r#"
export type SerializedExp = {
    type: "Number",
    value: number
} | {
    type: "Variable",
    value: string
} | {
    type: "Abs",
    value: SerializedExp
} | {
    type: "Min",
    value: SerializedExp[]
} | {
    type: "Max",
    value: SerializedExp[]
} | {
    type: "BinOp",
    value: {
        op: BinOp,
        lhs: SerializedExp,
        rhs: SerializedExp
    }
} | {
    type: "UnOp",
    value: {
        op: UnOp,
        exp: SerializedExp
    }
}
"#;

impl Exp {
    /// Creates a new binary operation expression.
    ///
    /// # Arguments
    /// * `op` - The binary operator
    /// * `lhs` - Left-hand side expression
    /// * `rhs` - Right-hand side expression
    ///
    /// # Returns
    /// A boxed binary operation expression
    pub fn make_binop(op: BinOp, lhs: Exp, rhs: Exp) -> Box<Self> {
        Exp::BinOp(op, lhs.to_box(), rhs.to_box()).to_box()
    }

    /// Converts an expression into a boxed expression.
    pub fn to_box(self) -> Box<Exp> {
        Box::new(self)
    }

    /// Converts a pre-expression into an expression.
    ///
    /// # Arguments
    /// * `pre_exp` - The pre-expression to convert
    /// * `context` - Transformer context containing variable information
    /// * `fn_context` - Function context containing function definitions
    ///
    /// # Returns
    /// The converted expression or a transform error
    pub fn from_pre_exp(
        pre_exp: &PreExp,
        context: &mut TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Self, TransformError> {
        pre_exp.into_exp(context, fn_context)
    }

    /// Simplifies the expression by applying algebraic rules.
    ///
    /// Performs simplifications like:
    /// - Evaluating constant expressions
    /// - Removing identity operations (x + 0, x * 1)
    /// - Simplifying operations with zero
    ///
    /// # Returns
    /// A new simplified expression
    pub fn simplify(&self) -> Exp {
        //implement the simplify function by using e-graphs egg
        match self {
            Exp::BinOp(op, lhs, rhs) => {
                let lhs = lhs.simplify();
                let rhs = rhs.simplify();
                match (op, lhs, rhs) {
                    (op, Exp::Number(lhs), Exp::Number(rhs)) => match op {
                        BinOp::Add => Exp::Number(lhs + rhs),
                        BinOp::Sub => Exp::Number(lhs - rhs),
                        BinOp::Mul => Exp::Number(lhs * rhs),
                        BinOp::Div => Exp::Number(lhs / rhs),
                    },
                    (BinOp::Add, Exp::Number(0.0), rhs) => rhs,
                    (BinOp::Add, lhs, Exp::Number(0.0)) => lhs,
                    (BinOp::Sub, lhs, Exp::Number(0.0)) => lhs,
                    (BinOp::Mul, Exp::Number(0.0), _) => Exp::Number(0.0),
                    (BinOp::Mul, _, Exp::Number(0.0)) => Exp::Number(0.0),
                    (BinOp::Mul, Exp::Number(1.0), rhs) => rhs,
                    (BinOp::Mul, lhs, Exp::Number(1.0)) => lhs,
                    (BinOp::Div, lhs, Exp::Number(1.0)) => lhs,
                    //this would be an error, keep it as it is
                    (BinOp::Div, lhs, Exp::Number(0.0)) => {
                        Exp::BinOp(BinOp::Div, lhs.to_box(), Exp::Number(0.0).to_box())
                    }
                    (BinOp::Div, Exp::Number(0.0), _) => Exp::Number(0.0),
                    // num1 + num2 + x = (num1 + num2) + x
                    // num1 - num2 - x = (num1 - num2) - x
                    // num1 * num2 * x = (num1 * num2) * x
                    // num1 / num2 / x = (num1 / num2) / x
                    (op, Exp::Number(lhs), Exp::BinOp(op2, inner_lhs, inner_rhs)) => {
                        let inner_lhs = inner_lhs.simplify();
                        let inner_rhs = inner_rhs.simplify();
                        if *op != op2 {
                            return Exp::BinOp(
                                *op,
                                Exp::Number(lhs).to_box(),
                                Exp::BinOp(op2, inner_lhs.to_box(), inner_rhs.to_box()).to_box(),
                            );
                        }
                        if let Exp::Number(rhs) = inner_lhs {
                            let val = match op {
                                BinOp::Add => lhs + rhs,
                                BinOp::Sub => lhs - rhs,
                                BinOp::Mul => lhs * rhs,
                                BinOp::Div => lhs / rhs,
                            };
                            Exp::BinOp(op2, Exp::Number(val).to_box(), inner_rhs.to_box())
                        } else {
                            Exp::BinOp(
                                *op,
                                Exp::Number(lhs).to_box(),
                                Exp::BinOp(op2, inner_lhs.to_box(), inner_rhs.to_box()).to_box(),
                            )
                        }
                    }
                    //keep the rest equal
                    (op, lhs, rhs) => Exp::BinOp(*op, lhs.to_box(), rhs.to_box()),
                }
            }
            Exp::UnOp(op, exp) => {
                let exp = exp.simplify();
                match op {
                    UnOp::Neg => match exp {
                        Exp::Number(value) => Exp::Number(-value),
                        _ => Exp::UnOp(UnOp::Neg, exp.to_box()),
                    },
                }
            }
            Exp::Max(exps) => {
                //if they are all numbers, return the max
                let nums = exps
                    .iter()
                    .map(|exp| {
                        let exp = exp.simplify();
                        if let Exp::Number(value) = exp {
                            Some(value)
                        } else {
                            None
                        }
                    })
                    .collect::<Option<Vec<f64>>>();
                match nums {
                    Some(nums) => {
                        Exp::Number(nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
                    }
                    None => Exp::Max(exps.iter().map(|exp| exp.simplify()).collect::<Vec<_>>()),
                }
            }
            Exp::Min(exps) => {
                //if they are all numbers, return the min
                let nums = exps
                    .iter()
                    .map(|exp| {
                        let exp = exp.simplify();
                        if let Exp::Number(value) = exp {
                            Some(value)
                        } else {
                            None
                        }
                    })
                    .collect::<Option<Vec<f64>>>();
                match nums {
                    Some(nums) => Exp::Number(nums.iter().cloned().fold(f64::INFINITY, f64::min)),
                    None => Exp::Min(exps.iter().map(|exp| exp.simplify()).collect::<Vec<_>>()),
                }
            }
            exp => exp.clone(),
        }
    }

    /// Flattens nested expressions by applying distributive properties.
    ///
    /// Applies transformations like:
    /// - (a + b)c = ac + bc
    /// - (a - b)c = ac - bc
    /// - -(a)b = -ab
    /// - (a + b)/c = a/c + b/c
    ///
    /// # Returns
    /// A new flattened expression
    pub fn flatten(self) -> Exp {
        match self {
            Exp::BinOp(op, lhs, rhs) => match (op, *lhs, *rhs) {
                //(a +- b)c = ac +- bc
                (BinOp::Mul, Exp::BinOp(inner_op @ (BinOp::Add | BinOp::Sub), lhs, rhs), c) => {
                    Exp::BinOp(
                        inner_op,
                        Exp::make_binop(BinOp::Mul, *lhs, c.clone()),
                        Exp::make_binop(BinOp::Mul, *rhs, c),
                    )
                    .flatten()
                }
                //c(a +- b) = ac +- bc
                (BinOp::Mul, c, Exp::BinOp(inner_op @ (BinOp::Add | BinOp::Sub), lhs, rhs)) => {
                    Exp::BinOp(
                        inner_op,
                        Exp::make_binop(BinOp::Mul, c.clone(), *lhs),
                        Exp::make_binop(BinOp::Mul, c, *rhs),
                    )
                    .flatten()
                }
                //-(a)b = -ab
                (BinOp::Mul, Exp::UnOp(op @ UnOp::Neg, lhs), c) => {
                    Exp::UnOp(op, Exp::make_binop(BinOp::Mul, *lhs, c).flatten().to_box())
                }
                //a(-b) = -ab
                (BinOp::Mul, c, Exp::UnOp(op @ UnOp::Neg, rhs)) => {
                    Exp::UnOp(op, Exp::make_binop(BinOp::Mul, c, *rhs).flatten().to_box())
                }
                //(a +- b)/c = a/c +- b/c
                (BinOp::Div, Exp::BinOp(inner_op @ (BinOp::Add | BinOp::Sub), lhs, rhs), c) => {
                    Exp::BinOp(
                        inner_op,
                        Exp::make_binop(BinOp::Div, *lhs, c.clone())
                            .flatten()
                            .to_box(),
                        Exp::make_binop(BinOp::Div, *rhs, c).flatten().to_box(),
                    )
                }

                (op, lhs, rhs) => Exp::BinOp(op, lhs.flatten().to_box(), rhs.flatten().to_box()),
            },
            _ => self,
        }
    }

    /// Checks if the expression is a leaf node (number or variable).
    ///
    /// # Returns
    /// true if the expression is a number or variable, false otherwise
    pub fn is_leaf(&self) -> bool {
        !matches!(self, Exp::BinOp(_, _, _) | Exp::UnOp(_, _))
    }

    /// Converts the expression to a string with proper operator precedence.
    ///
    /// # Arguments
    /// * `last_operator` - The operator from the parent expression for precedence comparison
    ///
    /// # Returns
    /// String representation with appropriate parentheses based on operator precedence
    pub fn to_string_with_precedence(&self, last_operator: BinOp) -> String {
        let last_precedence = last_operator.precedence();
        match self {
            Exp::BinOp(op, lhs, rhs) => {
                let string_lhs = lhs.to_string_with_precedence(*op);
                let string_rhs = rhs.to_string_with_precedence(*op);
                let precedence = op.precedence();
                if precedence < last_precedence {
                    format!("({} {} {})", string_lhs, op, string_rhs)
                } else {
                    //TODO improve this
                    match last_operator {
                        BinOp::Add | BinOp::Mul | BinOp::Div => {
                            format!("{} {} {}", string_lhs, op, string_rhs)
                        }
                        BinOp::Sub => match rhs.is_leaf() {
                            true => format!("{} {} {}", string_lhs, op, string_rhs),
                            false => format!("{} {} ({})", string_lhs, op, string_rhs),
                        },
                    }
                }
            }
            _ => self.to_string(),
        }
    }
}

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Exp::Number(value) => value.to_string(),
            Exp::Variable(name) => name.clone(),
            Exp::Abs(exp) => format!("|{}|", exp),
            Exp::Min(exps) => format!(
                "min{{ {} }}",
                exps.iter()
                    .map(|exp| exp.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Exp::Max(exps) => format!(
                "max{{ {} }}",
                exps.iter()
                    .map(|exp| exp.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Exp::BinOp(operator, lhs, rhs) => {
                //TODO: add parenthesis when needed
                let string_lhs = lhs.to_string_with_precedence(*operator);
                let string_rhs = rhs.to_string_with_precedence(*operator);
                format!("{} {} {}", string_lhs, operator, string_rhs)
            }
            Exp::UnOp(op, exp) => {
                if exp.is_leaf() {
                    format!("{}{}", op, exp)
                } else {
                    format!("{}({})", op, exp)
                }
            }
        };
        f.write_str(&s)
    }
}

/// Represents an optimization objective (minimize/maximize an expression).
#[derive(Debug, Serialize, Clone)]
pub struct Objective {
    /// Type of optimization (minimize or maximize)
    pub objective_type: OptimizationType,
    /// Expression to optimize
    pub rhs: Exp,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
pub const IObjective: &'static str = r#"
export type SerializedObjective = {
    objective_type: OptimizationType,
    rhs: SerializedExp
}
"#;

impl Objective {
    /// Creates a new optimization objective.
    ///
    /// # Arguments
    /// * `objective_type` - Whether to minimize or maximize
    /// * `rhs` - Expression to optimize
    pub fn new(objective_type: OptimizationType, rhs: Exp) -> Self {
        Self {
            objective_type,
            rhs,
        }
    }
}

impl fmt::Display for Objective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.objective_type, self.rhs)
    }
}

/// Represents a constraint in the optimization model (lhs comparison rhs).
#[derive(Debug, Clone, Serialize)]
pub struct Constraint {
    /// Name of the constraint
    name: String,
    /// Left-hand side expression
    lhs: Exp,
    /// Type of comparison (=, ≤, ≥, <, >)
    constraint_type: Comparison,
    /// Right-hand side expression
    rhs: Exp,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
pub const ICondition: &'static str = r#"
export type SerializedCondition = {
    lhs: SerializedExp,
    constraint_type: Comparison,
    rhs: SerializedExp
}
"#;

impl Constraint {
    /// Creates a new constraint.
    ///
    /// # Arguments
    /// * `lhs` - Left-hand side expression
    /// * `constraint_type` - Type of comparison (=, ≤, ≥, <, >)
    /// * `rhs` - Right-hand side expression
    /// * `name` - Name of the constraint
    pub fn new(lhs: Exp, constraint_type: Comparison, rhs: Exp, name: String) -> Self {
        Self {
            name,
            lhs,
            constraint_type,
            rhs,
        }
    }

    /// Decomposes the constraint into its components.
    ///
    /// # Returns
    /// A tuple of (lhs, comparison, rhs)
    pub fn into_parts(self) -> (Exp, Comparison, Exp, String) {
        (self.lhs, self.constraint_type, self.rhs, self.name)
    }
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = if self.name.starts_with("_c") {
            "".to_string()
        } else {
            format!("{}: ", self.name)
        };
        write!(
            f,
            "{}{} {} {}",
            name, self.lhs, self.constraint_type, self.rhs
        )
    }
}

/// Represents a complete optimization model.
///
/// Contains:
/// - An objective function to optimize
/// - A set of constraints that must be satisfied
/// - Domain information for variables
#[derive(Debug, Serialize, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Model {
    objective: Objective,
    constraints: Vec<Constraint>,
    domain: IndexMap<String, DomainVariable>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
pub const IModel: &'static str = r#"
export type SerializedModel = {
    objective: SerializedObjective,
    constraints: SerializedCondition[]
    domain: Record<string, DomainVariable>
}
"#;

impl Model {
    /// Creates a new optimization model.
    ///
    /// # Arguments
    /// * `objective` - The objective function to optimize
    /// * `constraints` - Vector of constraints that must be satisfied
    /// * `domain` - Map of variable names to their domains
    pub fn new(
        objective: Objective,
        constraints: Vec<Constraint>,
        domain: IndexMap<String, DomainVariable>,
    ) -> Self {
        Self {
            objective,
            constraints,
            domain,
        }
    }

    /// Decomposes the model into its components.
    ///
    /// # Returns
    /// A tuple of (objective, constraints, domain)
    pub fn into_components(self) -> (Objective, Vec<Constraint>, IndexMap<String, DomainVariable>) {
        (self.objective, self.constraints, self.domain)
    }

    /// Gets a reference to the objective function.
    pub fn objective(&self) -> &Objective {
        &self.objective
    }

    /// Gets a reference to the constraints.
    pub fn constraints(&self) -> &Vec<Constraint> {
        &self.constraints
    }

    /// Gets a reference to the variable domains.
    pub fn domain(&self) -> &IndexMap<String, DomainVariable> {
        &self.domain
    }

    /// Gets a mutable reference to the variable domains.
    pub fn domain_mut(&mut self) -> &mut IndexMap<String, DomainVariable> {
        &mut self.domain
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let constraints = self
            .constraints
            .iter()
            .map(|constraint| constraint.to_string())
            .collect::<Vec<_>>()
            .join("\n    ");
        let domain: String = if !self.domain.is_empty() {
            format!(
                "\ndefine\n    {}",
                format_domain(&self.domain)
                    .split("\n")
                    .collect::<Vec<_>>()
                    .join("\n    ")
            )
        } else {
            "".to_string()
        };
        write!(f, "{}\ns.t.\n    {}{}", self.objective, constraints, domain)
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
impl Model {
    pub fn to_string_wasm(&self) -> String {
        self.to_string()
    }
    pub fn serialize_wasm(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}

/// Represents a set of primitive values.
pub type PrimitiveSet = Vec<Primitive>;

/// Represents different kinds of variables in the model.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum VariableKind {
    /// A single variable with a name
    Single(Spanned<String>),
    /// A tuple of variables with names
    Tuple(Vec<Spanned<String>>),
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
pub const IVariableType: &'static str = r#"
export type SerializedVariableKind = {
    type: "Single",
    value: SerializedSpanned<string>
} | {
    type: "Tuple",
    value: SerializedSpanned<string>[]
}
"#;

impl ToLatex for VariableKind {
    fn to_latex(&self) -> String {
        match self {
            VariableKind::Single(name) => name.to_string(),
            VariableKind::Tuple(names) => format!(
                "({})",
                names
                    .iter()
                    .map(|name| escape_latex(name.value()))
                    .collect::<Vec<_>>()
                    .join(",\\ ")
            ),
        }
    }
}

impl fmt::Display for VariableKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VariableKind::Single(name) => f.write_str(name),
            VariableKind::Tuple(names) => write!(
                f,
                "({})",
                names
                    .iter()
                    .map(|name| name.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

/// Transforms a pre-constraint into a constraint.
///
/// # Arguments
/// * `constraint` - The pre-constraint to transform
/// * `context` - Transformer context containing variable information
/// * `fn_context` - Function context containing function definitions
///
/// # Returns
/// The transformed constraint or a transform error
pub fn transform_constraint(
    constraint: &PreConstraint,
    context: &mut TransformerContext,
    fn_context: &FunctionContext,
    index: usize,
) -> Result<Constraint, TransformError> {
    let lhs = constraint.lhs.into_exp(context, fn_context)?;
    let rhs = constraint.rhs.into_exp(context, fn_context)?;
    let name = match &constraint.name_exp {
        Some(v) => match v.value() {
            Variable::CompoundVariable(c) => {
                let indexes = c
                    .indexes
                    .iter()
                    .map(|v| v.as_primitive(context, fn_context))
                    .collect::<Result<Vec<Primitive>, TransformError>>()
                    .map_err(|e| e.add_span(v.span()))?;
                context
                    .flatten_compound_variable(&c.name, &indexes)
                    .map_err(|e| e.add_span(v.span()))?
            }
            Variable::Variable(name) => name.clone(),
        },
        None => format!("_c{}", index),
    };
    Ok(Constraint::new(lhs, constraint.constraint_type, rhs, name))
}

/// Transforms a pre-constraint with iteration into multiple constraints.
///
/// # Arguments
/// * `constraint` - The pre-constraint to transform
/// * `context` - Transformer context containing variable information
/// * `fn_context` - Function context containing function definitions
///
/// # Returns
/// A vector of transformed constraints or a transform error
pub fn transform_constraint_with_iteration(
    constraint: &PreConstraint,
    context: &mut TransformerContext,
    fn_context: &FunctionContext,
    last_index: usize,
) -> Result<Vec<Constraint>, TransformError> {
    let index = Cell::new(last_index);
    if constraint.iteration.is_empty() {
        return Ok(vec![transform_constraint(
            constraint, context, fn_context, last_index,
        )?]);
    }
    let mut results: Vec<Constraint> = Vec::new();
    recursive_set_resolver(
        &constraint.iteration,
        context,
        fn_context,
        &mut results,
        0,
        &|c| {
            index.set(index.get() + 1);
            transform_constraint(constraint, c, fn_context, index.get())
        },
    )
    .map_err(|e| e.add_span(&constraint.span))?;
    Ok(results)
}

/// Transforms a pre-objective into an objective.
///
/// # Arguments
/// * `objective` - The pre-objective to transform
/// * `context` - Transformer context containing variable information
/// * `fn_context` - Function context containing function definitions
///
/// # Returns
/// The transformed objective or a transform error
pub fn transform_objective(
    objective: &PreObjective,
    context: &mut TransformerContext,
    fn_context: &FunctionContext,
) -> Result<Objective, TransformError> {
    let rhs = objective.rhs.into_exp(context, fn_context)?;
    Ok(Objective::new(objective.objective_type.clone(), rhs))
}

/// Transforms a pre-model into a complete optimization model.
///
/// # Arguments
/// * `problem` - The pre-model to transform
/// * `context` - Transformer context containing variable information
/// * `fn_context` - Function context containing function definitions
///
/// # Returns
/// The transformed model or a transform error
pub fn transform_model(
    problem: PreModel,
    mut context: TransformerContext,
    fn_context: &FunctionContext,
) -> Result<Model, TransformError> {
    let objective = transform_objective(problem.objective(), &mut context, fn_context)?;
    let mut constraints: Vec<Constraint> = Vec::new();
    let mut index = 0;
    for constraint in problem.constraints().iter() {
        let transformed =
            transform_constraint_with_iteration(constraint, &mut context, fn_context, index)?;
        index += transformed.len();
        for transformed_constraint in transformed {
            constraints.push(transformed_constraint);
        }
    }
    let domain = context.into_components();
    Ok(Model::new(objective, constraints, domain))
}

/// Transforms a parsed problem into a complete optimization model.
///
/// # Arguments
/// * `pre_problem` - The parsed pre-model
/// * `constants` - Vector of constant values
/// * `fns` - Map of function names to implementations
///
/// # Returns
/// The transformed model or a transform error
pub fn transform_parsed_problem(
    pre_problem: PreModel,
    constants: Vec<Constant>,
    fns: &IndexMap<String, Box<dyn RoocFunction>>,
) -> Result<Model, TransformError> {
    let std = make_std();
    let fn_context = FunctionContext::new(fns, &std);
    let mut c = make_std_constants();
    c.extend(constants);
    c.extend(pre_problem.constants().clone());
    let context =
        TransformerContext::new_from_constants(c, pre_problem.domains().clone(), &fn_context)?;
    transform_model(pre_problem, context, &fn_context)
}
