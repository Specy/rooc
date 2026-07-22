//! Variables, expressions, operators and expression helpers for the builder.

use crate::math::{BinOp, UnOp};
use crate::parser::model_transformer::Exp;

/// Lightweight copyable variable handle referencing a variable index in the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Var {
    pub index: usize,
}

/// Index-based expression tree used while building a model.
///
/// This mirrors [`Exp`] but references variables by index rather than by name,
/// which is what lets [`Var`] be `Copy` and participate in operator overloading.
/// New variants must be kept in sync with `Exp` and handled in `to_exp`.
#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    Variable(usize),
    Abs(Box<Expr>),
    Min(Vec<Expr>),
    Max(Vec<Expr>),
    And(Vec<Expr>),
    Or(Vec<Expr>),
    Not(Box<Expr>),
    Xor(Box<Expr>, Box<Expr>),
    Implies(Box<Expr>, Box<Expr>),
    Iff(Box<Expr>, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    UnOp(UnOp, Box<Expr>),
}

impl From<Var> for Expr {
    fn from(v: Var) -> Self {
        Expr::Variable(v.index)
    }
}

impl From<f64> for Expr {
    fn from(n: f64) -> Self {
        Expr::Number(n)
    }
}

impl From<i32> for Expr {
    fn from(n: i32) -> Self {
        Expr::Number(n as f64)
    }
}

impl Expr {
    pub fn implies(self, other: impl Into<Expr>) -> Expr {
        Expr::Implies(Box::new(self), Box::new(other.into()))
    }

    pub fn iff(self, other: impl Into<Expr>) -> Expr {
        Expr::Iff(Box::new(self), Box::new(other.into()))
    }
}

impl Var {
    pub fn implies(self, other: impl Into<Expr>) -> Expr {
        Expr::from(self).implies(other)
    }

    pub fn iff(self, other: impl Into<Expr>) -> Expr {
        Expr::from(self).iff(other)
    }
}

/// Recursive helper to translate index-based Expr into name-based Exp.
pub(crate) fn to_exp(expr: &Expr, names: &[String]) -> Exp {
    match expr {
        Expr::Number(n) => Exp::Number(*n),
        Expr::Variable(idx) => Exp::Variable(names[*idx].clone()),
        Expr::Abs(inner) => Exp::Abs(Box::new(to_exp(inner, names))),
        Expr::Min(inners) => Exp::Min(inners.iter().map(|e| to_exp(e, names)).collect()),
        Expr::Max(inners) => Exp::Max(inners.iter().map(|e| to_exp(e, names)).collect()),
        Expr::And(inners) => Exp::And(inners.iter().map(|e| to_exp(e, names)).collect()),
        Expr::Or(inners) => Exp::Or(inners.iter().map(|e| to_exp(e, names)).collect()),
        Expr::Not(inner) => Exp::Not(Box::new(to_exp(inner, names))),
        Expr::Xor(lhs, rhs) => Exp::Xor(Box::new(to_exp(lhs, names)), Box::new(to_exp(rhs, names))),
        Expr::Implies(lhs, rhs) => {
            Exp::Implies(Box::new(to_exp(lhs, names)), Box::new(to_exp(rhs, names)))
        }
        Expr::Iff(lhs, rhs) => Exp::Iff(Box::new(to_exp(lhs, names)), Box::new(to_exp(rhs, names))),
        Expr::BinOp(op, lhs, rhs) => Exp::BinOp(
            *op,
            Box::new(to_exp(lhs, names)),
            Box::new(to_exp(rhs, names)),
        ),
        Expr::UnOp(op, inner) => Exp::UnOp(*op, Box::new(to_exp(inner, names))),
    }
}

/// Whether a numeric value counts as logically true (everything but zero).
fn truthy(x: f64) -> bool {
    x != 0.0
}

