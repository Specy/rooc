#[cfg(any(feature = "clarabel", feature = "highs"))]
use rooc::{Comparison, DualValues, LinearModel, OptimizationType, VariableType};
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

#[cfg(feature = "highs")]
#[test]
fn highs_maps_named_constraint_duals() {
    let solution = rooc::solve_lp_problem_highs(&lower_bound_model()).unwrap();
    assert!((solution.shadow_price("lower").unwrap() + 1.0).abs() < 1e-7);
}

#[cfg(feature = "clarabel")]
#[test]
fn clarabel_maps_named_constraint_duals() {
    let solution = rooc::solve_real_lp_problem_clarabel(&lower_bound_model()).unwrap();
    assert!((solution.shadow_price("lower").unwrap() - 1.0).abs() < 1e-7);
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
        .unwrap();

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
