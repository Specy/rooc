//! Integration tests for the fluent builder API.
//!
//! These exercise the public surface exactly as a downstream user would, so
//! they also guard the crate's re-exports.

use indexmap::IndexMap;
use rooc::builder::{any, sum};
use rooc::{
    Assignment, Auto, Clarabel, DualValues, LinearModel, LpSolution, MILPValue, Microlp,
    ModelBuilder, Reoptimize, SolutionStatus, Solution, Solver, SolverError, VariableType,
    constraint, vars,
};

fn bool_of(v: MILPValue) -> bool {
    match v {
        MILPValue::Bool(b) => b,
        other => panic!("expected a boolean value, got {other:?}"),
    }
}

fn int_of(v: MILPValue) -> i32 {
    match v {
        MILPValue::Int(i) => i,
        other => panic!("expected an integer value, got {other:?}"),
    }
}

/// The knapsack-with-logic model has a single optimum: x0 = x1 = true,
/// x2 = false, objective 13. Written objective-first in one chain.
#[test]
fn builder_solves_knapsack_with_logic_and_resolves_handles() {
    let mut model = ModelBuilder::new();
    vars! { model =>
        x0: bool;
        x1: bool;
        x2: bool;
    };

    let solution = model
        .maximize(5.0 * x0 + 8.0 * x1 + 12.0 * x2)
        .with(constraint!(2.0 * x0 + 3.0 * x1 + 5.0 * x2 <= 6.0))
        .with(constraint!(x0 -> x1))
        .with(constraint!(x1 -> !x2))
        .with(constraint!(x2 <-> !x0))
        .with(constraint!(any(vec![x0, x2])))
        .solve_with(Microlp::new())
        .expect("model should solve");

    assert_eq!(solution.value(), 13.0);
    assert!(bool_of(solution.var_value(x0).unwrap()));
    assert!(bool_of(solution.var_value(x1).unwrap()));
    assert!(!bool_of(solution.var_value(x2).unwrap()));
}

#[test]
fn var_value_resolves_each_handle_independently() {
    let mut model = ModelBuilder::new();
    let a = model.add_var("a", VariableType::integer_range(0, 10));
    let b = model.add_var("b", VariableType::integer_range(0, 10));
    let c = model.add_var("c", VariableType::integer_range(0, 10));

    let solution = model
        .maximize(sum(vec![a, b, c]))
        .with_all(vec![
            constraint!(a <= 2.0),
            constraint!(b <= 5.0),
            constraint!(c <= 9.0),
        ])
        .solve_with(Microlp::new())
        .expect("model should solve");

    assert_eq!(int_of(solution.var_value(a).unwrap()), 2);
    assert_eq!(int_of(solution.var_value(b).unwrap()), 5);
    assert_eq!(int_of(solution.var_value(c).unwrap()), 9);
}

#[test]
fn constraints_can_surround_the_objective() {
    let mut model = ModelBuilder::new();
    vars! { model =>
        x: int(0, 10);
        y: int(0, 10);
    };

    let solution = model
        .with(constraint!(x <= 3.0))
        .maximize(x + y)
        .with(constraint!(y <= 5.0))
        .solve_with(Microlp::new())
        .expect("model should solve");

    assert_eq!(int_of(solution.var_value(x).unwrap()), 3);
    assert_eq!(int_of(solution.var_value(y).unwrap()), 5);
}

#[test]
fn a_model_without_an_objective_defaults_to_satisfy() {
    let mut model = ModelBuilder::new();
    vars! { model => x: int(0, 10); };

    let solution = model
        .with(constraint!(x >= 4.0))
        .with(constraint!(x <= 7.0))
        .solve_with(Microlp::new())
        .expect("a feasible point should be found");

    let x_val = int_of(solution.var_value(x).unwrap());
    assert!((4..=7).contains(&x_val), "x = {x_val} is out of [4, 7]");
}

#[test]
#[should_panic(expected = "already")]
fn duplicate_variable_name_is_rejected() {
    let mut model = ModelBuilder::new();
    let _ = model.add_var("x", VariableType::bool());
    let _ = model.add_var("x", VariableType::bool());
}

#[test]
fn builder_solves_real_lp_with_clarabel() {
    let mut model = ModelBuilder::new();
    vars! { model =>
        x: nonneg;
        y: nonneg;
    };

    let solution = model
        .maximize(x + 2.0 * y)
        .with(constraint!(x + y <= 10.0))
        .solve_with(Clarabel)
        .expect("model should solve");

    assert!((solution.value() - 20.0).abs() < 1e-6);
    assert!((solution.var_value(y).unwrap() - 10.0).abs() < 1e-6);
    assert!(solution.var_value(x).unwrap().abs() < 1e-6);
}

