//! Declarative macros for building expressions, constraints and variables.

// Macro helper declarations
#[doc(hidden)]
#[macro_export]
macro_rules! munch_expr {
    // 1. Found ->
    ([$($lhs:tt)*] -> $($rhs:tt)*) => {
        $crate::Expr::Implies(
            Box::new($crate::expr!($($lhs)*)),
            Box::new($crate::expr!($($rhs)*))
        )
    };
    // 2. Found <->
    ([$($lhs:tt)*] <-> $($rhs:tt)*) => {
        $crate::Expr::Iff(
            Box::new($crate::expr!($($lhs)*)),
            Box::new($crate::expr!($($rhs)*))
        )
    };
    // 3. Recurse
    ([$($accum:tt)*] $head:tt $($tail:tt)*) => {
        $crate::munch_expr!([$($accum)* $head] $($tail)*)
    };
    // 4. Base case
    ([$($val:tt)*]) => {
        $crate::Expr::from($($val)*)
    };
}

#[macro_export]
macro_rules! expr {
    ($($t:tt)*) => {
        $crate::munch_expr!([] $($t)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! munch_constraint {
    // 1. Found <=
    ([$($lhs:tt)*] <= $($rhs:tt)*) => {
        $crate::BuilderConstraint::new(
            $crate::expr!($($lhs)*),
            $crate::Comparison::LessOrEqual,
            $crate::expr!($($rhs)*),
            "".to_string()
        )
    };
    // 2. Found >=
    ([$($lhs:tt)*] >= $($rhs:tt)*) => {
        $crate::BuilderConstraint::new(
            $crate::expr!($($lhs)*),
            $crate::Comparison::GreaterOrEqual,
            $crate::expr!($($rhs)*),
            "".to_string()
        )
    };
    // 3. Found ==
    ([$($lhs:tt)*] == $($rhs:tt)*) => {
        $crate::BuilderConstraint::new(
            $crate::expr!($($lhs)*),
            $crate::Comparison::Equal,
            $crate::expr!($($rhs)*),
            "".to_string()
        )
    };
    // 4. Found <
    ([$($lhs:tt)*] < $($rhs:tt)*) => {
        $crate::BuilderConstraint::new(
            $crate::expr!($($lhs)*),
            $crate::Comparison::Less,
            $crate::expr!($($rhs)*),
            "".to_string()
        )
    };
    // 5. Found >
    ([$($lhs:tt)*] > $($rhs:tt)*) => {
        $crate::BuilderConstraint::new(
            $crate::expr!($($lhs)*),
            $crate::Comparison::Greater,
            $crate::expr!($($rhs)*),
            "".to_string()
        )
    };
    // 6. Found ->
    ([$($lhs:tt)*] -> $($rhs:tt)*) => {
        $crate::BuilderConstraint::new_logic_assertion(
            $crate::expr!($($lhs)* -> $($rhs)*),
            "".to_string()
        )
    };
    // 7. Found <->
    ([$($lhs:tt)*] <-> $($rhs:tt)*) => {
        $crate::BuilderConstraint::new_logic_assertion(
            $crate::expr!($($lhs)* <-> $($rhs)*),
            "".to_string()
        )
    };
    // 8. Recurse
    ([$($accum:tt)*] $head:tt $($tail:tt)*) => {
        $crate::munch_constraint!([$($accum)* $head] $($tail)*)
    };
    // 9. Base case
    ([$($logic:tt)*]) => {
        $crate::BuilderConstraint::new_logic_assertion(
            $crate::expr!($($logic)*),
            "".to_string()
        )
    };
}

#[macro_export]
macro_rules! constraint {
    ($name:ident : $($rest:tt)*) => {
        {
            let mut c = $crate::munch_constraint!([] $($rest)*);
            c.name = stringify!($name).to_string();
            c
        }
    };
    ($($rest:tt)*) => {
        $crate::munch_constraint!([] $($rest)*)
    };
}

/// Declares one or more variables on a [`ModelBuilder`](crate::ModelBuilder),
/// binding a copyable [`Var`](crate::Var) handle with the same name into the
/// current scope (in the style of `good_lp`'s `variables!`).
///
/// The identifier on the left of `:` becomes both the Rust binding and the
/// model variable name. Each declaration ends with `;`. Supported domains:
///
/// - `bool` -> boolean (0/1)
/// - `real` / `real(min, max)` -> real, unbounded or bounded
/// - `nonneg` / `nonneg(min, max)` -> non-negative real, unbounded or bounded
/// - `int(min, max)` -> integer in `[min, max]`
///
/// The model must be a plain binding (an identifier), because it is borrowed
/// once per declaration.
///
/// ```
/// use rooc::{vars, ModelBuilder};
/// let mut model = ModelBuilder::new();
/// vars! { model =>
///     x: bool;
///     y: int(0, 10);
///     z: real(0.0, 5.0);
/// };
/// let _ = (x, y, z); // x, y, z are Var handles in scope
/// ```
#[macro_export]
macro_rules! vars {
    ($model:ident => $($rest:tt)*) => {
        $crate::vars!(@munch $model $($rest)*);
    };

    // No more declarations.
    (@munch $model:ident) => {};

    // Array forms: `name[count]: domain;` binds a single `Vec<Var>`.
    (@munch $model:ident $name:ident [ $count:expr ] : bool ; $($rest:tt)*) => {
        let $name = $model.add_vars(stringify!($name), $count, $crate::VariableType::bool());
        $crate::vars!(@munch $model $($rest)*);
    };
    (@munch $model:ident $name:ident [ $count:expr ] : real ( $min:expr , $max:expr ) ; $($rest:tt)*) => {
        let $name = $model.add_vars(stringify!($name), $count, $crate::VariableType::Real($min, $max));
        $crate::vars!(@munch $model $($rest)*);
    };
    (@munch $model:ident $name:ident [ $count:expr ] : real ; $($rest:tt)*) => {
        let $name = $model.add_vars(stringify!($name), $count, $crate::VariableType::real());
        $crate::vars!(@munch $model $($rest)*);
    };
    (@munch $model:ident $name:ident [ $count:expr ] : nonneg ( $min:expr , $max:expr ) ; $($rest:tt)*) => {
        let $name = $model.add_vars(
            stringify!($name),
            $count,
            $crate::VariableType::NonNegativeReal($min, $max),
        );
        $crate::vars!(@munch $model $($rest)*);
    };
    (@munch $model:ident $name:ident [ $count:expr ] : nonneg ; $($rest:tt)*) => {
        let $name =
            $model.add_vars(stringify!($name), $count, $crate::VariableType::non_negative_real());
        $crate::vars!(@munch $model $($rest)*);
    };
    (@munch $model:ident $name:ident [ $count:expr ] : int ( $min:expr , $max:expr ) ; $($rest:tt)*) => {
        let $name = $model.add_vars(
            stringify!($name),
            $count,
            $crate::VariableType::integer_range($min, $max),
        );
        $crate::vars!(@munch $model $($rest)*);
    };

    // Boolean.
    (@munch $model:ident $name:ident : bool ; $($rest:tt)*) => {
        let $name = $model.add_var(stringify!($name), $crate::VariableType::bool());
        $crate::vars!(@munch $model $($rest)*);
    };

    // Real, bounded (must precede the unbounded rule).
    (@munch $model:ident $name:ident : real ( $min:expr , $max:expr ) ; $($rest:tt)*) => {
        let $name = $model.add_var(stringify!($name), $crate::VariableType::Real($min, $max));
        $crate::vars!(@munch $model $($rest)*);
    };
    // Real, unbounded.
    (@munch $model:ident $name:ident : real ; $($rest:tt)*) => {
        let $name = $model.add_var(stringify!($name), $crate::VariableType::real());
        $crate::vars!(@munch $model $($rest)*);
    };

    // Non-negative real, bounded (must precede the unbounded rule).
    (@munch $model:ident $name:ident : nonneg ( $min:expr , $max:expr ) ; $($rest:tt)*) => {
        let $name = $model.add_var(
            stringify!($name),
            $crate::VariableType::NonNegativeReal($min, $max),
        );
        $crate::vars!(@munch $model $($rest)*);
    };
    // Non-negative real, unbounded.
    (@munch $model:ident $name:ident : nonneg ; $($rest:tt)*) => {
        let $name = $model.add_var(stringify!($name), $crate::VariableType::non_negative_real());
        $crate::vars!(@munch $model $($rest)*);
    };

    // Integer range.
    (@munch $model:ident $name:ident : int ( $min:expr , $max:expr ) ; $($rest:tt)*) => {
        let $name = $model.add_var(
            stringify!($name),
            $crate::VariableType::integer_range($min, $max),
        );
        $crate::vars!(@munch $model $($rest)*);
    };
}
