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
    let solution = solver.solve_using(solve_milp_lp_problem)?;

    assert_eq!(solution.value(), 1.0);
    Ok(())
}
