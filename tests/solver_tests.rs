#[cfg(test)]
pub mod solver_tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    use indexmap::IndexMap;
    use rooc::common::LpSolution;
    use rooc::linear_integer_binary_solver::IntOrBoolValue;
    use rooc::pipe::{
        BinarySolverPipe, CompilerPipe, IntegerBinarySolverPipe, LinearModelPipe, MILPSolverPipe,
        ModelPipe, PreModelPipe, RealSolver, StandardLinearModelPipe, TableauPipe,
    };
    use rooc::pipe::{PipeContext, PipeRunner};
    use rooc::pipe::{PipeDataType, PipeError, PipeableData, StepByStepSimplexPipe};
    #[allow(unused_imports)]
    use rooc::simplex::{CanonicalTransformError, OptimalTableau, SimplexError};
    use rooc::{float_eq, float_ne};
    use rooc::{MILPValue, OptimalTableauWithSteps};

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
    #[allow(clippy::result_large_err)]
    fn solve_binary(source: &str) -> Result<LpSolution<bool>, PipeError> {
        let pipe_runner = PipeRunner::new(vec![
            Box::new(CompilerPipe::new()),
            Box::new(PreModelPipe::new()),
            Box::new(ModelPipe::new()),
            Box::new(LinearModelPipe::new()),
            Box::new(BinarySolverPipe::new()),
        ]);

        let result = pipe_runner.run(
            PipeableData::String(source.to_string()),
            &PipeContext::new(vec![], &IndexMap::new()),
        );
        match result {
            Ok(data) => {
                let last = data.last().unwrap();
                match last {
                    PipeableData::BinarySolution(data) => Ok(data.clone()),
                    _ => Err(PipeError::InvalidData {
                        expected: PipeDataType::BinarySolution,
                        got: last.get_type(),
                    }),
                }
            }
            Err((error, _context)) => Err(error),
        }
    }

    #[allow(unused)]
    #[allow(clippy::result_large_err)]
    fn solve_integer_binary(source: &str) -> Result<LpSolution<IntOrBoolValue>, PipeError> {
        let pipe_runner = PipeRunner::new(vec![
            Box::new(CompilerPipe::new()),
            Box::new(PreModelPipe::new()),
            Box::new(ModelPipe::new()),
            Box::new(LinearModelPipe::new()),
            Box::new(IntegerBinarySolverPipe::new()),
        ]);

        let result = pipe_runner.run(
            PipeableData::String(source.to_string()),
            &PipeContext::new(vec![], &IndexMap::new()),
        );
        match result {
            Ok(data) => {
                let last = data.last().unwrap();
                match last {
                    PipeableData::IntegerBinarySolution(data) => Ok(data.clone()),
                    _ => Err(PipeError::InvalidData {
                        expected: PipeDataType::IntegerBinarySolution,
                        got: last.get_type(),
                    }),
                }
            }
            Err((error, _context)) => Err(error),
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
                        expected: PipeDataType::IntegerBinarySolution,
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
    fn assert_variables_binary(variables: &Vec<bool>, expected: &Vec<bool>, lax_var_num: bool) {
        if variables.len() != expected.len() && !lax_var_num {
            panic!(
                "Different length, expected {:?} but got {:?}",
                expected, variables
            );
        }
        for (v, e) in variables.iter().zip(expected.iter()) {
            if *v != *e {
                panic!(
                    "{:?}!={:?} Expected  {:?} but got {:?}",
                    v, e, expected, variables
                );
            }
        }
    }

    #[allow(unused)]
    fn assert_variables_integer(
        variables: &[IntOrBoolValue],
        expected: &[IntOrBoolValue],
        lax_var_num: bool,
    ) {
        if variables.len() != expected.len() && !lax_var_num {
            panic!(
                "Different length, expected {:?} but got {:?}",
                expected, variables
            );
        }
        for (v, e) in variables.iter().zip(expected.iter()) {
            match (v, e) {
                (IntOrBoolValue::Bool(v), IntOrBoolValue::Bool(e)) => {
                    if *v != *e {
                        panic!(
                            "{:?}!={:?} Expected  {:?} but got {:?}",
                            v, e, expected, variables
                        );
                    }
                }
                (IntOrBoolValue::Int(v), IntOrBoolValue::Int(e)) => {
                    if *v != *e {
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
        assert_correct_solution(solution, 21.0, vec![vec![9.0, 6.0, 14.0, 0.0, 0.0, 6.0]]);
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
        assert_correct_solution(
            solution,
            107.0,
            vec![vec![8.0, 0.0, 9.0, 11.0, 0.0, 0.0, 0.0]],
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
        assert_correct_solution(solution, 12.0, vec![vec![4.0, 4.0, 6.0, 2.0, 0.0, 0.0]]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[ignore] //TODO normal simplex doesn't correctly calculate optimal value, 10 is missing
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
            94.0,
            vec![
                vec![0.0, 8.0, 0.0, 10.0, 0.0, 4.0, 0.0, 0.0],
                vec![4.34136, 8.0, 1.68587, 10.0],
            ],
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
        assert_correct_solution(
            solution,
            18.0,
            vec![
                vec![22.0 / 3.0, 10.0 / 3.0, 0.0, 8.0, 0.0],
                vec![6.03716, 5.92566],
            ],
        );
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
    // Number of nutrients
    let N = len(Nmax)
define
    //bound the amount of each serving of food i
    x_i as NonNegativeReal(Fmin[i], Fmax[i]) for i in 0..N  

    "#;
        let solution = solve(source).unwrap();
        assert_correct_solution(
            solution,
            6.04444,
            vec![vec![
                1.3259259259259264,
                4.111111111111111,
                0.9999999999999998,
                0.0,
                0.0,
                26.62962962962963,
                100.0,
                100.0,
                43.37037037037037,
                0.32592592592592595,
                3.6740740740740745,
                3.111111111111111,
                0.8888888888888888,
                0.0,
                4.0,
            ]],
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
        let solution = solve_binary(source).unwrap();
        assert_precision(solution.value(), 280.0);
        assert_variables_binary(
            &solution.assignment_values(),
            &vec![false, false, true, false, true, true, true, true],
            true,
        );
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
        let solution = solve_integer_binary(source).unwrap();
        assert_precision(solution.value(), 21.0);
        let assignment = solution.assignment_values();
        assert_variables_integer(
            &assignment,
            &[IntOrBoolValue::Int(0), IntOrBoolValue::Int(7)],
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
    #[should_panic]
    fn should_detect_invalid_domain_2() {
        let source = r#"
    max 2x_1 + 3x_2
    s.t.
        x_1 + x_2 <= 7
        2x_1 + 3x_2 <= 21
    define
        x_1 as Real
        x_2 as IntegerRange(0, 10)
    "#; //here only IntegerRange and boolean are allowed
        solve_integer_binary(source).unwrap();
    }
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic]
    fn should_detect_invalid_domain_3() {
        let source = r#"
    max 2x_1 + 3x_2
    s.t.
        x_1 + x_2 <= 7
        2x_1 + 3x_2 <= 21
    define
        x_1 as Boolean
        x_2 as IntegerRange(0, 10)
    "#; //here only Boolean is allowed
        solve_binary(source).unwrap();
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
        let result = solve_integer_binary(source).unwrap();
        assert_precision(result.value(), 10.0);
        let assignment = result.assignment_values();
        assert_variables_integer(
            &assignment,
            &[
                IntOrBoolValue::Int(1),
                IntOrBoolValue::Int(2),
                IntOrBoolValue::Int(3),
                IntOrBoolValue::Int(4),
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
}