#[test]
fn builder_auto_selects_a_solver() {
    let mut model = ModelBuilder::new();
    vars! { model =>
        x: int(0, 4);
        y: nonneg;
    };

    let solution = model
        .maximize(x + y)
        .with(constraint!(x + y <= 3.0))
        .solve_with(Auto)
        .expect("model should solve");

    assert!((solution.value() - 3.0).abs() < 1e-6);
}

// --- Custom solvers: the extension point -----------------------------------

/// A trivial solver that assigns every variable 0. It reuses the built-in
/// `LpSolution` as its solution type, which provides the core capabilities.
struct ZeroSolver;

impl Solver for ZeroSolver {
    type Solution = LpSolution<f64>;

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        let assignment = model
            .variables()
            .iter()
            .map(|name| Assignment {
                name: name.clone(),
                value: 0.0,
            })
            .collect();
        Ok(LpSolution::new(
            assignment,
            model.objective_offset(),
            Default::default(),
        ))
    }
}

#[test]
fn a_custom_solver_can_be_plugged_into_solve_with() {
    let mut model = ModelBuilder::new();
    vars! { model =>
        x: real(0.0, 10.0);
        y: real(0.0, 10.0);
    };

    let solution = model
        .minimize(x + y)
        .solve_with(ZeroSolver)
        .expect("custom solver should produce a solution");

    assert_eq!(solution.var_value(x), Some(0.0));
    assert_eq!(solution.var_value(y), Some(0.0));
}

/// A custom solver whose own solution type also implements `DualValues`, so its
/// solutions expose `shadow_price` while the built-in solvers' solutions do not.
struct DualSolution {
    inner: LpSolution<f64>,
    duals: IndexMap<String, f64>,
}

impl Solution for DualSolution {
    type Value = f64;
    fn objective_value(&self) -> f64 {
        self.inner.value()
    }
    fn var_value(&self, variable: &str) -> Option<f64> {
        self.inner.value_of(variable)
    }
}

impl DualValues for DualSolution {
    fn shadow_price(&self, constraint: &str) -> Option<f64> {
        self.duals.get(constraint).copied()
    }
}

struct DualSolver;

impl Solver for DualSolver {
    type Solution = DualSolution;

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        let assignment = model
            .variables()
            .iter()
            .map(|name| Assignment {
                name: name.clone(),
                value: 0.0,
            })
            .collect();
        let mut duals = IndexMap::new();
        for c in model.constraints() {
            if !c.name().is_empty() {
                duals.insert(c.name(), 1.5);
            }
        }
        Ok(DualSolution {
            inner: LpSolution::new(assignment, model.objective_offset(), Default::default()),
            duals,
        })
    }
}

#[test]
fn a_solver_that_implements_dual_values_exposes_shadow_prices() {
    let mut model = ModelBuilder::new();
    vars! { model => x: nonneg; };

    let solution = model
        .minimize(x)
        .with(constraint!(cap: x >= 2.0))
        .solve_with(DualSolver)
        .expect("model should solve");

    assert_eq!(solution.shadow_price("cap"), Some(1.5));
    assert_eq!(solution.shadow_price("missing"), None);
}

// --- Reading the solution --------------------------------------------------

#[test]
fn eval_computes_expression_values_at_the_solution() {
    let mut model = ModelBuilder::new();
    vars! { model =>
        x: int(0, 10);
        y: int(0, 10);
    };

    let solution = model
        .maximize(x + y)
        .with(constraint!(x <= 3.0))
        .with(constraint!(y <= 4.0))
        .solve_with(Microlp::new())
        .expect("model should solve");

    assert_eq!(solution.numeric_value(x), Some(3.0));
    assert_eq!(solution.eval(&(x + y)), 7.0);
    assert_eq!(solution.eval(&(2.0 * x + y)), 10.0);
}

#[test]
fn a_solved_model_reports_optimal_status() {
    let mut model = ModelBuilder::new();
    vars! { model => x: bool; };

    let solution = model
        .maximize(x)
        .with(constraint!(x <= 1.0))
        .solve_with(Microlp::new())
        .expect("model should solve");

    assert_eq!(solution.status(), SolutionStatus::Optimal);
}

#[test]
fn constraint_value_reads_a_named_constraint_activity() {
    let mut model = ModelBuilder::new();
    vars! { model =>
        x: int(0, 10);
        y: int(0, 10);
    };

    let solution = model
        .maximize(x + y)
        .with(constraint!(cap: x + y <= 6.0))
        .solve_with(Microlp::new())
        .expect("model should solve");

    assert_eq!(solution.constraint_value("cap"), Some(6.0));
    assert_eq!(solution.constraint_value("missing"), None);
}

// --- Continue solving ------------------------------------------------------

#[test]
fn a_solution_can_add_constraints_and_resolve() {
    let mut model = ModelBuilder::new();
    vars! { model => x: int(0, 10); };

    let solution = model
        .maximize(x)
        .solve_with(Microlp::new())
        .expect("model should solve");
    assert_eq!(int_of(solution.var_value(x).unwrap()), 10);

    // Add a constraint and re-solve from the solution.
    let tighter = solution
        .with(constraint!(x <= 4.0))
        .resolve()
        .expect("re-solve should succeed");
    assert_eq!(int_of(tighter.var_value(x).unwrap()), 4);
}

