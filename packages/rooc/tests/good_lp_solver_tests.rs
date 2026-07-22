#![cfg(any(feature = "coin_cbc", feature = "highs"))]

use indexmap::IndexMap;
use rooc::model_transformer::DomainVariable;
use rooc::{
    Comparison, InputSpan, LinearConstraint, LinearModel, LpSolution, OptimizationType,
    SolutionStatus, Solver, SolverError, VariableType,
};

fn mixed_domain_model() -> LinearModel {
    let mut model = LinearModel::new();
    model.add_variable("continuous", VariableType::non_negative_real());
    model.add_variable("integer", VariableType::IntegerRange(0, 4));
    model.add_variable("flag", VariableType::Boolean);
    model.add_named_constraint(
        vec![1.0, 2.0, 1.0],
        Comparison::GreaterOrEqual,
        4.0,
        "requirement",
    );
    model.add_named_constraint(
        vec![1.0, 1.0, 1.0],
        Comparison::LessOrEqual,
        5.0,
        "capacity",
    );
    model.add_named_constraint(
        vec![1.0, 0.0, 0.0],
        Comparison::LessOrEqual,
        1.0,
        "continuous_cap",
    );
    model.set_objective(vec![1.0, 2.0, 3.0], OptimizationType::Min);
    model
}

fn assert_mixed_domain_solution(solution: &LpSolution<f64>) {
    assert_eq!(solution.status(), SolutionStatus::Optimal);
    assert!((solution.value() - 4.0).abs() < 1e-7);
    assert!((solution.value_of("continuous").unwrap() - 0.0).abs() < 1e-7);
    assert!((solution.value_of("integer").unwrap() - 2.0).abs() < 1e-7);
    assert!((solution.value_of("flag").unwrap() - 0.0).abs() < 1e-7);
    assert!((solution.constraints()["requirement"] - 4.0).abs() < 1e-7);
    assert!((solution.constraints()["capacity"] - 2.0).abs() < 1e-7);
}

#[cfg(feature = "coin_cbc")]
#[test]
fn coin_cbc_solves_mixed_domain_model() {
    let model = mixed_domain_model();
    let solution =
        rooc::solve_lp_problem_coin_cbc(&model).expect("Coin CBC should solve the model");
    assert_mixed_domain_solution(&solution);

    let builder_solution = rooc::CoinCbc
        .solve(&model)
        .expect("builder solver should solve the model");
    assert_mixed_domain_solution(&builder_solution);
}

#[cfg(feature = "highs")]
#[test]
fn highs_solves_mixed_domain_model() {
    let model = mixed_domain_model();
    let solution = rooc::solve_lp_problem_highs(&model).expect("HiGHS should solve the model");
    assert_mixed_domain_solution(&solution);

    let builder_solution = rooc::Highs
        .solve(&model)
        .expect("builder solver should solve the model");
    assert_mixed_domain_solution(&builder_solution);
}

#[cfg(feature = "highs")]
#[test]
fn highs_preserves_objective_offset_and_comparison_activities() {
    let mut domain = IndexMap::new();
    domain.insert(
        "x".to_string(),
        DomainVariable::new(VariableType::non_negative_real(), InputSpan::default()),
    );
    let model = LinearModel::new_from_parts(
        vec![1.0],
        OptimizationType::Min,
        7.0,
        vec![
            LinearConstraint::new_with_name(
                vec![1.0],
                Comparison::LessOrEqual,
                5.0,
                "upper".to_string(),
            ),
            LinearConstraint::new_with_name(
                vec![1.0],
                Comparison::GreaterOrEqual,
                2.0,
                "lower".to_string(),
            ),
            LinearConstraint::new_with_name(vec![1.0], Comparison::Equal, 2.0, "fixed".to_string()),
        ],
        vec!["x".to_string()],
        domain,
    );

    let solution = rooc::solve_lp_problem_highs(&model).expect("HiGHS should solve the model");

    assert_eq!(solution.status(), SolutionStatus::Optimal);
    assert!((solution.value() - 9.0).abs() < 1e-7);
    assert!((solution.value_of("x").unwrap() - 2.0).abs() < 1e-7);
    assert!((solution.constraints()["upper"] - 2.0).abs() < 1e-7);
    assert!((solution.constraints()["lower"] - 2.0).abs() < 1e-7);
    assert!((solution.constraints()["fixed"] - 2.0).abs() < 1e-7);
}

#[cfg(feature = "highs")]
#[test]
fn highs_reports_infeasible_and_unbounded_models() {
    let mut infeasible = LinearModel::new();
    infeasible.add_variable("x", VariableType::non_negative_real());
    infeasible.add_constraint(vec![1.0], Comparison::LessOrEqual, 1.0);
    infeasible.add_constraint(vec![1.0], Comparison::GreaterOrEqual, 2.0);
    infeasible.set_objective(vec![1.0], OptimizationType::Min);
    assert!(matches!(
        rooc::solve_lp_problem_highs(&infeasible),
        Err(SolverError::Infeasible)
    ));

    let mut unbounded = LinearModel::new();
    unbounded.add_variable("x", VariableType::real());
    unbounded.set_objective(vec![1.0], OptimizationType::Max);
    assert!(matches!(
        rooc::solve_lp_problem_highs(&unbounded),
        Err(SolverError::Unbounded)
    ));
}
