use crate::math::math_utils::{float_eq, float_ne};
use crate::pipe::pipe::{PipeDataType, PipeError, PipeableData};
use crate::pipe::pipe_executors::{
    CompilerPipe, LinearModelPipe, ModelPipe, OptimalTableauPipe, PreModelPipe,
    StandardLinearModelPipe, TableauPipe,
};
use crate::pipe::pipe_runner::PipeRunner;
use crate::solvers::simplex::{CanonicalTransformError, OptimalTableau, SimplexError};

#[allow(dead_code)]
fn solve(source: &str) -> Result<OptimalTableau, PipeError> {
    let pipe_runner = PipeRunner::new(vec![
        Box::new(CompilerPipe::new()),
        Box::new(PreModelPipe::new()),
        Box::new(ModelPipe::new()),
        Box::new(LinearModelPipe::new()),
        Box::new(StandardLinearModelPipe::new()),
        Box::new(TableauPipe::new()),
        Box::new(OptimalTableauPipe::new()),
    ]);

    let result = pipe_runner.run(PipeableData::String(source.to_string()));
    match result {
        Ok(data) => {
            let last = data.last().unwrap();
            match last {
                PipeableData::OptimalTableau(data) => Ok(data.clone()),
                _ => Err(PipeError::InvalidData {
                    expected: PipeDataType::OptimalTableau,
                    got: last.get_type(),
                }),
            }
        }
        Err((error, _context)) => {
            /*
            let context = context
                .iter()
                .map(|data| format!("//--------{}--------//\n\n{}", data.get_type(), data))
                .collect::<Vec<String>>()
                .join("\n\n");
             */
            Err(error)
        }
    }
}

#[allow(dead_code)]
fn assert_variables(variables: &Vec<f64>, expected: &Vec<f64>, lax_var_num: bool) {
    if variables.len() != expected.len() && !lax_var_num {
        panic!(
            "Different length, expected {:?} but got {:?}",
            expected, variables
        );
    }
    for (v, e) in variables.iter().zip(expected.iter()) {
        if float_ne(*v, *e) {
            panic!("{:?}!={:?} Expected  {:?} but got {:?}",v, e,  expected, variables);
        }
    }
}

fn assert_precision(a: f64, b: f64) -> bool {
    float_eq(a, b)
}


#[test]
fn should_solve_correctly() {
    let source = r#"
    max x_1 + 2x_2
    s.t.
        /* write the constraints here */
        x_2 <= 2x_1 + 2
        x_1 + 3x_2 <= 27
        x_1 + x_2 <= 15
        2x_1 <= x_2 + 18
    define
        x_1, x_2 as PositiveReal
    "#;
    let solution = solve(source).unwrap();
    assert_precision(solution.get_optimal_value(), 21.0);
    let variables = solution.get_variables_values();
    assert_variables(variables, &vec![9.0, 6.0, 14.0, 0.0, 0.0, 6.0], false);
}

#[test]
fn should_solve_correctly2() {
    let source = r#"
    max 2x_1 + 3x_2 + 4x_3 + 5x_4
    s.t.
        /* write the constraints here */
        x_1 + x_2 - x_3 + x_4 <= 10
        x_1 + 2x_2 <= 8
        x_3 + x_4 <= 20
    define
        x_1, x_2, x_3, x_4 as PositiveReal
    "#;
    let solution = solve(source).unwrap();
    assert_precision(solution.get_optimal_value(), 107.0);
    let variables = solution.get_variables_values();
    assert_variables(variables, &vec![8.0, 0.0, 9.0, 11.0, 0.0, 0.0, 0.0], false);
}

#[test]
fn should_solve_correctly_3() {
    let source = r#"
    min x_1 - x_2
    s.t.
        x_1 - x_2 >= -2
        -x_1 + 2x_2 >= -1
    define
        x_1, x_2 as PositiveReal
     "#;
    let solution = solve(source).unwrap();
    assert_precision(solution.get_optimal_value(), -2.0);
    let variables = solution.get_variables_values();
    assert_variables(variables, &vec![0.0, 2.0, 0.0, 5.0], false);
}

#[test]
fn should_find_unbounded_2d() {
    let source = r#"
    max x_1 + x_2
    s.t.
        x_1 - x_2 >= -2
        -x_1 + 2x_2 >= 1
    define
        x_1, x_2 as PositiveReal
     "#;
    let solution = solve(source);
    match solution {
        Ok(_) => panic!("Should not reach here"),
        Err(e) => {
            match e {
                PipeError::SimplexError(SimplexError::Unbounded, _tableau) => {
                    //TODO
                            /*

                            let variables = tableau.get_b();
                            let solution = tableau.get_current_value();
                            assert_precision(solution, 1.0);
                            assert_variables(variables, &vec![0.0, 1.0, 3.0, 0.0]);
                             */
                }
                _ => panic!("Should be unbounded"),
            }
        }
    }
}