// --- MILP solver options ---------------------------------------------------

#[test]
fn microlp_options_still_return_the_optimum() {
    use std::time::Duration;

    let mut model = ModelBuilder::new();
    vars! { model =>
        x0: bool;
        x1: bool;
    };

    let solution = model
        .maximize(3.0 * x0 + 2.0 * x1)
        .with(constraint!(x0 + x1 <= 1.0))
        .solve_with(
            Microlp::new()
                .with_mip_gap(0.0)
                .with_time_limit(Duration::from_secs(5)),
        )
        .expect("model should solve");

    assert_eq!(solution.value(), 3.0);
    assert!(bool_of(solution.var_value(x0).unwrap()));
    assert!(!bool_of(solution.var_value(x1).unwrap()));
}

// --- vars! macro -----------------------------------------------------------

#[test]
fn indexed_vars_macro_binds_a_vec_and_indexes_in_constraints() {
    let mut model = ModelBuilder::new();
    vars! { model =>
        x[3]: bool;
    };
    assert_eq!(x.len(), 3);

    let solution = model
        .maximize(sum(x.iter().copied()))
        .with(constraint!(x[0] + x[1] + x[2] <= 2.0))
        .solve_with(Microlp::new())
        .expect("model should solve");

    assert_eq!(solution.value(), 2.0);
    let chosen = x
        .iter()
        .filter(|&&v| bool_of(solution.var_value(v).unwrap()))
        .count();
    assert_eq!(chosen, 2);
}

#[test]
fn add_vars_creates_an_indexed_family() {
    let mut model = ModelBuilder::new();
    let y = model.add_vars("y", 4, VariableType::integer_range(0, 3));
    assert_eq!(y.len(), 4);

    let solution = model
        .maximize(sum(y.iter().copied()))
        .with_all(y.iter().map(|&v| constraint!(v <= 2.0)))
        .solve_with(Microlp::new())
        .expect("model should solve");

    assert_eq!(solution.value(), 8.0);
}

#[test]
fn indexed_vars_macro_supports_every_domain_form() {
    let mut model = ModelBuilder::new();
    vars! { model =>
        a[2]: bool;
        b[2]: real;
        c[2]: real(-1.0, 1.0);
        d[2]: nonneg;
        e[2]: nonneg(0.0, 10.0);
        f[2]: int(0, 3);
    };
    assert_eq!(
        (a.len(), b.len(), c.len(), d.len(), e.len(), f.len()),
        (2, 2, 2, 2, 2, 2)
    );

    let all: Vec<_> = a
        .iter()
        .chain(&b)
        .chain(&c)
        .chain(&d)
        .chain(&e)
        .chain(&f)
        .copied()
        .collect();
    assert!(model.minimize(sum(all)).linearize().is_ok());
}

#[test]
fn lp_export_produces_cplex_lp_format() {
    let mut model = ModelBuilder::new();
    vars! { model =>
        x: bool;
        y: int(0, 5);
        z: nonneg(0.0, 10.0);
    };

    let lp = model
        .maximize(3.0 * x + 2.0 * y + z)
        .with(constraint!(cap: 2.0 * x + y + z <= 8.0))
        .linearize()
        .expect("model should linearize");

    let text = lp.to_lp_format();

    assert!(text.contains("Maximize"), "missing objective direction:\n{text}");
    assert!(text.contains("Subject To"), "missing constraints section:\n{text}");
    assert!(text.contains("cap:"), "named constraint not emitted:\n{text}");
    assert!(text.contains("<= 8"), "constraint rhs not emitted:\n{text}");
    assert!(text.contains("Binary"), "binary section missing:\n{text}");
    assert!(text.contains("General"), "general (integer) section missing:\n{text}");
    assert!(text.trim_end().ends_with("End"), "missing End marker:\n{text}");
}

#[test]
fn vars_macro_declares_handles_with_correct_domains() {
    let mut model = ModelBuilder::new();
    vars! { model =>
        pick: bool;
        count: int(0, 10);
        amount: real(0.0, 5.0);
    };

    let solution = model
        .maximize(pick + count + amount)
        .with(constraint!(count <= 4.0))
        .with(constraint!(amount <= 2.5))
        .solve_with(Microlp::new())
        .expect("model should solve");

    assert!(matches!(solution.var_value(pick).unwrap(), MILPValue::Bool(_)));
    assert!(matches!(solution.var_value(count).unwrap(), MILPValue::Int(_)));
    assert!(matches!(
        solution.var_value(amount).unwrap(),
        MILPValue::Real(_)
    ));
    assert_eq!(int_of(solution.var_value(count).unwrap()), 4);
}
