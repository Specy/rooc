use rooc::{RoocSolver, solve_milp_lp_problem};

#[test]
fn language_api_errors_can_be_propagated_with_question_mark()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "max x
s.t.
    x <= 1
define
    x as Boolean";

    let solver = RoocSolver::try_new(source.to_owned())?;
    // `InterruptedSolve` is an `Error`, so the outcome propagates with `?` too.
    let solution = solver.solve_using(solve_milp_lp_problem)?.into_solution()?;

    assert_eq!(solution.value(), 1.0);
    Ok(())
}