#[test]
fn should_find_unbounded_4d() {
    let source = r#"
        max x_1 + x_2 + x_3 + x_4
        s.t.
            x_1 + x_2 - x_3 - 2x_4 <= 10
            2x_1 - 3x_2 + x_3 - x_4 <= 8
            x_1 - x_2 - x_3 + x_4 <= 7
        define
            x_1, x_2, x_3, x_4 as PositiveReal
     "#;
    let solution = solve(source);
    match solution {
        Ok(_) => panic!("Should not reach here"),
        Err(e) => {
            match e {
                PipeError::SimplexError(SimplexError::Unbounded, _tableau) => {
                    //TODO
                    /*
                    let variables = tableau.get_b();
                    println!("{}", tableau.to_string());
                    let solution = tableau.get_current_value();
                    assert_precision(solution, 10.0);
                    assert_variables(variables, &vec![0.0, 10.0, 0.0, 0.0, 0.0, 38.0, 17.0]);
                     */
                }
                _ => panic!("Should be unbounded"),
            }
        }
    }
}

#[test]
fn should_solve_degen_2d() {
    let source = r#"
    max x_1 + 2x_2
    s.t.
        x_2 <= 2x_1 + 2
        x_2 <= x_1 + 2
        x_2 <= (1/2)x_1 + 2
        x_1 <= 4
    define
        x_1, x_2 as PositiveReal
    "#;
    let solution = solve(source).unwrap();
    assert_precision(solution.get_optimal_value(), 12.0);
    let variables = solution.get_variables_values();
    assert_variables(variables, &vec![4.0, 4.0, 6.0, 2.0, 0.0, 0.0], false);
}

#[test]
fn should_solve_degen_4d() {
    let source = r#"
    max 2x_1 + 3x_2 + 4x_3 + 5x_4 + 10
    s.t.
        2x_1 + x_2 - x_3 <= 8
        2x_2 + x_3 - x_4 <= 10
        -x_1 + 2x_3 + x_4 <= 10
        x_1 - x_2 + 2x_4 <= 12
    define
        x_1, x_2, x_3, x_4 as PositiveReal
    "#;
    let solution = solve(source).unwrap();
    assert_precision(solution.get_optimal_value(), 84.0);
    let variables = solution.get_variables_values();
    assert_variables(variables, &vec![0.0, 8.0, 0.0, 10.0, 0.0, 4.0, 0.0, 0.0], false);
}

#[test]
fn should_solve_multiple_solutions() {
    let source = r#"
    max 2x_1 + x_2
    s.t.
        2x_1 + x_2 <= 18
        -x_1 + x_2 <= 4
        x_1 - x_2 <= 4
    define
        x_1, x_2 as PositiveReal
    "#;
    let solution = solve(source).unwrap();
    assert_precision(solution.get_optimal_value(), 18.0);
    let variables = solution.get_variables_values();
    // assert_variables(variables, &vec![14.0/3.0, 26.0/3.0, 0.0, 0.0, 8.0]); alternative solution
    assert_variables(variables, &vec![22.0 / 3.0, 10.0 / 3.0, 0.0, 8.0, 0.0], false);
}

#[test]
fn infeasible_starting_basis() {
    let source = r#"
    max x_2
    s.t.
        x_1 - 2x_2 <= -2
        -2x_1 + x_2 <= -4
        x_1 + x_2 <= 4
    define
        x_1, x_2 as PositiveReal
    "#;
    let solution = solve(source);
    match solution {
        Ok(_) => panic!("Should not reach here"),
        Err(e) => match e {
            PipeError::CanonicalizationError(CanonicalTransformError::Infesible(_)) => {}
            _ => panic!("Should be infeasible"),
        },
    }
}


//TODO make this work :(
#[test]
fn should_solve_diet(){
    let source = r#"
    min sum((cost, i) in enumerate(C)) { cost * x_i }
    s.t.
        sum(i in 0..F) { a[i][j] * x_i } >= Nmin[j] for j in 0..len(Nmin)
        sum(i in 0..F) { a[i][j] * x_i } <= Nmax[j] for j in 0..len(Nmax)
        x_i <= Fmax[i] for i in 0..N
        x_i >= Fmin[i] for i in 0..N
        where
        let C = [1.5, 0.5, 2.0]
        let Nmin = [50, 200, 0]
        let Nmax = [150, 300, 70]
        let Fmin = [1, 1, 1]
        let Fmax = [5, 5, 5]
        let a = [
            [30, 0, 5], // Chicken
            [2, 45, 0], // Rice
            [2, 15, 20] // Avocado
        ]
        let F = len(a)
        let N = len(Nmax)
    define
        x_i as PositiveReal for i in 0..N
    "#;
    let solution = solve(source).unwrap();
    assert_precision(solution.get_optimal_value(), 6.0);
    let variables = solution.get_variables_values();
    assert_variables(variables, &vec![1.32592, 4.11111, 1.0], true);

}