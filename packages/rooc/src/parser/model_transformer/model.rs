use crate::domain_declaration::{Variable, format_domain};
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
use crate::runtime_builtin::{RoocFunction, make_std, make_std_constants};
use crate::traits::{ToLatex, escape_latex};
use crate::type_checker::type_checker_context::FunctionContext;
use crate::{primitives::Primitive, utils::Spanned};
use core::fmt;
use indexmap::IndexMap;
use serde::Serialize;

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
#[serde(tag = "type", content = "value")]
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
    /// Logic conjunction of multiple expressions
    And(Vec<Exp>),
    /// Logic disjunction of multiple expressions
    Or(Vec<Exp>),
    /// Logic negation of an expression
    Not(Box<Exp>),
    /// Logic exclusive disjunction between two expressions
    Xor(Box<Exp>, Box<Exp>),
    /// Logic implication between two expressions
    Implies(Box<Exp>, Box<Exp>),
    /// Logic biconditional between two expressions
    Iff(Box<Exp>, Box<Exp>),
    /// Binary operation between two expressions
    BinOp(BinOp, Box<Exp>, Box<Exp>),
    /// Unary operation on an expression
    UnOp(UnOp, Box<Exp>),
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
pub const IExp: &'static str = r#"
export type SerializedBinOp = { type: keyof typeof BinOp }
export type SerializedUnOp = { type: keyof typeof UnOp }
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
    type: "And",
    value: SerializedExp[]
} | {
    type: "Or",
    value: SerializedExp[]
} | {
    type: "Not",
    value: SerializedExp
} | {
    type: "Xor",
    value: [SerializedExp, SerializedExp]
} | {
    type: "Implies",
    value: [SerializedExp, SerializedExp]
} | {
    type: "Iff",
    value: [SerializedExp, SerializedExp]
} | {
    type: "BinOp",
    value: [SerializedBinOp, SerializedExp, SerializedExp]
} | {
    type: "UnOp",
    value: [SerializedUnOp, SerializedExp]
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
                match op {
                    BinOp::Add => match (lhs, rhs) {
                        (Exp::Number(lhs), Exp::Number(rhs)) => Exp::Number(lhs + rhs),
                        (Exp::Number(0.0), rhs) => rhs,
                        (lhs, Exp::Number(0.0)) => lhs,
                        (lhs, rhs) => Exp::BinOp(BinOp::Add, lhs.to_box(), rhs.to_box()),
                    },
                    BinOp::Sub => match (lhs, rhs) {
                        (Exp::Number(lhs), Exp::Number(rhs)) => Exp::Number(lhs - rhs),
                        (lhs, Exp::Number(0.0)) => lhs,
                        (lhs, rhs) => Exp::BinOp(BinOp::Sub, lhs.to_box(), rhs.to_box()),
                    },
                    BinOp::Mul => match (lhs, rhs) {
                        (Exp::Number(lhs), Exp::Number(rhs)) => Exp::Number(lhs * rhs),
                        (Exp::Number(0.0), _) | (_, Exp::Number(0.0)) => Exp::Number(0.0),
                        (Exp::Number(1.0), rhs) => rhs,
                        (lhs, Exp::Number(1.0)) => lhs,
                        (lhs, rhs) => Exp::BinOp(BinOp::Mul, lhs.to_box(), rhs.to_box()),
                    },
                    BinOp::Div => match (lhs, rhs) {
                        (Exp::Number(lhs), Exp::Number(rhs)) => {
                            if rhs == 0.0 {
                                Exp::BinOp(
                                    BinOp::Div,
                                    Exp::Number(lhs).to_box(),
                                    Exp::Number(rhs).to_box(),
                                )
                            } else {
                                Exp::Number(lhs / rhs)
                            }
                        }
                        (lhs, Exp::Number(1.0)) => lhs,
                        // Keep zero numerators and divisors visible so
                        // variable denominators and division by zero are still
                        // diagnosed by the linearizer.
                        (lhs, rhs) => Exp::BinOp(BinOp::Div, lhs.to_box(), rhs.to_box()),
                    },
                    // Logic operators are normalized into structural variants.
                    BinOp::And => Exp::And(vec![lhs, rhs]).simplify(),
                    BinOp::Or => Exp::Or(vec![lhs, rhs]).simplify(),
                    BinOp::Xor => Exp::Xor(lhs.to_box(), rhs.to_box()).simplify(),
                    BinOp::Implies => Exp::Implies(lhs.to_box(), rhs.to_box()).simplify(),
                    BinOp::Iff => Exp::Iff(lhs.to_box(), rhs.to_box()).simplify(),
                }
            }
            Exp::UnOp(op, exp) => {
                let exp = exp.simplify();
                match op {
                    UnOp::Neg => match exp {
                        Exp::Number(value) => Exp::Number(-value),
                        _ => Exp::UnOp(UnOp::Neg, exp.to_box()),
                    },
                    //normalized into the structural logic variant
                    UnOp::Not => match exp {
                        Exp::Number(value) => Exp::Number(logic_number(!num_truthy(value))),
                        exp => Exp::Not(exp.to_box()),
                    },
                }
            }
            Exp::Abs(exp) => {
                //if the inner expression is a number, return its absolute value
                let exp = exp.simplify();
                match exp {
                    Exp::Number(value) => Exp::Number(value.abs()),
                    exp => Exp::Abs(exp.to_box()),
                }
            }
            Exp::And(exps) => simplify_logic_nary(exps, true),
            Exp::Or(exps) => simplify_logic_nary(exps, false),
            Exp::Not(exp) => {
                let exp = exp.simplify();
                match exp {
                    Exp::Number(value) => Exp::Number(logic_number(!num_truthy(value))),
                    exp => Exp::Not(exp.to_box()),
                }
            }
            Exp::Xor(lhs, rhs) => {
                let lhs = lhs.simplify();
                let rhs = rhs.simplify();
                match (lhs, rhs) {
                    (Exp::Number(a), Exp::Number(b)) => {
                        Exp::Number(logic_number(num_truthy(a) != num_truthy(b)))
                    }
                    (lhs, rhs) => Exp::Xor(lhs.to_box(), rhs.to_box()),
                }
            }
            Exp::Implies(lhs, rhs) => {
                let lhs = lhs.simplify();
                let rhs = rhs.simplify();
                match (lhs, rhs) {
                    (Exp::Number(a), Exp::Number(b)) => {
                        Exp::Number(logic_number(!num_truthy(a) || num_truthy(b)))
                    }
                    (lhs, rhs) => Exp::Implies(lhs.to_box(), rhs.to_box()),
                }
            }
            Exp::Iff(lhs, rhs) => {
                let lhs = lhs.simplify();
                let rhs = rhs.simplify();
                match (lhs, rhs) {
                    (Exp::Number(a), Exp::Number(b)) => {
                        Exp::Number(logic_number(num_truthy(a) == num_truthy(b)))
                    }
                    (lhs, rhs) => Exp::Iff(lhs.to_box(), rhs.to_box()),
                }
            }
            Exp::Max(exps) => {
                if exps.is_empty() {
                    return Exp::Max(vec![]);
                }
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
                if exps.is_empty() {
                    return Exp::Min(vec![]);
                }
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
                (BinOp::Mul, Exp::UnOp(UnOp::Neg, lhs), c) => Exp::UnOp(
                    UnOp::Neg,
                    Exp::make_binop(BinOp::Mul, *lhs, c).flatten().to_box(),
                ),
                //a(-b) = -ab
                (BinOp::Mul, c, Exp::UnOp(UnOp::Neg, rhs)) => Exp::UnOp(
                    UnOp::Neg,
                    Exp::make_binop(BinOp::Mul, c, *rhs).flatten().to_box(),
                ),
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

                (BinOp::Add, lhs, rhs) => {
                    Exp::BinOp(BinOp::Add, lhs.flatten().to_box(), rhs.flatten().to_box())
                }
                (BinOp::Sub, lhs, rhs) => {
                    Exp::BinOp(BinOp::Sub, lhs.flatten().to_box(), rhs.flatten().to_box())
                }
                (BinOp::Mul, lhs, rhs) => {
                    Exp::BinOp(BinOp::Mul, lhs.flatten().to_box(), rhs.flatten().to_box())
                }
                (BinOp::Div, lhs, rhs) => {
                    Exp::BinOp(BinOp::Div, lhs.flatten().to_box(), rhs.flatten().to_box())
                }
                (BinOp::And, lhs, rhs) => {
                    Exp::BinOp(BinOp::And, lhs.flatten().to_box(), rhs.flatten().to_box())
                }
                (BinOp::Or, lhs, rhs) => {
                    Exp::BinOp(BinOp::Or, lhs.flatten().to_box(), rhs.flatten().to_box())
                }
                (BinOp::Xor, lhs, rhs) => {
                    Exp::BinOp(BinOp::Xor, lhs.flatten().to_box(), rhs.flatten().to_box())
                }
                (BinOp::Implies, lhs, rhs) => Exp::BinOp(
                    BinOp::Implies,
                    lhs.flatten().to_box(),
                    rhs.flatten().to_box(),
                ),
                (BinOp::Iff, lhs, rhs) => {
                    Exp::BinOp(BinOp::Iff, lhs.flatten().to_box(), rhs.flatten().to_box())
                }
            },
            _ => self,
        }
    }

    /// Checks if the expression is a leaf node (number or variable).
    ///
    /// # Returns
    /// true if the expression is a number or variable, false otherwise
    pub fn is_leaf(&self) -> bool {
        !matches!(
            self,
            Exp::BinOp(_, _, _)
                | Exp::UnOp(_, _)
                | Exp::And(_)
                | Exp::Or(_)
                | Exp::Not(_)
                | Exp::Xor(_, _)
                | Exp::Implies(_, _)
                | Exp::Iff(_, _)
        )
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
                        BinOp::Add
                        | BinOp::Mul
                        | BinOp::Div
                        | BinOp::And
                        | BinOp::Or
                        | BinOp::Xor
                        | BinOp::Implies
                        | BinOp::Iff => {
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

/// Whether a constant number counts as true, everything except zero does.
fn num_truthy(value: f64) -> bool {
    value != 0.0
}

/// Converts a truth value into its 0/1 numeric representation.
fn logic_number(value: bool) -> f64 {
    if value { 1.0 } else { 0.0 }
}

/// Simplifies an n-ary logic expression: children are simplified, nested
/// expressions of the same kind are flattened, identity constants dropped and
/// absorbing constants short-circuit the whole expression.
fn simplify_logic_nary(exps: &[Exp], is_and: bool) -> Exp {
    let mut flattened: Vec<Exp> = Vec::new();
    for exp in exps {
        let exp = exp.simplify();
        match (is_and, exp) {
            (true, Exp::And(inner)) => flattened.extend(inner),
            (false, Exp::Or(inner)) => flattened.extend(inner),
            (_, exp) => flattened.push(exp),
        }
    }
    let mut result: Vec<Exp> = Vec::new();
    for exp in flattened {
        if let Exp::Number(value) = exp {
            let truthy = num_truthy(value);
            if is_and && !truthy {
                return Exp::Number(0.0);
            }
            if !is_and && truthy {
                return Exp::Number(1.0);
            }
            //identity constants are dropped
        } else {
            result.push(exp);
        }
    }
    match result.len() {
        0 => Exp::Number(logic_number(is_and)),
        1 => result.into_iter().next().unwrap(),
        _ => {
            if is_and {
                Exp::And(result)
            } else {
                Exp::Or(result)
            }
        }
    }
}

/// Formats a logic operand, wrapping it in parenthesis unless it is a leaf
/// or a negation of a leaf, which are unambiguous on their own.
fn logic_operand_to_string(exp: &Exp) -> String {
    match exp {
        exp if exp.is_leaf() => exp.to_string(),
        Exp::Not(inner) if inner.is_leaf() => exp.to_string(),
        exp => format!("({})", exp),
    }
}

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Exp::Number(value) => value.to_string(),
            Exp::Variable(name) => name.clone(),
            Exp::Abs(exp) => format!("abs{{ {} }}", exp),
            Exp::And(exps) => exps
                .iter()
                .map(logic_operand_to_string)
                .collect::<Vec<_>>()
                .join(" and "),
            Exp::Or(exps) => exps
                .iter()
                .map(logic_operand_to_string)
                .collect::<Vec<_>>()
                .join(" or "),
            Exp::Not(exp) => {
                if exp.is_leaf() {
                    format!("not {}", exp)
                } else {
                    format!("not ({})", exp)
                }
            }
            Exp::Xor(lhs, rhs) => format!(
                "{} xor {}",
                logic_operand_to_string(lhs),
                logic_operand_to_string(rhs)
            ),
            Exp::Implies(lhs, rhs) => format!(
                "{} implies {}",
                logic_operand_to_string(lhs),
                logic_operand_to_string(rhs)
            ),
            Exp::Iff(lhs, rhs) => format!(
                "{} iff {}",
                logic_operand_to_string(lhs),
                logic_operand_to_string(rhs)
            ),
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
    objective_type: SerializedOptimizationType,
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
    /// Whether the source asserted the left-hand side without an explicit
    /// comparison.
    is_logic_assertion: bool,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
pub const ICondition: &'static str = r#"
export type SerializedCondition = {
    lhs: SerializedExp,
    constraint_type: SerializedComparison,
    rhs: SerializedExp,
    is_logic_assertion: boolean,
    name: string
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
            is_logic_assertion: false,
        }
    }

    /// Creates a constraint that directly asserts a logic expression.
    pub fn new_logic_assertion(lhs: Exp, name: String) -> Self {
        Self {
            name,
            lhs,
            constraint_type: Comparison::Equal,
            rhs: Exp::Number(1.0),
            is_logic_assertion: true,
        }
    }

    /// Decomposes the constraint into its components.
    ///
    /// # Returns
    /// A tuple of (lhs, comparison, rhs)
    pub fn into_parts(self) -> (Exp, Comparison, Exp, String) {
        (self.lhs, self.constraint_type, self.rhs, self.name)
    }

    /// Returns the constraint name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the left-hand side expression.
    pub fn lhs(&self) -> &Exp {
        &self.lhs
    }

    /// Returns the comparison operator.
    pub fn constraint_type(&self) -> Comparison {
        self.constraint_type
    }

    /// Returns the right-hand side expression.
    pub fn rhs(&self) -> &Exp {
        &self.rhs
    }

    /// Returns whether the source used the bare logic-assertion form.
    pub fn is_logic_assertion(&self) -> bool {
        self.is_logic_assertion
    }
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = if self.name.is_empty() {
            "".to_string()
        } else {
            format!("{}: ", self.name)
        };
        if self.is_logic_assertion {
            write!(f, "{}{}", name, self.lhs)
        } else {
            write!(
                f,
                "{}{} {} {}",
                name, self.lhs, self.constraint_type, self.rhs
            )
        }
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
) -> Result<Constraint, TransformError> {
    let lhs = constraint.lhs.into_exp(context, fn_context)?;
    let rhs = constraint.rhs.into_exp(context, fn_context)?;
    let name = match &constraint.name_exp {
        Some(v) => match v.value() {
            Variable::CompoundVariable(c) => {
                let indexes = c
                    .compute_indexes(context, fn_context)
                    .map_err(|e| e.add_span(v.span()))?;
                context
                    .flatten_compound_variable(&c.name, &indexes)
                    .map_err(|e| e.add_span(v.span()))?
            }
            Variable::Variable(name) => name.clone(),
        },
        // unnamed constraints stay unnamed: generated row names would only
        // clutter the compiled output
        None => String::new(),
    };
    if constraint.is_logic_assertion() {
        Ok(Constraint::new_logic_assertion(lhs, name))
    } else {
        Ok(Constraint::new(lhs, constraint.constraint_type, rhs, name))
    }
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
) -> Result<Vec<Constraint>, TransformError> {
    if constraint.iteration.is_empty() {
        return Ok(vec![transform_constraint(constraint, context, fn_context)?]);
    }
    let mut results: Vec<Constraint> = Vec::new();
    recursive_set_resolver(
        &constraint.iteration,
        context,
        fn_context,
        &mut results,
        0,
        &|c| transform_constraint(constraint, c, fn_context),
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
    for constraint in problem.constraints().iter() {
        let transformed = transform_constraint_with_iteration(constraint, &mut context, fn_context)?;
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

#[cfg(test)]
mod serialization_tests {
    use super::{Constraint, Exp};
    use crate::math::{BinOp, Comparison};
    use crate::parser::il::PreExp;
    use crate::utils::{InputSpan, Spanned};

    #[test]
    fn serialized_expressions_use_the_declared_tagged_shape() {
        let exp = Exp::Xor(
            Exp::Variable("a".to_string()).to_box(),
            Exp::Variable("b".to_string()).to_box(),
        );

        let serialized = serde_json::to_value(exp).unwrap();

        assert_eq!(serialized["type"], "Xor");
        assert_eq!(serialized["value"][0]["type"], "Variable");
        assert_eq!(serialized["value"][0]["value"], "a");
        assert_eq!(serialized["value"][1]["value"], "b");
    }

    #[test]
    fn serialized_pre_expressions_match_the_tuple_contract() {
        let span = InputSpan::default();
        let exp = PreExp::BinaryOperation(
            Spanned::new(BinOp::Or, span.clone()),
            PreExp::Variable(Spanned::new("a".to_string(), span.clone())).to_boxed(),
            PreExp::Variable(Spanned::new("b".to_string(), span)).to_boxed(),
        );

        let serialized = serde_json::to_value(exp).unwrap();

        assert_eq!(serialized["type"], "BinaryOperation");
        assert_eq!(serialized["value"][0]["value"]["type"], "Or");
        assert_eq!(serialized["value"][1]["type"], "Variable");
        assert_eq!(serialized["value"][1]["value"]["value"], "a");
    }

    #[test]
    fn serialized_operators_and_assertions_match_the_typescript_contract() {
        let exp = Exp::BinOp(
            BinOp::Add,
            Exp::Variable("a".to_string()).to_box(),
            Exp::Number(1.0).to_box(),
        );
        let serialized = serde_json::to_value(exp).unwrap();
        assert_eq!(serialized["value"][0]["type"], "Add");

        let assertion = Constraint::new_logic_assertion(
            Exp::Variable("flag".to_string()),
            "assert_flag".to_string(),
        );
        let serialized = serde_json::to_value(assertion).unwrap();
        assert_eq!(serialized["constraint_type"]["type"], "Equal");
        assert_eq!(serialized["is_logic_assertion"], true);

        let comparison = Constraint::new(
            Exp::Variable("flag".to_string()),
            Comparison::Equal,
            Exp::Number(1.0),
            "compare_flag".to_string(),
        );
        let serialized = serde_json::to_value(comparison).unwrap();
        assert_eq!(serialized["is_logic_assertion"], false);
    }
}
