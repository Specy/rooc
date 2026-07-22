use rooc::{
    Comparison, ConstraintValues, LinearModel, OptimizationType, Solution, SolutionStatus,
    SolveStatus, Solver, VariableType,
};

fn conformance_model() -> LinearModel {
    let mut model = LinearModel::new();
    model.add_variable("x", VariableType::non_negative_real());
    model.add_named_constraint(vec![1.0], Comparison::GreaterOrEqual, 2.0, "minimum");
    model.set_objective(vec![1.0], OptimizationType::Min);
    model
}

fn assert_conforms<S>(solver: S)
where
    S: Solver,
    S::Solution: Solution + SolveStatus + ConstraintValues,
{
    let solution = solver.solve(&conformance_model()).unwrap();
    assert_eq!(SolveStatus::status(&solution), SolutionStatus::Optimal);
    assert!((solution.objective_value() - 2.0).abs() < 1e-7);
    let x: f64 = solution.var_value("x").unwrap().into();
    assert!((x - 2.0).abs() < 1e-7);
    assert!((solution.constraint_value("minimum").unwrap() - 2.0).abs() < 1e-7);
}

#[cfg(feature = "microlp")]
#[test]
fn microlp_conforms() {
    assert_conforms(rooc::Microlp::new());
}

#[cfg(feature = "clarabel")]
#[test]
fn clarabel_conforms() {
    assert_conforms(rooc::Clarabel);
}

#[cfg(feature = "coin_cbc")]
#[test]
fn coin_cbc_conforms() {
    assert_conforms(rooc::CoinCbc::new());
}

#[cfg(feature = "highs")]
#[test]
fn highs_conforms() {
    assert_conforms(rooc::Highs::new());
}

#[cfg(feature = "lpsolve")]
#[test]
fn lp_solve_conforms() {
    assert_conforms(rooc::LpSolve::new());
}

#[cfg(any(feature = "scip", feature = "scip_bundled"))]
#[test]
fn scip_conforms() {
    assert_conforms(rooc::Scip::new());
}

#[cfg(feature = "lp-solvers")]
#[test]
fn lp_solvers_conforms() {
    assert_conforms(rooc::LpSolvers);
}
