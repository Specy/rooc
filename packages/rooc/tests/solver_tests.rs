#[cfg(test)]
pub mod solver_tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    use indexmap::IndexMap;
    use rooc::common::LpSolution;
    use rooc::pipe::{
        CompilerPipe, LinearModelPipe, MILPSolverPipe,
        ModelPipe, PreModelPipe, RealSolver, StandardLinearModelPipe, TableauPipe,
    };
    use rooc::pipe::{PipeContext, PipeRunner};
    use rooc::pipe::{PipeDataType, PipeError, PipeableData, StepByStepSimplexPipe};
    #[allow(unused_imports)]
    use rooc::simplex::{CanonicalTransformError, OptimalTableau, SimplexError};
    use rooc::{MILPValue, OptimalTableauWithSteps};
    use rooc::{float_eq, float_ne};

    #[allow(unused)]
    #[allow(clippy::result_large_err)]
    fn solve(source: &str) -> Result<(OptimalTableauWithSteps, LpSolution<f64>), PipeError> {
        let pipe_runner = PipeRunner::new(vec![
            Box::new(CompilerPipe::new()),
            Box::new(PreModelPipe::new()),
            Box::new(ModelPipe::new()),
            Box::new(LinearModelPipe::new()),
            Box::new(RealSolver::new()),
        ]);

        let result = pipe_runner.run(
            PipeableData::String(source.to_string()),
            &PipeContext::new(vec![], &IndexMap::new()),
        );
        let simplex = match result {
            Ok(data) => {
                let last = data.last().unwrap();
                match last {
                    PipeableData::RealSolution(data) => Ok(data.clone()),
                    _ => Err(PipeError::InvalidData {
                        expected: PipeDataType::OptimalTableau,
                        got: last.get_type(),
                    }),
                }
            }
            Err((error, _context)) => Err(error),
        };
        let pipe_runner = PipeRunner::new(vec![
            Box::new(CompilerPipe::new()),
            Box::new(PreModelPipe::new()),
            Box::new(ModelPipe::new()),
            Box::new(LinearModelPipe::new()),
            Box::new(StandardLinearModelPipe::new()),
            Box::new(TableauPipe::new()),
            Box::new(StepByStepSimplexPipe::new()),
        ]);

        let result = pipe_runner.run(
            PipeableData::String(source.to_string()),
            &PipeContext::new(vec![], &IndexMap::new()),
        );
        let simplex2 = match result {
            Ok(data) => {
                let last = data.last().unwrap();
                match last {
                    PipeableData::OptimalTableauWithSteps(data) => Ok(data.clone()),
                    _ => Err(PipeError::InvalidData {
                        expected: PipeDataType::OptimalTableau,
                        got: last.get_type(),
                    }),
                }
            }
            Err((error, _context)) => Err(error),
        };
        Ok((simplex2?, simplex?))
    }
    #[allow(unused)]
    #[allow(clippy::result_large_err)]
    fn assert_correct_solution(
        solution: (OptimalTableauWithSteps, LpSolution<f64>),
        expected_value: f64,
        expected_solutions: Vec<Vec<f64>>,
    ) {
        let val_1 = solution.1.value();
        let val_2 = solution.0.result().optimal_value();
        assert_precision(val_1, expected_value);
        assert_precision(val_2, expected_value);
        let variables = solution.1.assignment_values();
        assert_variables(&variables, &expected_solutions, true);
        let variables_2 = solution.0.result().variables_values();
        assert_variables(variables_2, &expected_solutions, false);
    }

    #[allow(unused)]
    fn assert_correct_named_real_solution(
        solution: (OptimalTableauWithSteps, LpSolution<f64>),
        expected_value: f64,
        expected_variables: &[(&str, f64)],
    ) {
        assert_precision(solution.1.value(), expected_value);
        assert_precision(solution.0.result().optimal_value(), expected_value);
        let tableau_solution = solution.0.result().as_lp_solution();
        for (name, expected) in expected_variables {
            let real_value = solution
                .1
                .assignment()
                .iter()
                .find(|assignment| assignment.name == *name)
                .unwrap_or_else(|| panic!("missing variable {name} in real solver result"))
                .value;
            let tableau_value = tableau_solution
                .assignment()
                .iter()
                .find(|assignment| assignment.name == *name)
                .unwrap_or_else(|| panic!("missing variable {name} in tableau result"))
                .value;
            assert_precision(real_value, *expected);
            assert_precision(tableau_value, *expected);
        }
    }



    #[allow(unused)]
    #[allow(clippy::result_large_err)]
    fn solve_milp(source: &str) -> Result<LpSolution<MILPValue>, PipeError> {
        let pipe_runner = PipeRunner::new(vec![
            Box::new(CompilerPipe::new()),
            Box::new(PreModelPipe::new()),
            Box::new(ModelPipe::new()),
            Box::new(LinearModelPipe::new()),
            Box::new(MILPSolverPipe::new()),
        ]);

        let result = pipe_runner.run(
            PipeableData::String(source.to_string()),
            &PipeContext::new(vec![], &IndexMap::new()),
        );
        match result {
            Ok(data) => {
                let last = data.last().unwrap();
                match last {
                    PipeableData::MILPSolution(data) => Ok(data.clone()),
                    _ => Err(PipeError::InvalidData {
                        expected: PipeDataType::MILPSolution,
                        got: last.get_type(),
                    }),
                }
            }
            Err((error, _context)) => Err(error),
        }
    }

    #[allow(unused)]
    fn assert_variables(
        variables: &Vec<f64>,
        possible_solutions: &Vec<Vec<f64>>,
        lax_var_num: bool,
    ) {
        if variables.len() != possible_solutions[0].len() && !lax_var_num {
            panic!(
                "Different length, expected {:?} but got {:?}",
                possible_solutions[0], variables
            );
        }
        for solution in possible_solutions {
            let mut found = true;
            for (v, e) in variables.iter().zip(solution.iter()) {
                if float_ne(*v, *e) {
                    found = false;
                    break;
                }
            }
            if found {
                return;
            }
        }
        panic!(
            "Expected one of {:?} but got {:?}",
            possible_solutions, variables
        );
    }


    #[allow(unused)]
    fn assert_variables_milp(variables: &[MILPValue], expected: &[MILPValue], lax_var_num: bool) {
        if variables.len() != expected.len() && !lax_var_num {
            panic!(
                "Different length, expected {:?} but got {:?}",
                expected, variables
            );
        }
        for (v, e) in variables.iter().zip(expected.iter()) {
            match (v, e) {
                (MILPValue::Bool(v), MILPValue::Bool(e)) => {
                    if *v != *e {
                        panic!(
                            "{:?}!={:?} Expected  {:?} but got {:?}",
                            v, e, expected, variables
                        );
                    }
                }
                (MILPValue::Int(v), MILPValue::Int(e)) => {
                    if *v != *e {
                        panic!(
                            "{:?}!={:?} Expected  {:?} but got {:?}",
                            v, e, expected, variables
                        );
                    }
                }
                (MILPValue::Real(v), MILPValue::Real(e)) => {
                    if float_ne(*v, *e) {
                        panic!(
                            "{:?}!={:?} Expected  {:?} but got {:?}",
                            v, e, expected, variables
                        );
                    }
                }
                _ => panic!("Different types"),
            }
        }
    }

    #[allow(unused)]
    fn assert_precision(a: f64, b: f64) -> bool {
        if float_eq(a, b) {
            true
        } else {
            panic!("{} != {}", a, b);
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
        x_1, x_2 as NonNegativeReal
    "#;
        let solution = solve(source).unwrap();
        assert_correct_named_real_solution(solution, 21.0, &[("x_1", 9.0), ("x_2", 6.0)]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_correctly2() {
        let source = r#"
    max 2x_1 + 3x_2 + 4x_3 + 5x_4
    s.t.
        /* write the constraints here */
        x_1 + x_2 - x_3 + x_4 <= 10
        x_1 + 2x_2 <= 8
        x_3 + x_4 <= 20
    define
        x_1, x_2, x_3, x_4 as NonNegativeReal
    "#;
        let solution = solve(source).unwrap();
        assert_correct_named_real_solution(
            solution,
            107.0,
            &[("x_1", 8.0), ("x_2", 0.0), ("x_3", 9.0), ("x_4", 11.0)],
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_correctly_3() {
        let source = r#"
    min x_1 - x_2
    s.t.
        x_1 - x_2 >= -2
        -x_1 + 2x_2 >= -1
    define
        x_1, x_2 as NonNegativeReal
     "#;
        let solution = solve(source).unwrap();
        assert_correct_solution(
            solution,
            -2.0,
            vec![vec![0.0, 2.0, 0.0, 5.0], vec![0.66486, 2.66486]],
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_find_unbounded_2d() {
        let source = r#"
    max x_1 + x_2
    s.t.
        x_1 - x_2 >= -2
        -x_1 + 2x_2 >= 1
    define
        x_1, x_2 as NonNegativeReal
     "#;
        let solution = solve(source);
        match solution {
            Ok(_) => panic!("Should not reach here"),
            Err(e) => match e {
                PipeError::StepByStepSimplexError(SimplexError::Unbounded, _tableau) => {}
                _ => panic!("Should be unbounded"),
            },
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_find_unbounded_4d() {
        let source = r#"
        max x_1 + x_2 + x_3 + x_4
        s.t.
            x_1 + x_2 - x_3 - 2x_4 <= 10
            2x_1 - 3x_2 + x_3 - x_4 <= 8
            x_1 - x_2 - x_3 + x_4 <= 7
        define
            x_1, x_2, x_3, x_4 as NonNegativeReal
     "#;
        let solution = solve(source);
        match solution {
            Ok(_) => panic!("Should not reach here"),
            Err(e) => match e {
                PipeError::StepByStepSimplexError(SimplexError::Unbounded, _tableau) => {}
                _ => panic!("Should be unbounded"),
            },
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_degen_2d() {
        let source = r#"
    max x_1 + 2x_2
    s.t.
        x_2 <= 2x_1 + 2
        x_2 <= x_1 + 2
        x_2 <= (1/2)x_1 + 2
        x_1 <= 4
    define
        x_1, x_2 as NonNegativeReal
    "#;
        let solution = solve(source).unwrap();
        assert_correct_named_real_solution(solution, 12.0, &[("x_1", 4.0), ("x_2", 4.0)]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_degen_4d() {
        let source = r#"
    max 2x_1 + 3x_2 + 4x_3 + 5x_4 + 10
    s.t.
        2x_1 + x_2 - x_3 <= 8
        2x_2 + x_3 - x_4 <= 10
        -x_1 + 2x_3 + x_4 <= 10
        x_1 - x_2 + 2x_4 <= 12
    define
        x_1, x_2, x_3, x_4 as NonNegativeReal
    "#;
        let solution = solve(source).unwrap();
        assert_correct_solution(
            solution,
            84.0,
            vec![vec![0.0, 8.0, 0.0, 10.0, 0.0, 4.0, 0.0, 0.0]],
        );
    }
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_non_unbounded() {
        let source = r#"
             max x + sum(i in list) { z_i }
             subject to
                 //write the constraints here
                 x <= y
                 cons_i: z_i <= i * 2 for i in list
             where
                 // write the constants here
                 let y = 10
                 let list = [2,4,6]
             define
                 // define the model's variables here
                 x as NonNegativeReal
                 z_i as NonNegativeReal for i in list
     "#;
        let solution = solve(source).unwrap();
        assert_correct_named_real_solution(
            solution,
            34.0,
            &[("x", 10.0), ("z_2", 4.0), ("z_4", 8.0), ("z_6", 12.0)],
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_multiple_solutions() {
        let source = r#"
    max 2x_1 + x_2
    s.t.
        2x_1 + x_2 <= 18
        -x_1 + x_2 <= 4
        x_1 - x_2 <= 4
    define
        x_1, x_2 as NonNegativeReal
    "#;
        let solution = solve(source).unwrap();
        assert_precision(solution.1.value(), 18.0);
        assert_precision(solution.0.result().optimal_value(), 18.0);
        let tableau_solution = solution.0.result().as_lp_solution();
        for result in [&solution.1, &tableau_solution] {
            let x_1 = result
                .assignment()
                .iter()
                .find(|assignment| assignment.name == "x_1")
                .expect("missing x_1")
                .value;
            let x_2 = result
                .assignment()
                .iter()
                .find(|assignment| assignment.name == "x_2")
                .expect("missing x_2")
                .value;
            assert_precision(2.0 * x_1 + x_2, 18.0);
            assert!(x_2 <= x_1 + 4.0 + 1e-9);
            assert!(x_1 <= x_2 + 4.0 + 1e-9);
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn infeasible_starting_basis() {
        let source = r#"
    max x_2
    s.t.
        x_1 - 2x_2 <= -2
        -2x_1 + x_2 <= -4
        x_1 + x_2 <= 4
    define
        x_1, x_2 as NonNegativeReal
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

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_diet() {
        let source = r#"
//This is a simple diet problem
//minimize the cost of the diet
min sum((cost, i) in enumerate(C)) { cost * x_i }
s.t.
    //the diet must have at least of nutrient j
    sum(i in 0..F) { a[i][j] * x_i} >= Nmin[j] for j in 0..len(Nmin)
    //the diet must have at most of nutrient j
    sum(i in 0..F) { a[i][j] * x_i } <= Nmax[j] for j in 0..len(Nmax)
where
    // Cost of chicken, rice, avocado
    let C = [1.5, 0.5, 2.0]
    // Min and max of: protein, carbs, fats
    let Nmin = [50, 200, 0]
    let Nmax = [150, 300, 70]
    // Min and max servings of each food
    let Fmin = [1, 1, 1]
    let Fmax = [5, 5, 5]
    let a = [
        //protein, carbs, fats
        [30, 0, 5], // Chicken
        [2, 45, 0], // Rice
        [2, 15, 20] // Avocado
    ]
    // Number of foods
    let F = len(a)
define
    //bound the amount of each serving of food i
    x_i as NonNegativeReal(Fmin[i], Fmax[i]) for i in 0..len(Nmax)

    "#;
        let solution = solve(source).unwrap();
        assert_correct_named_real_solution(
            solution,
            6.04444,
            &[
                ("x_0", 1.3259259259259264),
                ("x_1", 4.111111111111111),
                ("x_2", 0.9999999999999998),
            ],
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic]
    fn should_be_unbounded() {
        let source = r#"
    min x_1 + 2x_2 - x_3
s.t.
    -x_1 + x_2 = 5
    2x_1 - x_2 - x_3 <= 3
define
    x_1 as Real
    x_2, x_3 as NonNegativeReal
    "#;
        //TODO improve test detection
        solve(source).unwrap();
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic]
    fn should_transform_free_variables() {
        todo!("Create this test plis")
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_binary_problem() {
        let source = r#"
    //knapsack problem
    max sum((value, i) in enumerate(values)) { value * x_i }
    s.t.
        sum((weight, i) in enumerate(weights)) { weight * x_i } <= capacity
    where
        let weights = [10, 60, 30, 40, 30, 20, 20, 2]
        let values = [1, 10, 15, 40, 60, 90, 100, 15]
        let capacity = 102
    define
        x_i as Boolean for i in 0..len(weights)
    "#;
        let solution = solve_milp(source).unwrap();
        assert_precision(solution.value(), 280.0);
        assert_variables_milp(
            &solution.assignment_values(),
            &vec![
                MILPValue::Bool(false),
                MILPValue::Bool(false),
                MILPValue::Bool(true),
                MILPValue::Bool(false),
                MILPValue::Bool(true),
                MILPValue::Bool(true),
                MILPValue::Bool(true),
                MILPValue::Bool(true),
            ],
            true,
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_apply_unary_minus_to_boolean_variables() {
        //prefix minus must behave like 0 - x on 0/1 variables
        let source = "max -x + y\ns.t.\n    x + y >= 1\ndefine\n    x, y as Boolean";
        let solution = solve_milp(source).unwrap();
        assert_precision(solution.value(), 1.0);
        assert_variables_milp(&solution.assignment_values(), &vec![MILPValue::Bool(false), MILPValue::Bool(true)], true);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_integer_problem() {
        let source = r#"
    max 2x_1 + 3x_2
    s.t.
        x_1 + x_2 <= 7
        2x_1 + 3x_2 <= 21
    define
        x_1, x_2 as IntegerRange(0, 10)
    "#;
        let solution = solve_milp(source).unwrap();
        assert_precision(solution.value(), 21.0);
        let assignment = solution.assignment_values();
        assert_variables_milp(
            &assignment,
            &[MILPValue::Int(0), MILPValue::Int(7)],
            false,
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic]
    fn should_detect_invalid_domain() {
        let source = r#"
    max 2x_1 + 3x_2
    s.t.
        x_1 + x_2 <= 7
        2x_1 + 3x_2 <= 21
    define
        x_1 as Real
        x_2 as IntegerRange(0, 10)
    "#; //here only Real and NonNegativeReal are allowed
        solve(source).unwrap();
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_dynamic_domain() {
        let source = r#"
    max sum((value, i) in enumerate(arr)) { x_i }
    s.t.
        sum((value, i) in enumerate(arr)) { x_i } <= 1000
    where
        let arr = [1,2,3,4]
    define
        x_i as IntegerRange(0, arr[i]) for i in 0..len(arr)
    "#; //here only Boolean is allowed
        let result = solve_milp(source).unwrap();
        assert_precision(result.value(), 10.0);
        let assignment = result.assignment_values();
        assert_variables_milp(
            &assignment,
            &[
                MILPValue::Int(1),
                MILPValue::Int(2),
                MILPValue::Int(3),
                MILPValue::Int(4),
            ],
            false,
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_milp_problem() {
        let source = "
max 50 * x + 40 * y + 45 * z
s.t.
    // Machine time constraint
    3 * x + 2 * y + 1 * z <= 20
     // Labor time constraint
    2 * x + 1 * y + 3 * z <= 15
     // Minimum production constraint for Product A
    x >= 2
    // Maximum production constraint for Product B
    y <= 7
define
    x, y as NonNegativeReal
    z as IntegerRange(0, 10)";

        let result = solve_milp(source).unwrap();
        assert_precision(result.value(), 405.0);
        let assignment = result.assignment_values();
        assert_variables_milp(
            &assignment,
            &[
                MILPValue::Real(2.0),
                MILPValue::Real(6.5),
                MILPValue::Int(1),
            ],
            false,
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_abs_in_objective() {
        //|x - 7| is replaced by $abs_0 with $abs_0 >= x - 7 and $abs_0 >= -(x - 7),
        //minimizing pushes $abs_0 down onto |x - 7|
        let source = r#"
    min abs { x - 7 }
    s.t.
        x <= 3
    define
        x as NonNegativeReal
    "#;
        let solution = solve(source).unwrap();
        assert_correct_named_real_solution(solution, 4.0, &[("x", 3.0)]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_abs_in_constraints() {
        //|e| <= c bounds e within [-c, c], this reformulation is exact
        let source = r#"
    max x + y
    s.t.
        abs { x - 4 } <= 2
        abs { y - 1 } <= 3
    define
        x, y as NonNegativeReal
    "#;
        let solution = solve(source).unwrap();
        assert_correct_named_real_solution(solution, 10.0, &[("x", 6.0), ("y", 4.0)]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_multiple_abs_with_coefficients() {
        let source = r#"
    min abs { x - 6 } + 2 * abs { y - 3 }
    s.t.
        x <= 4
        y <= 2
    define
        x, y as NonNegativeReal
    "#;
        let solution = solve(source).unwrap();
        assert_correct_named_real_solution(solution, 4.0, &[("x", 4.0), ("y", 2.0)]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_abs_of_multivariable_expression() {
        let source = r#"
    min abs { x + y - 10 }
    s.t.
        x <= 3
        y <= 4
    define
        x, y as NonNegativeReal
    "#;
        let solution = solve(source).unwrap();
        assert_correct_named_real_solution(solution, 3.0, &[("x", 3.0), ("y", 4.0)]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_milp_with_abs() {
        let source = r#"
    min abs { 2x - 7 }
    s.t.
        x <= 3
    define
        x as IntegerRange(0, 10)
    "#;
        let result = solve_milp(source).unwrap();
        assert_precision(result.value(), 1.0);
        let x = result
            .assignment()
            .iter()
            .find(|assignment| assignment.name == "x")
            .expect("missing x in MILP result");
        assert!(matches!(x.value, MILPValue::Int(3)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_abs_of_free_variable() {
        //the inner expression is negative at the optimum
        let source = r#"
    min abs { x + 5 }
    s.t.
        x <= -8
    define
        x as Real
    "#;
        let result = solve_milp(source).unwrap();
        assert_precision(result.value(), 3.0);
        let x = result
            .assignment()
            .iter()
            .find(|assignment| assignment.name == "x")
            .expect("missing x in MILP result");
        match x.value {
            MILPValue::Real(value) => {
                assert_precision(value, -8.0);
            }
            MILPValue::Bool(_) | MILPValue::Int(_) => {
                panic!("expected x to be real, got {}", x.value)
            }
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_apply_objective_offset_in_min() {
        let source = r#"
    min x + 10
    s.t.
        x >= 2
    define
        x as NonNegativeReal
    "#;
        let solution = solve(source).unwrap();
        assert_correct_named_real_solution(solution, 12.0, &[("x", 2.0)]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_sat_problem_with_logic() {
        //unique satisfying assignment: a=false, b=true
        let source = r#"
    solve
    s.t.
        a or b
        (not a) or (not b)
        not a
    define
        a, b as Boolean
    "#;
        let solution = solve_milp(source).unwrap();
        let assignment = solution.assignment();
        let a = match assignment.iter().find(|v| v.name == "a").unwrap().value {
            MILPValue::Bool(b) => b,
            _ => panic!("expected bool"),
        };
        let b = match assignment.iter().find(|v| v.name == "b").unwrap().value {
            MILPValue::Bool(b) => b,
            _ => panic!("expected bool"),
        };
        assert!(!a && b, "expected a=false b=true, got a={} b={}", a, b);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_max_sat_style_objective() {
        //(a and b) is forbidden, the best is (not a) plus (b or c) = 2
        let source = r#"
    max 2*(a and b) + (not a) + (b or c)
    s.t.
        not (a and b)
    define
        a, b, c as Boolean
    "#;
        let solution = solve_milp(source).unwrap();
        assert_precision(solution.value(), 2.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_implication_chain() {
        //x_1 is forced and implications propagate to all three variables
        let source = r#"
    min x_1 + x_2 + x_3
    s.t.
        x_1
        x_1 implies x_2
        x_2 implies x_3
    define
        x_1, x_2, x_3 as Boolean
    "#;
        let solution = solve_milp(source).unwrap();
        assert_precision(solution.value(), 3.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_xor_parity() {
        //a xor b is false with both true, so c must be true
        let source = r#"
    solve
    s.t.
        (a xor b) xor c
        a
        b
    define
        a, b, c as Boolean
    "#;
        let solution = solve_milp(source).unwrap();
        let c_val = solution
            .assignment()
            .iter()
            .find(|v| v.name == "c")
            .unwrap()
            .value;
        let c = match c_val {
            MILPValue::Bool(b) => b,
            _ => panic!("expected bool"),
        };
        assert!(c, "a xor b is false, so c must be true");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_iff_constraint() {
        //y iff x with x forced true means y must be true, minimization checks it
        let source = r#"
    min 10y + z
    s.t.
        x
        y iff x
        z or y
    define
        x, y, z as Boolean
    "#;
        let solution = solve_milp(source).unwrap();
        //y forced to 1 by the iff, z free to be 0 since y satisfies the or
        assert_precision(solution.value(), 10.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_mix_logic_and_arithmetic() {
        //the reified (a and b) is a 0/1 value usable inside arithmetic
        let source = r#"
    max 3*(a and b) + y
    s.t.
        y <= 1.5
        a iff b
    define
        a, b as Boolean
        y as NonNegativeReal
    "#;
        let result = solve_milp(source).unwrap();
        assert_precision(result.value(), 4.5);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_bare_logic_between_other_constraints() {
        //several bare logic constraints interleaved with comparisons,
        //written without parenthesis
        let source = r#"
    min x + y + z
    s.t.
        a or b
        x + y >= 3
        a implies b and not c
        z >= 2 * x
        not a or c
    define
        x, y, z as NonNegativeReal
        a, b, c as Boolean
    "#;
        let result = solve_milp(source).unwrap();
        //the logic part is satisfiable without forcing anything expensive,
        //the arithmetic part is minimized at x=0, y=3, z=0
        assert_precision(result.value(), 3.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_logic_blocks() {
        //all forces every x to true, checked by the minimization
        let source = r#"
    min sum(i in 0..3) { x_i }
    s.t.
        all(i in 0..3) { x_i }
    define
        x_i as Boolean for i in 0..3
    "#;
        let solution = solve_milp(source).unwrap();
        assert_precision(solution.value(), 3.0);
        //any forces at least one x to true
        let source = r#"
    min sum(i in 0..3) { x_i }
    s.t.
        any(i in 0..3) { x_i }
    define
        x_i as Boolean for i in 0..3
    "#;
        let solution = solve_milp(source).unwrap();
        assert_precision(solution.value(), 1.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_solve_independent_set_with_logic() {
        //maximum independent set on a triangle with a pendant vertex,
        //adjacent vertices cannot both be selected
        let source = r#"
    max sum(v in nodes(G)) { x_v }
    s.t.
        not x_u or not x_v for (u, v) in edges(G)
    where
        let G = Graph {
            A -> [B, C],
            B -> [A, C],
            C -> [A, B, D],
            D -> [C]
        }
    define
        x_v as Boolean for v in nodes(G)
    "#;
        let solution = solve_milp(source).unwrap();
        //the best independent set picks D plus one vertex of the triangle
        assert_precision(solution.value(), 2.0);
        let assignment = solution.assignment();
        let d_val = assignment.iter().find(|v| v.name == "x_D").unwrap().value;
        let d = match d_val {
            MILPValue::Bool(b) => b,
            _ => panic!("expected bool"),
        };
        assert!(
            d,
            "the pendant vertex D is in every maximum independent set"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_fold_constant_abs() {
        //an absolute value of constants is folded to a number, no
        //auxiliary variable is created
        let source = r#"
    min x + abs { 2 - 5 }
    s.t.
        x >= 2
    define
        x as NonNegativeReal
    "#;
        let solution = solve(source).unwrap();
        assert_correct_named_real_solution(solution, 5.0, &[("x", 2.0)]);
    }
}
