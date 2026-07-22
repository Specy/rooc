use std::fs;
use std::path::{Path, PathBuf};

fn crate_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

#[test]
fn good_lp_backends_use_explicit_modules_without_builder_macros() {
    for backend in [
        "clarabel.rs",
        "coin_cbc.rs",
        "highs.rs",
        "lpsolve.rs",
        "scip.rs",
        "lp_solvers.rs",
        "cplex.rs",
    ] {
        assert!(
            crate_path(&format!("src/solvers/{backend}")).is_file(),
            "missing low-level solver module {backend}"
        );
    }

    for backend in [
        "coin_cbc.rs",
        "highs.rs",
        "lpsolve.rs",
        "scip.rs",
        "lp_solvers.rs",
        "cplex.rs",
    ] {
        let path = crate_path(&format!("src/builder/solvers/{backend}"));
        assert!(path.is_file(), "missing builder solver module {backend}");
        let source = fs::read_to_string(path).unwrap();
        assert!(!source.contains("macro_rules!"));
    }

    assert!(!crate_path("src/builder/solvers/good_lp.rs").exists());

    let shared = fs::read_to_string(crate_path("src/solvers/good_lp.rs")).unwrap();
    for backend_function in [
        "solve_lp_problem_coin_cbc",
        "solve_lp_problem_highs",
        "solve_lp_problem_lpsolve",
        "solve_lp_problem_scip",
        "solve_lp_problem_lp_solvers",
        "solve_lp_problem_cplex",
    ] {
        assert!(
            !shared.contains(backend_function),
            "shared scaffold still contains {backend_function}"
        );
    }
}

#[cfg(feature = "clarabel")]
#[test]
fn clarabel_keeps_the_public_real_solver_module_path() {
    let solve: fn(&rooc::LinearModel) -> Result<rooc::LpSolution<f64>, rooc::SolverError> =
        rooc::real_solver::solve_real_lp_problem_clarabel;
    let _ = solve;
}
