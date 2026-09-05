#[cfg(any(feature = "clarabel", feature = "highs"))]
use rooc::{
    Comparison, DualValues, LinearModel, ModelBuilder, OptimizationType, VariableType, constraint,
    vars,
};
#[cfg(feature = "highs")]
use rooc::{Solver, SolverError};
#[cfg(any(
    feature = "coin_cbc",
    feature = "highs",
    feature = "lpsolve",
    feature = "scip",
    feature = "scip_bundled",
    feature = "cplex-rs"
))]
use std::time::Duration;

#[cfg(any(feature = "clarabel", feature = "highs"))]
fn lower_bound_model() -> LinearModel {
    let mut model = LinearModel::new();
    model.add_variable("x", VariableType::non_negative_real());
    model.add_named_constraint(vec![1.0], Comparison::GreaterOrEqual, 2.0, "lower");
    model.set_objective(vec![1.0], OptimizationType::Min);
    model
}

#[cfg(any(feature = "clarabel", feature = "highs"))]
fn named_cap_builder_model() -> LinearModel {
    let mut model = ModelBuilder::new();
    vars! { model =>
        x: nonneg;
        y: nonneg;
    };

    model
        .maximize(3.0 * x + 2.0 * y)
        .with(constraint!(capacity: x + y <= 4.0))
        .with(constraint!(x_cap: x <= 2.0))
        .with(constraint!(y_cap: y <= 3.0))
        .linearize()
        .unwrap()
}

/// `lower_bound_model` is `min x` subject to `x >= 2`, so raising that constraint's
/// right-hand side by one raises the objective by one: its shadow price is `+1`, and
/// every backend must agree on that. This asserted `-1` until good_lp 1.15.3.
///
/// good_lp normalizes every inequality to a `<=` row internally. Up to 1.15.2 the HiGHS
/// backend handed that normalized row straight to HiGHS, so `x >= 2` arrived as
/// `-x <= -2` and HiGHS returned the dual of the negated row - the correct value for the
/// row it was given, and the negation of the one the caller asked about. 1.15.3 added
/// `Constraint::is_greater_or_equal` specifically so backends can rebuild the original
/// row, and HiGHS now returns `+1` like Clarabel always did.
///
/// The floor on good_lp in Cargo.toml is 1.15.3 for this reason: on 1.15.2 this assertion
/// is off by a sign, and so is every shadow price a caller reads back from the HiGHS
/// solver.
#[cfg(feature = "highs")]
#[test]
fn highs_maps_named_constraint_duals() {
    let solution = rooc::solve_lp_problem_highs(&lower_bound_model())
        .unwrap()
        .into_solution()
        .expect("an unlimited solve must produce a solution");
    assert!((solution.shadow_price("lower").unwrap() - 1.0).abs() < 1e-7);
}

/// The same `+1` the HiGHS test above asserts. Clarabel has always reported it.
#[cfg(feature = "clarabel")]
#[test]
fn clarabel_maps_named_constraint_duals() {
    let solution = rooc::solve_real_lp_problem_clarabel(&lower_bound_model())
        .unwrap()
        .into_solution()
        .expect("an unlimited solve must produce a solution");
    assert!((solution.shadow_price("lower").unwrap() - 1.0).abs() < 1e-7);
}

#[cfg(feature = "highs")]
#[test]
fn highs_builder_named_caps_have_complete_shadow_prices() {
    let solution = rooc::solve_lp_problem_highs(&named_cap_builder_model())
        .unwrap()
        .into_solution()
        .expect("an unlimited solve must produce a solution");
    assert!((solution.shadow_price("capacity").unwrap().abs() - 2.0).abs() < 1e-6);
    assert!((solution.shadow_price("x_cap").unwrap().abs() - 1.0).abs() < 1e-6);
    assert!(solution.shadow_price("y_cap").unwrap().abs() < 1e-6);
}

#[cfg(feature = "clarabel")]
#[test]
fn clarabel_builder_named_caps_have_complete_shadow_prices() {
    let solution = rooc::solve_real_lp_problem_clarabel(&named_cap_builder_model())
        .unwrap()
        .into_solution()
        .expect("an unlimited solve must produce a solution");
    assert!((solution.shadow_price("capacity").unwrap().abs() - 2.0).abs() < 1e-6);
    assert!((solution.shadow_price("x_cap").unwrap().abs() - 1.0).abs() < 1e-6);
    assert!(solution.shadow_price("y_cap").unwrap().abs() < 1e-6);
}

#[cfg(feature = "highs")]
#[test]
fn highs_rejects_an_initial_solution_for_an_unknown_variable() {
    let result = rooc::Highs::new()
        .with_initial_solution([("missing", 1.0)])
        .solve(&lower_bound_model());
    assert!(matches!(
        result,
        Err(SolverError::Other(message)) if message.contains("missing")
    ));
}

#[cfg(feature = "highs")]
#[test]
fn highs_rejects_a_negative_mip_gap() {
    let result = rooc::Highs::new()
        .with_mip_gap(-0.1)
        .solve(&lower_bound_model());
    assert!(matches!(
        result,
        Err(SolverError::Other(message)) if message.contains("MIP gap")
    ));
}

#[cfg(feature = "highs")]
#[test]
fn highs_applies_time_limit_mip_gap_and_initial_solution_options() {
    let solution = rooc::Highs::new()
        .with_time_limit(Duration::from_secs(5))
        .with_mip_gap(0.0)
        .with_initial_solution([("x", 2.0)])
        .solve(&lower_bound_model())
        .unwrap()
        .into_solution()
        .expect("an unlimited solve must produce a solution");

    assert!((solution.value() - 2.0).abs() < 1e-7);
}

#[cfg(feature = "coin_cbc")]
#[test]
fn coin_cbc_exposes_good_lp_configuration_traits() {
    let _solver = rooc::CoinCbc::new()
        .with_time_limit(Duration::from_secs(5))
        .with_mip_gap(0.01)
        .with_initial_solution([("x", 2.0)]);
}

#[cfg(feature = "lpsolve")]
#[test]
fn lp_solve_exposes_its_good_lp_time_limit_trait() {
    let _solver = rooc::LpSolve::new().with_time_limit(Duration::from_secs(5));
}

#[cfg(any(feature = "scip", feature = "scip_bundled"))]
#[test]
fn scip_exposes_good_lp_configuration_traits() {
    let _solver = rooc::Scip::new()
        .with_time_limit(Duration::from_secs(5))
        .with_mip_gap(0.01)
        .with_initial_solution([("x", 2.0)]);
}

#[cfg(feature = "cplex-rs")]
#[test]
fn cplex_exposes_its_good_lp_time_limit_trait() {
    let _solver = rooc::Cplex::new().with_time_limit(Duration::from_secs(5));
}