/// Converts a boolean into its 0/1 numeric representation.
fn bool_num(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

/// Evaluates an index-based [`Expr`] numerically, resolving each variable
/// through `var`. Logic operators follow the same nonzero-is-true semantics the
/// language uses.
pub(crate) fn eval_expr(expr: &Expr, var: &impl Fn(usize) -> f64) -> f64 {
    match expr {
        Expr::Number(n) => *n,
        Expr::Variable(i) => var(*i),
        Expr::Abs(e) => eval_expr(e, var).abs(),
        Expr::Min(es) => es
            .iter()
            .map(|e| eval_expr(e, var))
            .fold(f64::INFINITY, f64::min),
        Expr::Max(es) => es
            .iter()
            .map(|e| eval_expr(e, var))
            .fold(f64::NEG_INFINITY, f64::max),
        Expr::And(es) => bool_num(es.iter().all(|e| truthy(eval_expr(e, var)))),
        Expr::Or(es) => bool_num(es.iter().any(|e| truthy(eval_expr(e, var)))),
        Expr::Not(e) => bool_num(!truthy(eval_expr(e, var))),
        Expr::Xor(a, b) => bool_num(truthy(eval_expr(a, var)) != truthy(eval_expr(b, var))),
        Expr::Implies(a, b) => bool_num(!truthy(eval_expr(a, var)) || truthy(eval_expr(b, var))),
        Expr::Iff(a, b) => bool_num(truthy(eval_expr(a, var)) == truthy(eval_expr(b, var))),
        Expr::BinOp(op, a, b) => {
            let l = eval_expr(a, var);
            let r = eval_expr(b, var);
            match op {
                BinOp::Add => l + r,
                BinOp::Sub => l - r,
                BinOp::Mul => l * r,
                BinOp::Div => l / r,
                BinOp::And => bool_num(truthy(l) && truthy(r)),
                BinOp::Or => bool_num(truthy(l) || truthy(r)),
                BinOp::Xor => bool_num(truthy(l) != truthy(r)),
                BinOp::Implies => bool_num(!truthy(l) || truthy(r)),
                BinOp::Iff => bool_num(truthy(l) == truthy(r)),
            }
        }
        Expr::UnOp(op, e) => {
            let v = eval_expr(e, var);
            match op {
                UnOp::Neg => -v,
                UnOp::Not => bool_num(!truthy(v)),
            }
        }
    }
}

// Implement standard mathematical operators for Expr & Var combinations.
macro_rules! impl_binary_op {
    ($trait:ident, $method:ident, $op:expr) => {
        // Expr OP Expr
        impl std::ops::$trait<Expr> for Expr {
            type Output = Expr;
            fn $method(self, rhs: Expr) -> Self::Output {
                Expr::BinOp($op, Box::new(self), Box::new(rhs))
            }
        }
        // Expr OP &Expr
        impl std::ops::$trait<&Expr> for Expr {
            type Output = Expr;
            fn $method(self, rhs: &Expr) -> Self::Output {
                Expr::BinOp($op, Box::new(self), Box::new(rhs.clone()))
            }
        }
        // Var OP Var
        impl std::ops::$trait<Var> for Var {
            type Output = Expr;
            fn $method(self, rhs: Var) -> Self::Output {
                Expr::BinOp($op, Box::new(self.into()), Box::new(rhs.into()))
            }
        }
        // Var OP Expr
        impl std::ops::$trait<Expr> for Var {
            type Output = Expr;
            fn $method(self, rhs: Expr) -> Self::Output {
                Expr::BinOp($op, Box::new(self.into()), Box::new(rhs))
            }
        }
        // Expr OP Var
        impl std::ops::$trait<Var> for Expr {
            type Output = Expr;
            fn $method(self, rhs: Var) -> Self::Output {
                Expr::BinOp($op, Box::new(self), Box::new(rhs.into()))
            }
        }
        // Var OP f64
        impl std::ops::$trait<f64> for Var {
            type Output = Expr;
            fn $method(self, rhs: f64) -> Self::Output {
                Expr::BinOp($op, Box::new(self.into()), Box::new(Expr::Number(rhs)))
            }
        }
        // f64 OP Var
        impl std::ops::$trait<Var> for f64 {
            type Output = Expr;
            fn $method(self, rhs: Var) -> Self::Output {
                Expr::BinOp($op, Box::new(Expr::Number(self)), Box::new(rhs.into()))
            }
        }
        // Expr OP f64
        impl std::ops::$trait<f64> for Expr {
            type Output = Expr;
            fn $method(self, rhs: f64) -> Self::Output {
                Expr::BinOp($op, Box::new(self), Box::new(Expr::Number(rhs)))
            }
        }
        // f64 OP Expr
        impl std::ops::$trait<Expr> for f64 {
            type Output = Expr;
            fn $method(self, rhs: Expr) -> Self::Output {
                Expr::BinOp($op, Box::new(Expr::Number(self)), Box::new(rhs))
            }
        }
        // Var OP i32
        impl std::ops::$trait<i32> for Var {
            type Output = Expr;
            fn $method(self, rhs: i32) -> Self::Output {
                Expr::BinOp(
                    $op,
                    Box::new(self.into()),
                    Box::new(Expr::Number(rhs as f64)),
                )
            }
        }
        // i32 OP Var
        impl std::ops::$trait<Var> for i32 {
            type Output = Expr;
            fn $method(self, rhs: Var) -> Self::Output {
                Expr::BinOp(
                    $op,
                    Box::new(Expr::Number(self as f64)),
                    Box::new(rhs.into()),
                )
            }
        }
        // Expr OP i32
        impl std::ops::$trait<i32> for Expr {
            type Output = Expr;
            fn $method(self, rhs: i32) -> Self::Output {
                Expr::BinOp($op, Box::new(self), Box::new(Expr::Number(rhs as f64)))
            }
        }
        // i32 OP Expr
        impl std::ops::$trait<Expr> for i32 {
            type Output = Expr;
            fn $method(self, rhs: Expr) -> Self::Output {
                Expr::BinOp($op, Box::new(Expr::Number(self as f64)), Box::new(rhs))
            }
        }
    };
}

impl_binary_op!(Add, add, BinOp::Add);
impl_binary_op!(Sub, sub, BinOp::Sub);
impl_binary_op!(Mul, mul, BinOp::Mul);
impl_binary_op!(Div, div, BinOp::Div);

// Unary negation operator
impl std::ops::Neg for Expr {
    type Output = Expr;
    fn neg(self) -> Self::Output {
        Expr::UnOp(UnOp::Neg, Box::new(self))
    }
}
impl std::ops::Neg for Var {
    type Output = Expr;
    fn neg(self) -> Self::Output {
        Expr::UnOp(UnOp::Neg, Box::new(self.into()))
    }
}

// Logical AND
impl std::ops::BitAnd<Expr> for Expr {
    type Output = Expr;
    fn bitand(self, rhs: Expr) -> Self::Output {
        Expr::And(vec![self, rhs])
    }
}
impl std::ops::BitAnd<Var> for Expr {
    type Output = Expr;
    fn bitand(self, rhs: Var) -> Self::Output {
        Expr::And(vec![self, rhs.into()])
    }
}
impl std::ops::BitAnd<Expr> for Var {
    type Output = Expr;
    fn bitand(self, rhs: Expr) -> Self::Output {
        Expr::And(vec![self.into(), rhs])
    }
}
impl std::ops::BitAnd<Var> for Var {
    type Output = Expr;
    fn bitand(self, rhs: Var) -> Self::Output {
        Expr::And(vec![self.into(), rhs.into()])
    }
}
impl std::ops::BitAnd<bool> for Expr {
    type Output = Expr;
    fn bitand(self, rhs: bool) -> Self::Output {
        Expr::And(vec![self, Expr::Number(if rhs { 1.0 } else { 0.0 })])
    }
}
impl std::ops::BitAnd<bool> for Var {
    type Output = Expr;
    fn bitand(self, rhs: bool) -> Self::Output {
        Expr::And(vec![self.into(), Expr::Number(if rhs { 1.0 } else { 0.0 })])
    }
}
impl std::ops::BitAnd<Expr> for bool {
    type Output = Expr;
    fn bitand(self, rhs: Expr) -> Self::Output {
        Expr::And(vec![Expr::Number(if self { 1.0 } else { 0.0 }), rhs])
    }
}
impl std::ops::BitAnd<Var> for bool {
    type Output = Expr;
    fn bitand(self, rhs: Var) -> Self::Output {
        Expr::And(vec![Expr::Number(if self { 1.0 } else { 0.0 }), rhs.into()])
    }
}

// Logical OR
impl std::ops::BitOr<Expr> for Expr {
    type Output = Expr;
    fn bitor(self, rhs: Expr) -> Self::Output {
        Expr::Or(vec![self, rhs])
    }
}
impl std::ops::BitOr<Var> for Expr {
    type Output = Expr;
    fn bitor(self, rhs: Var) -> Self::Output {
        Expr::Or(vec![self, rhs.into()])
    }
}
impl std::ops::BitOr<Expr> for Var {
    type Output = Expr;
    fn bitor(self, rhs: Expr) -> Self::Output {
        Expr::Or(vec![self.into(), rhs])
    }
}
impl std::ops::BitOr<Var> for Var {
    type Output = Expr;
    fn bitor(self, rhs: Var) -> Self::Output {
        Expr::Or(vec![self.into(), rhs.into()])
    }
}
impl std::ops::BitOr<bool> for Expr {
    type Output = Expr;
    fn bitor(self, rhs: bool) -> Self::Output {
        Expr::Or(vec![self, Expr::Number(if rhs { 1.0 } else { 0.0 })])
    }
}
impl std::ops::BitOr<bool> for Var {
    type Output = Expr;
    fn bitor(self, rhs: bool) -> Self::Output {
        Expr::Or(vec![self.into(), Expr::Number(if rhs { 1.0 } else { 0.0 })])
    }
}
impl std::ops::BitOr<Expr> for bool {
    type Output = Expr;
    fn bitor(self, rhs: Expr) -> Self::Output {
        Expr::Or(vec![Expr::Number(if self { 1.0 } else { 0.0 }), rhs])
    }
}
impl std::ops::BitOr<Var> for bool {
    type Output = Expr;
    fn bitor(self, rhs: Var) -> Self::Output {
        Expr::Or(vec![Expr::Number(if self { 1.0 } else { 0.0 }), rhs.into()])
    }
}

// Logical XOR
impl std::ops::BitXor<Expr> for Expr {
    type Output = Expr;
    fn bitxor(self, rhs: Expr) -> Self::Output {
        Expr::Xor(Box::new(self), Box::new(rhs))
    }
}
impl std::ops::BitXor<Var> for Expr {
    type Output = Expr;
    fn bitxor(self, rhs: Var) -> Self::Output {
        Expr::Xor(Box::new(self), Box::new(rhs.into()))
    }
}
impl std::ops::BitXor<Expr> for Var {
    type Output = Expr;
    fn bitxor(self, rhs: Expr) -> Self::Output {
        Expr::Xor(Box::new(self.into()), Box::new(rhs))
    }
}
impl std::ops::BitXor<Var> for Var {
    type Output = Expr;
    fn bitxor(self, rhs: Var) -> Self::Output {
        Expr::Xor(Box::new(self.into()), Box::new(rhs.into()))
    }
}

// Logical NOT
impl std::ops::Not for Expr {
    type Output = Expr;
    fn not(self) -> Self::Output {
        Expr::Not(Box::new(self))
    }
}
impl std::ops::Not for Var {
    type Output = Expr;
    fn not(self) -> Self::Output {
        Expr::Not(Box::new(self.into()))
    }
}

// Global logic / block helper implementations
pub fn abs(expr: impl Into<Expr>) -> Expr {
    Expr::Abs(Box::new(expr.into()))
}

pub fn min(exprs: impl IntoIterator<Item = impl Into<Expr>>) -> Expr {
    Expr::Min(exprs.into_iter().map(|e| e.into()).collect())
}

pub fn max(exprs: impl IntoIterator<Item = impl Into<Expr>>) -> Expr {
    Expr::Max(exprs.into_iter().map(|e| e.into()).collect())
}

pub fn sum(exprs: impl IntoIterator<Item = impl Into<Expr>>) -> Expr {
    let mut iter = exprs.into_iter();
    if let Some(first) = iter.next() {
        let mut res = first.into();
        for item in iter {
            res = Expr::BinOp(BinOp::Add, Box::new(res), Box::new(item.into()));
        }
        res
    } else {
        Expr::Number(0.0)
    }
}

pub fn all(exprs: impl IntoIterator<Item = impl Into<Expr>>) -> Expr {
    Expr::And(exprs.into_iter().map(|e| e.into()).collect())
}

pub fn any(exprs: impl IntoIterator<Item = impl Into<Expr>>) -> Expr {
    Expr::Or(exprs.into_iter().map(|e| e.into()).collect())
}
