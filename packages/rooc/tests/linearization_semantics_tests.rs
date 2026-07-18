#[cfg(test)]
mod linearization_semantics_tests {
    use indexmap::IndexMap;
    use rooc::model_transformer::{Constraint, Exp, Model, Objective};
    use rooc::{
        BinOp, Comparison, LinearModel, LinearizationError, Linearizer, OptimizationType,
        RoocParser, SolverError, UnOp, VariableType, auto_solver, solve_milp_lp_problem,
    };

    fn transformed(source: &str) -> Model {
        RoocParser::new(source.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("failed to transform source model")
    }

    fn compile(source: &str) -> Result<LinearModel, LinearizationError> {
        Linearizer::linearize(transformed(source))
    }

    fn truthy(value: f64) -> bool {
        value != 0.0
    }

    fn logic_number(value: bool) -> f64 {
        if value { 1.0 } else { 0.0 }
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() <= 1e-7,
            "expected {expected}, got {actual}"
        );
    }

    fn eval_exp(exp: &Exp, assignment: &IndexMap<String, f64>) -> f64 {
        match exp {
            Exp::Number(value) => *value,
            Exp::Variable(name) => assignment[name],
            Exp::Abs(inner) => eval_exp(inner, assignment).abs(),
            Exp::Min(exps) => exps
                .iter()
                .map(|exp| eval_exp(exp, assignment))
                .reduce(f64::min)
                .expect("numeric min must be non-empty"),
            Exp::Max(exps) => exps
                .iter()
                .map(|exp| eval_exp(exp, assignment))
                .reduce(f64::max)
                .expect("numeric max must be non-empty"),
            Exp::And(exps) => {
                logic_number(exps.iter().all(|exp| truthy(eval_exp(exp, assignment))))
            }
            Exp::Or(exps) => logic_number(exps.iter().any(|exp| truthy(eval_exp(exp, assignment)))),
            Exp::Not(inner) => logic_number(!truthy(eval_exp(inner, assignment))),
            Exp::Xor(lhs, rhs) => {
                logic_number(truthy(eval_exp(lhs, assignment)) != truthy(eval_exp(rhs, assignment)))
            }
            Exp::Implies(lhs, rhs) => logic_number(
                !truthy(eval_exp(lhs, assignment)) || truthy(eval_exp(rhs, assignment)),
            ),
            Exp::Iff(lhs, rhs) => {
                logic_number(truthy(eval_exp(lhs, assignment)) == truthy(eval_exp(rhs, assignment)))
            }
            Exp::BinOp(op, lhs, rhs) => {
                let lhs = eval_exp(lhs, assignment);
                let rhs = eval_exp(rhs, assignment);
                match op {
                    BinOp::Add => lhs + rhs,
                    BinOp::Sub => lhs - rhs,
                    BinOp::Mul => lhs * rhs,
                    BinOp::Div => lhs / rhs,
                    BinOp::And => logic_number(truthy(lhs) && truthy(rhs)),
                    BinOp::Or => logic_number(truthy(lhs) || truthy(rhs)),
                    BinOp::Xor => logic_number(truthy(lhs) != truthy(rhs)),
                    BinOp::Implies => logic_number(!truthy(lhs) || truthy(rhs)),
                    BinOp::Iff => logic_number(truthy(lhs) == truthy(rhs)),
                }
            }
            Exp::UnOp(op, inner) => match op {
                UnOp::Neg => -eval_exp(inner, assignment),
                UnOp::Not => logic_number(!truthy(eval_exp(inner, assignment))),
            },
        }
    }

    fn eval_constraint(constraint: &Constraint, assignment: &IndexMap<String, f64>) -> bool {
        let lhs = eval_exp(constraint.lhs(), assignment);
        let rhs = eval_exp(constraint.rhs(), assignment);
        match constraint.constraint_type() {
            Comparison::LessOrEqual => lhs <= rhs + 1e-9,
            Comparison::GreaterOrEqual => lhs + 1e-9 >= rhs,
            Comparison::Equal => (lhs - rhs).abs() <= 1e-9,
            Comparison::Less => lhs < rhs,
            Comparison::Greater => lhs > rhs,
        }
    }

    fn fix_assignment(model: &mut LinearModel, assignment: &IndexMap<String, f64>) {
        for (name, value) in assignment {
            let index = model
                .variables()
                .iter()
                .position(|variable| variable == name)
                .unwrap_or_else(|| panic!("missing source variable {name}"));
            let mut coefficients = vec![0.0; model.variables().len()];
            coefficients[index] = 1.0;
            model.add_named_constraint(
                coefficients,
                Comparison::Equal,
                *value,
                &format!("__test_fix_{name}"),
            );
        }
    }

    fn feasible_with_assignment(model: &LinearModel, assignment: &IndexMap<String, f64>) -> bool {
        let mut fixed = model.clone();
        fix_assignment(&mut fixed, assignment);
        fixed.set_objective(
            vec![0.0; fixed.variables().len()],
            OptimizationType::Satisfy,
        );
        match solve_milp_lp_problem(&fixed) {
            Ok(_) => true,
            Err(SolverError::Infeasible) => false,
            Err(error) => panic!("unexpected solver error: {error}"),
        }
    }

    fn solve_fixed_objective(
        model: &LinearModel,
        assignment: &IndexMap<String, f64>,
        direction: OptimizationType,
    ) -> Result<f64, SolverError> {
        let mut fixed = model.clone();
        fix_assignment(&mut fixed, assignment);
        let objective = fixed.objective().clone();
        fixed.set_objective(objective, direction);
        solve_milp_lp_problem(&fixed).map(|solution| solution.value())
    }

    fn boolean_assignment(entries: &[(&str, bool)]) -> IndexMap<String, f64> {
        entries
            .iter()
            .map(|(name, value)| ((*name).to_string(), if *value { 1.0 } else { 0.0 }))
            .collect()
    }

    fn assert_boolean_projection(source: &str, variable_names: &[&str]) {
        let transformed = transformed(source);
        let linear = Linearizer::linearize(transformed.clone()).unwrap();
        for mask in 0..(1usize << variable_names.len()) {
            let assignment = variable_names
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    (
                        (*name).to_string(),
                        if mask & (1 << index) == 0 { 0.0 } else { 1.0 },
                    )
                })
                .collect::<IndexMap<_, _>>();
            let expected = transformed
                .constraints()
                .iter()
                .all(|constraint| eval_constraint(constraint, &assignment));
            let projected_assignment = assignment
                .iter()
                .filter(|(name, _)| linear.variables().contains(name))
                .map(|(name, value)| (name.clone(), *value))
                .collect();
            assert_eq!(
                feasible_with_assignment(&linear, &projected_assignment),
                expected,
                "source:\n{source}\nassignment={assignment:?}"
            );
        }
    }

    fn assert_projection_matches_source(source: &str, assignments: &[IndexMap<String, f64>]) {
        let transformed = transformed(source);
        let linear = Linearizer::linearize(transformed.clone()).unwrap();
        for assignment in assignments {
            let expected = transformed
                .constraints()
                .iter()
                .all(|constraint| eval_constraint(constraint, assignment));
            let projected_assignment = assignment
                .iter()
                .filter(|(name, _)| linear.variables().contains(name))
                .map(|(name, value)| (name.clone(), *value))
                .collect();
            assert_eq!(
                feasible_with_assignment(&linear, &projected_assignment),
                expected,
                "source:\n{source}\nassignment={assignment:?}"
            );
        }
    }

    fn assert_objective_representation(
        source: &str,
        assignments: &[IndexMap<String, f64>],
        exact: bool,
    ) {
        let transformed = transformed(source);
        let direction = transformed.objective().objective_type.clone();
        let linear = Linearizer::linearize(transformed.clone()).unwrap();
        let mut checked = 0;
        for assignment in assignments {
            if !transformed
                .constraints()
                .iter()
                .all(|constraint| eval_constraint(constraint, assignment))
            {
                continue;
            }
            checked += 1;
            let projected_assignment = assignment
                .iter()
                .filter(|(name, _)| linear.variables().contains(name))
                .map(|(name, value)| (name.clone(), *value))
                .collect();
            let expected = eval_exp(&transformed.objective().rhs, assignment);
            if exact {
                for solve_direction in [OptimizationType::Min, OptimizationType::Max] {
                    assert_close(
                        solve_fixed_objective(&linear, &projected_assignment, solve_direction)
                            .unwrap(),
                        expected,
                    );
                }
            } else {
                match &direction {
                    OptimizationType::Min | OptimizationType::Max => assert_close(
                        solve_fixed_objective(&linear, &projected_assignment, direction.clone())
                            .unwrap(),
                        expected,
                    ),
                    OptimizationType::Satisfy => {
                        panic!("objective representation requires min or max")
                    }
                }
            }
        }
        assert!(
            checked > 0,
            "no feasible assignments checked for:\n{source}"
        );
    }

    fn assert_compiled_objective(source: &str, assignment: &[(&str, f64)], expected: f64) {
        let assignment = assignment
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        let linear = compile(source).unwrap();
        for direction in [OptimizationType::Min, OptimizationType::Max] {
            assert_close(
                solve_fixed_objective(&linear, &assignment, direction).unwrap(),
                expected,
            );
        }
    }

    fn enumerate_integer_assignments(
        variables: &[(&str, std::ops::RangeInclusive<i32>)],
    ) -> Vec<IndexMap<String, f64>> {
        let mut assignments = vec![IndexMap::new()];
        for (name, values) in variables {
            let mut next = Vec::new();
            for assignment in assignments {
                for value in values.clone() {
                    let mut assignment = assignment.clone();
                    assignment.insert((*name).to_string(), value as f64);
                    next.push(assignment);
                }
            }
            assignments = next;
        }
        assignments
    }

    fn linear_model_satisfied(model: &LinearModel, values: &[f64]) -> bool {
        if values.len() != model.variables().len() {
            return false;
        }
        model.constraints().iter().all(|constraint| {
            let lhs = constraint
                .coefficients()
                .iter()
                .zip(values)
                .map(|(coefficient, value)| coefficient * value)
                .sum::<f64>();
            match constraint.constraint_type() {
                Comparison::LessOrEqual => lhs <= constraint.rhs() + 1e-9,
                Comparison::GreaterOrEqual => lhs + 1e-9 >= constraint.rhs(),
                Comparison::Equal => (lhs - constraint.rhs()).abs() <= 1e-9,
                Comparison::Less => lhs < constraint.rhs(),
                Comparison::Greater => lhs > constraint.rhs(),
            }
        })
    }

    #[test]
    fn variable_denominators_are_rejected_even_with_constant_numerator() {
        let result = compile("min 1 / x\ns.t.\n    x >= 1\ndefine\n    x as Real(1, 2)");
        assert!(matches!(
            result,
            Err(LinearizationError::NonLinearExpression(_))
        ));
    }

    #[test]
    fn zero_divisors_have_a_dedicated_error() {
        let result = compile("min 1 / 0\ns.t.\n");
        assert!(matches!(result, Err(LinearizationError::DivisionByZero(_))));
    }

    #[test]
    fn simplification_does_not_erase_a_variable_denominator() {
        let simplified = Exp::BinOp(
            BinOp::Div,
            Exp::Number(0.0).to_box(),
            Exp::Variable("x".to_string()).to_box(),
        )
        .simplify();
        assert!(matches!(simplified, Exp::BinOp(BinOp::Div, _, _)));
    }

    #[test]
    fn nested_division_simplification_does_not_compute_through_zero() {
        let simplified = Exp::BinOp(
            BinOp::Div,
            Exp::Number(1.0).to_box(),
            Exp::BinOp(
                BinOp::Div,
                Exp::Number(0.0).to_box(),
                Exp::Variable("x".to_string()).to_box(),
            )
            .to_box(),
        )
        .simplify();
        assert!(matches!(
            simplified,
            Exp::BinOp(BinOp::Div, lhs, rhs)
                if matches!(*lhs, Exp::Number(1.0))
                    && matches!(*rhs, Exp::BinOp(BinOp::Div, _, _))
        ));
    }

    #[test]
    fn simplification_preserves_right_nested_non_associative_operations() {
        let assignment = IndexMap::from([("x".to_string(), 2.0)]);
        for original in [
            Exp::BinOp(
                BinOp::Sub,
                Exp::Number(10.0).to_box(),
                Exp::BinOp(
                    BinOp::Sub,
                    Exp::Number(3.0).to_box(),
                    Exp::Variable("x".to_string()).to_box(),
                )
                .to_box(),
            ),
            Exp::BinOp(
                BinOp::Div,
                Exp::Number(12.0).to_box(),
                Exp::BinOp(
                    BinOp::Div,
                    Exp::Number(3.0).to_box(),
                    Exp::Variable("x".to_string()).to_box(),
                )
                .to_box(),
            ),
        ] {
            let simplified = original.simplify();
            assert_close(
                eval_exp(&simplified, &assignment),
                eval_exp(&original, &assignment),
            );
        }
    }

    #[test]
    fn empty_numeric_aggregations_are_rejected() {
        for (name, exp) in [("min", Exp::Min(vec![])), ("max", Exp::Max(vec![]))] {
            let model = Model::new(
                Objective::new(OptimizationType::Min, exp),
                vec![],
                IndexMap::new(),
            );
            let result = Linearizer::linearize(model);
            assert!(matches!(
                result,
                Err(LinearizationError::EmptyAggregation(kind)) if kind == name
            ));
        }
    }

    #[test]
    fn semantic_harness_evaluates_values_and_feasible_projections() {
        let source = "min 2 * x - 3\ns.t.\n    x >= 0\ndefine\n    x as IntegerRange(0, 2)";
        let model = transformed(source);
        let assignment = IndexMap::from([("x".to_string(), 1.0)]);
        assert_eq!(eval_exp(&model.objective().rhs, &assignment), -1.0);
        assert!(
            model
                .constraints()
                .iter()
                .all(|constraint| eval_constraint(constraint, &assignment))
        );
        let linear = compile(source).unwrap();
        assert_eq!(
            solve_fixed_objective(&linear, &assignment, OptimizationType::Min).unwrap(),
            -1.0
        );
        assert_eq!(
            solve_fixed_objective(&linear, &assignment, OptimizationType::Max).unwrap(),
            -1.0
        );

        let logic = compile("solve\ns.t.\n    a\ndefine\n    a as Boolean").unwrap();
        assert!(feasible_with_assignment(
            &logic,
            &IndexMap::from([("a".to_string(), 1.0)])
        ));
        assert!(!feasible_with_assignment(
            &logic,
            &IndexMap::from([("a".to_string(), 0.0)])
        ));
    }

    #[test]
    fn inferred_bounds_are_copied_to_the_linear_model_domain() {
        let linear = compile(
            "min x\ns.t.\n    y = x + 2\n    y <= 5\n    x >= -10\ndefine\n    x, y as Real",
        )
        .unwrap();
        assert_eq!(
            linear.domain()["x"].get_type(),
            &VariableType::Real(-10.0, 3.0)
        );
        assert_eq!(
            linear.domain()["y"].get_type(),
            &VariableType::Real(-8.0, 5.0)
        );
    }

    #[test]
    fn contradictory_constraints_stay_a_solver_infeasibility() {
        // an infeasible model is well formed: it must compile and come back
        // from the solver as infeasible, not fail at linearization
        let linear = compile("min x\ns.t.\n    x <= 0\n    x >= 1\ndefine\n    x as Real")
            .unwrap_or_else(|error| panic!("unexpected linearization error: {error:?}"));
        assert!(matches!(
            solve_milp_lp_problem(&linear),
            Err(SolverError::Infeasible)
        ));

        // same through integer rounding: no integer exists in [0.4, 0.6]
        let linear = compile(
            "min x\ns.t.\n    10 * x >= 4\n    10 * x <= 6\ndefine\n    x as IntegerRange(0, 10)",
        )
        .unwrap_or_else(|error| panic!("unexpected linearization error: {error:?}"));
        assert!(matches!(
            solve_milp_lp_problem(&linear),
            Err(SolverError::Infeasible)
        ));
    }

    #[test]
    fn abs_uses_the_correct_formulation_in_both_objective_directions() {
        let maximize =
            compile("max abs { x }\ns.t.\n    x >= -3\n    x <= 2\ndefine\n    x as Real").unwrap();
        assert!(
            maximize
                .variables()
                .iter()
                .any(|name| name.starts_with("$abs_") && name.ends_with("_positive"))
        );
        assert_close(auto_solver(&maximize).unwrap().value(), 3.0);

        let minimize_negative =
            compile("min -abs { x }\ns.t.\n    x >= -3\n    x <= 2\ndefine\n    x as Real")
                .unwrap();
        assert!(
            minimize_negative
                .variables()
                .iter()
                .any(|name| name.starts_with("$abs_") && name.ends_with("_positive"))
        );
        assert_close(auto_solver(&minimize_negative).unwrap().value(), -3.0);

        let maximize_negative =
            compile("max -abs { x }\ns.t.\n    x >= -3\n    x <= 2\ndefine\n    x as Real")
                .unwrap();
        assert!(
            maximize_negative
                .variables()
                .iter()
                .all(|name| !(name.starts_with("$abs_") && name.ends_with("_positive")))
        );
        assert_close(auto_solver(&maximize_negative).unwrap().value(), 0.0);
    }

    #[test]
    fn abs_constraint_direction_selects_one_sided_or_exact_lowering() {
        let upper = compile("solve\ns.t.\n    abs { x } <= 2\ndefine\n    x as Real").unwrap();
        assert!(
            upper
                .variables()
                .iter()
                .all(|name| !(name.starts_with("$abs_") && name.ends_with("_positive")))
        );

        for source in [
            "solve\ns.t.\n    abs { x } >= 2\ndefine\n    x as Real(-3, 3)",
            "solve\ns.t.\n    2 <= abs { x }\ndefine\n    x as Real(-3, 3)",
            "solve\ns.t.\n    abs { x } = 2\ndefine\n    x as Real(-3, 3)",
        ] {
            let exact = compile(source).unwrap();
            assert!(
                exact
                    .variables()
                    .iter()
                    .any(|name| name.starts_with("$abs_") && name.ends_with("_positive"))
            );
        }
    }

    #[test]
    fn exact_abs_reports_missing_finite_bounds() {
        let error = compile("max abs { x }\ns.t.\n    x = x\ndefine\n    x as Real").unwrap_err();
        assert!(matches!(
            &error,
            LinearizationError::MissingFiniteBounds {
                lower,
                upper,
                ..
            } if *lower == f64::NEG_INFINITY && *upper == f64::INFINITY
        ));
        let message = error.to_string();
        assert!(message.contains("x"));
        assert!(message.contains("Declare finite bounds"));
    }

    #[test]
    fn sign_known_abs_needs_no_auxiliary() {
        let linear =
            compile("max abs { x }\ns.t.\n    x >= 1\n    x <= 3\ndefine\n    x as Real").unwrap();
        assert!(
            linear
                .variables()
                .iter()
                .all(|name| !name.starts_with("$abs_"))
        );
        assert_close(auto_solver(&linear).unwrap().value(), 3.0);

        let negative =
            compile("max abs { x }\ns.t.\n    x >= -3\n    x <= -1\ndefine\n    x as Real")
                .unwrap();
        assert!(
            negative
                .variables()
                .iter()
                .all(|name| !name.starts_with("$abs_"))
        );
        assert_close(auto_solver(&negative).unwrap().value(), 3.0);
    }

    #[test]
    fn min_and_max_work_in_both_objective_directions() {
        let sources = [
            (
                "min max { x, y }\ns.t.\n    x = x\ndefine\n    x as Real(1, 3)\n    y as Real(2, 4)",
                2.0,
                false,
            ),
            (
                "max max { x, y }\ns.t.\n    x = x\ndefine\n    x as Real(1, 3)\n    y as Real(2, 4)",
                4.0,
                true,
            ),
            (
                "max min { x, y }\ns.t.\n    x = x\ndefine\n    x as Real(1, 3)\n    y as Real(2, 4)",
                3.0,
                false,
            ),
            (
                "min min { x, y }\ns.t.\n    x = x\ndefine\n    x as Real(1, 3)\n    y as Real(2, 4)",
                1.0,
                true,
            ),
        ];

        for (source, expected, has_selector) in sources {
            let linear = compile(source).unwrap();
            assert_close(auto_solver(&linear).unwrap().value(), expected);
            assert_eq!(
                linear
                    .variables()
                    .iter()
                    .any(|name| name.contains("_select_")),
                has_selector
            );
        }
    }

    #[test]
    fn min_max_requirements_follow_constraint_side_and_coefficient_sign() {
        for source in [
            "solve\ns.t.\n    max { x, y } >= 2\ndefine\n    x, y as Real(0, 3)",
            "solve\ns.t.\n    2 <= max { x, y }\ndefine\n    x, y as Real(0, 3)",
            "solve\ns.t.\n    min { x, y } <= 2\ndefine\n    x, y as Real(0, 3)",
            "min -max { x, y }\ns.t.\n    x = x\ndefine\n    x, y as Real(0, 3)",
            "max -min { x, y }\ns.t.\n    x = x\ndefine\n    x, y as Real(0, 3)",
        ] {
            let linear = compile(source).unwrap();
            assert!(
                linear
                    .variables()
                    .iter()
                    .any(|name| name.contains("_select_")),
                "expected exact selectors for:\n{source}"
            );
        }

        for source in [
            "solve\ns.t.\n    max { x, y } <= 2\ndefine\n    x, y as Real(0, 3)",
            "solve\ns.t.\n    2 >= max { x, y }\ndefine\n    x, y as Real(0, 3)",
            "solve\ns.t.\n    min { x, y } >= 2\ndefine\n    x, y as Real(0, 3)",
            "min max { x, y }\ns.t.\n    x = x\ndefine\n    x, y as Real(0, 3)",
            "max min { x, y }\ns.t.\n    x = x\ndefine\n    x, y as Real(0, 3)",
        ] {
            let linear = compile(source).unwrap();
            assert!(
                linear
                    .variables()
                    .iter()
                    .all(|name| !name.contains("_select_")),
                "did not expect exact selectors for:\n{source}"
            );
        }
    }

    #[test]
    fn dominated_extreme_operands_are_removed() {
        let max_model =
            compile("max max { x, 10 }\ns.t.\n    x = 0\ndefine\n    x as Real(0, 5)").unwrap();
        assert!(
            max_model
                .variables()
                .iter()
                .all(|name| !name.starts_with("$max_"))
        );
        assert_close(auto_solver(&max_model).unwrap().value(), 10.0);

        let min_model =
            compile("min min { x, -1 }\ns.t.\n    x = 0\ndefine\n    x as Real(0, 5)").unwrap();
        assert!(
            min_model
                .variables()
                .iter()
                .all(|name| !name.starts_with("$min_"))
        );
        assert_close(auto_solver(&min_model).unwrap().value(), -1.0);

        let equal_fixed =
            compile("max max { x, y }\ns.t.\n    x = 1\n    y = 1\ndefine\n    x, y as Real")
                .unwrap();
        assert!(
            equal_fixed
                .variables()
                .iter()
                .all(|name| !name.starts_with("$max_"))
        );
        assert_close(auto_solver(&equal_fixed).unwrap().value(), 1.0);
    }

    #[test]
    fn exact_extremes_report_missing_bounds() {
        assert!(matches!(
            compile("max max { x, 0 }\ns.t.\n    x = x\ndefine\n    x as Real").unwrap_err(),
            LinearizationError::MissingFiniteBounds { .. }
        ));
        assert!(matches!(
            compile("min min { x, 0 }\ns.t.\n    x = x\ndefine\n    x as Real").unwrap_err(),
            LinearizationError::MissingFiniteBounds { .. }
        ));
    }

    #[test]
    fn vertex_cover_logic_compiles_to_one_row_per_edge() {
        let linear = compile(
            r#"
min sum(v in nodes(G)) { x_v }
s.t.
    x_u or x_v for (u, v) in edges(G)
where
    let G = Graph {
        A -> [ B, C ],
        B -> [ D ],
        C -> [ D ],
        D -> [ E ],
        E
    }
define
    x_v as Boolean for v in nodes(G)
"#,
        )
        .unwrap();

        assert_eq!(linear.variables().len(), 5);
        assert_eq!(linear.constraints().len(), 5);
        assert!(
            linear
                .variables()
                .iter()
                .all(|name| !name.starts_with("$or_"))
        );
    }

    #[test]
    fn independent_set_compiles_to_exactly_one_row_per_edge() {
        let linear = compile(
            r#"
max sum(v in nodes(G)) { x_v }
s.t.
    !x_u or !x_v for (u, v) in edges(G)
where
    let G = Graph {
        A -> [ B, C ],
        B -> [ A, C ],
        C -> [ A, B, D ],
        D -> [ C ]
    }
define
    x_v as Boolean for v in nodes(G)
"#,
        )
        .unwrap();

        // eight directed edges, one asserted-or row each, and nothing else:
        // an extra aggregated row would change optima under other objectives
        assert_eq!(linear.constraints().len(), 8);
        for constraint in linear.constraints() {
            let nonzero = constraint
                .coefficients()
                .iter()
                .filter(|coefficient| **coefficient != 0.0)
                .count();
            assert_eq!(nonzero, 2, "each edge row involves exactly its endpoints");
        }
        let solution = solve_milp_lp_problem(&linear).unwrap();
        assert_close(solution.value(), 2.0);
    }

    #[test]
    fn equivalent_logic_comparisons_use_direct_assertions() {
        for source in [
            "solve\ns.t.\n    a or b\ndefine\n    a, b as Boolean",
            "solve\ns.t.\n    a or b = true\ndefine\n    a, b as Boolean",
            "solve\ns.t.\n    a or b >= 1\ndefine\n    a, b as Boolean",
            "solve\ns.t.\n    0 < a or b\ndefine\n    a, b as Boolean",
            "solve\ns.t.\n    a or b > 0\ndefine\n    a, b as Boolean",
            "solve\ns.t.\n    1 <= a or b\ndefine\n    a, b as Boolean",
            "solve\ns.t.\n    a or b < 1\ndefine\n    a, b as Boolean",
            "solve\ns.t.\n    1 > a or b\ndefine\n    a, b as Boolean",
        ] {
            let linear = compile(source).unwrap();
            assert_eq!(linear.constraints().len(), 1, "source:\n{source}");
            assert!(
                linear
                    .variables()
                    .iter()
                    .all(|name| !name.starts_with("$or_")),
                "source:\n{source}"
            );
        }
    }

    #[test]
    fn near_boolean_numbers_are_not_treated_as_boolean_literals() {
        let linear = compile("solve\ns.t.\n    a = 0.000009\ndefine\n    a as Boolean").unwrap();
        assert_eq!(linear.constraints().len(), 1);
        assert!(
            linear.constraints()[0]
                .coefficients()
                .iter()
                .all(|coefficient| *coefficient == 0.0)
        );
        assert_eq!(linear.constraints()[0].rhs(), 1.0);
        assert!(matches!(
            solve_milp_lp_problem(&linear),
            Err(SolverError::Infeasible)
        ));
    }

    #[test]
    fn generated_constraint_names_are_unique() {
        let linear = compile(
            "solve\ns.t.\n    logic: a or b\n    logic: a and b\ndefine\n    a, b as Boolean",
        )
        .unwrap();
        let names = linear
            .constraints()
            .iter()
            .map(|constraint| constraint.name())
            .collect::<Vec<_>>();
        let unique = names.iter().collect::<std::collections::HashSet<_>>();
        assert_eq!(names.len(), unique.len());
        assert!(names.iter().any(|name| name == "logic"));
        assert!(names.iter().any(|name| name == "logic__2"));
    }

    #[test]
    fn name_deduplication_avoids_user_written_names() {
        //the generated logic__2 would collide with the user's own logic__2,
        //so the second duplicate must skip to logic__3
        let linear = compile(
            "solve\ns.t.\n    logic: a or b\n    logic: a and b\n    logic__2: a or b\ndefine\n    a, b as Boolean",
        )
        .unwrap();
        let mut names = linear
            .constraints()
            .iter()
            .map(|constraint| constraint.name())
            .collect::<Vec<_>>();
        names.sort();
        assert_eq!(
            names,
            vec![
                "logic".to_string(),
                "logic__2".to_string(),
                "logic__3".to_string()
            ]
        );
    }

    #[test]
    fn compiled_output_round_trips_through_the_compiler() {
        //every intermediate representation must itself be a valid rooc model:
        //the displayed LinearModel is fed back through the whole compiler
        for source in [
            "max abs { x - y } + min { x, y }\ns.t.\n    cap: x + y <= 10\ndefine\n    x, y as NonNegativeReal(0, 8)",
            "min max { x, y }\ns.t.\n    lower: x + y >= 4\ndefine\n    x, y as NonNegativeReal(0, 9)",
            "max x_A + x_B + x_C\ns.t.\n    !x_u or !x_v for (u, v) in edges(G)\nwhere\n    let G = Graph { A -> [ B ], B -> [ A, C ], C -> [ B ] }\ndefine\n    x_v as Boolean for v in nodes(G)",
        ] {
            let first = compile(source).unwrap_or_else(|error| {
                panic!("failed to compile source: {error:?}\n{source}")
            });
            let expected = solve_milp_lp_problem(&first)
                .unwrap_or_else(|error| panic!("first solve failed: {error:?}"))
                .value();
            let refed_source = first.to_string();
            let refed = RoocParser::new(refed_source.clone())
                .parse_and_transform(vec![], &IndexMap::new())
                .unwrap_or_else(|error| {
                    panic!("compiled output failed to re-transform:\n{refed_source}\n{error}")
                });
            RoocParser::new(refed_source.clone())
                .type_check(&vec![], &IndexMap::new())
                .unwrap_or_else(|error| {
                    panic!("compiled output failed to type check:\n{refed_source}\n{error}")
                });
            let second = Linearizer::linearize(refed).unwrap_or_else(|error| {
                panic!("re-fed model failed to linearize:\n{refed_source}\n{error:?}")
            });
            let round = solve_milp_lp_problem(&second)
                .unwrap_or_else(|error| {
                    panic!("round trip solve failed:\n{refed_source}\n{error:?}")
                })
                .value();
            assert_close(round, expected);
        }
    }

    #[test]
    fn auto_generated_rows_are_unnamed() {
        let linear = compile(
            "max x + y\ns.t.\n    x + y <= 1\n    cap: x - y <= 1\ndefine\n    x, y as NonNegativeReal(0, 10)",
        )
        .unwrap_or_else(|error| panic!("unexpected linearization error: {error:?}"));
        let names = linear
            .constraints()
            .iter()
            .map(|constraint| constraint.name())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["".to_string(), "cap".to_string()]);
        let display = linear.to_string();
        assert!(!display.contains("_c1"), "auto name leaked:\n{display}");
        assert!(display.contains("cap: "));
    }

    #[test]
    fn helper_rows_from_reification_are_unnamed() {
        let linear = compile("solve\ns.t.\n    y = (a or b)\ndefine\n    a, b, y as Boolean")
            .unwrap_or_else(|error| panic!("unexpected linearization error: {error:?}"));
        assert!(linear.constraints().len() > 1);
        assert!(
            linear
                .constraints()
                .iter()
                .all(|constraint| constraint.name().is_empty()),
            "expected only unnamed rows, got: {:?}",
            linear
                .constraints()
                .iter()
                .map(|constraint| constraint.name())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn all_of_complex_formulas_lowers_without_root_reification() {
        let linear = compile(
            r#"
solve
s.t.
    all {
        a or b,
        c implies d,
        not (e xor f)
    }
define
    a, b, c, d, e, f as Boolean
"#,
        )
        .unwrap();
        assert_eq!(linear.constraints().len(), 3);
        assert!(
            linear
                .variables()
                .iter()
                .all(|name| !name.starts_with("$and_") && !name.starts_with("$or_"))
        );
    }

    #[test]
    fn any_of_conjunctions_uses_only_branch_witnesses() {
        let linear =
            compile("solve\ns.t.\n    any { a and b, c and d }\ndefine\n    a, b, c, d as Boolean")
                .unwrap();
        let witnesses = linear
            .variables()
            .iter()
            .filter(|name| name.starts_with("$logic_witness_"))
            .count();
        assert_eq!(witnesses, 2);
        assert_eq!(linear.constraints().len(), 5);
    }

    #[test]
    fn binary_logic_truth_tables_match_source_projection() {
        let cases = [
            ("a and b", (|a, b| a && b) as fn(bool, bool) -> bool),
            ("a or b", |a, b| a || b),
            ("a xor b", |a, b| a != b),
            ("a implies b", |a, b| !a || b),
            ("a iff b", |a, b| a == b),
        ];

        for (expression, oracle) in cases {
            for comparison in ["", " = true", " = false", " >= 1", " <= 0", " > 0", " < 1"] {
                let source = format!(
                    "solve\ns.t.\n    {}{}\ndefine\n    a, b as Boolean",
                    expression, comparison
                );
                let transformed = transformed(&source);
                let linear = Linearizer::linearize(transformed.clone()).unwrap();
                for a in [false, true] {
                    for b in [false, true] {
                        let assignment = boolean_assignment(&[("a", a), ("b", b)]);
                        let expected = transformed
                            .constraints()
                            .iter()
                            .all(|constraint| eval_constraint(constraint, &assignment));
                        let expects_true = comparison.is_empty()
                            || comparison == " = true"
                            || comparison == " >= 1"
                            || comparison == " > 0";
                        let expected_from_oracle = if expects_true {
                            oracle(a, b)
                        } else {
                            !oracle(a, b)
                        };
                        assert_eq!(expected, expected_from_oracle);
                        assert_eq!(
                            feasible_with_assignment(&linear, &assignment),
                            expected,
                            "expression={expression}, comparison={comparison}, a={a}, b={b}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn logic_projection_covers_not_nary_nested_constants_and_negated_literals() {
        for (source, variables) in [
            (
                "solve\ns.t.\n    not a\ndefine\n    a as Boolean",
                &["a"][..],
            ),
            (
                "solve\ns.t.\n    all { a, not b, c or d }\ndefine\n    a, b, c, d as Boolean",
                &["a", "b", "c", "d"][..],
            ),
            (
                "solve\ns.t.\n    any { a and b, not c }\ndefine\n    a, b, c as Boolean",
                &["a", "b", "c"][..],
            ),
            (
                "solve\ns.t.\n    xor { a, b, c }\ndefine\n    a, b, c as Boolean",
                &["a", "b", "c"][..],
            ),
            (
                "solve\ns.t.\n    (a or b) and (not c or d)\ndefine\n    a, b, c, d as Boolean",
                &["a", "b", "c", "d"][..],
            ),
            (
                "solve\ns.t.\n    all { a or false, not (b and false), true }\ndefine\n    a, b as Boolean",
                &["a", "b"][..],
            ),
            (
                "solve\ns.t.\n    not a or not b\ndefine\n    a, b as Boolean",
                &["a", "b"][..],
            ),
            (
                "solve\ns.t.\n    not (a and b)\ndefine\n    a, b as Boolean",
                &["a", "b"][..],
            ),
            (
                "solve\ns.t.\n    any { a or b, c and d }\ndefine\n    a, b, c, d as Boolean",
                &["a", "b", "c", "d"][..],
            ),
            (
                "solve\ns.t.\n    (a and b) implies (c or d)\ndefine\n    a, b, c, d as Boolean",
                &["a", "b", "c", "d"][..],
            ),
            (
                "solve\ns.t.\n    (a or b) iff (c and d)\ndefine\n    a, b, c, d as Boolean",
                &["a", "b", "c", "d"][..],
            ),
            (
                "solve\ns.t.\n    (a or b) xor (c and d)\ndefine\n    a, b, c, d as Boolean",
                &["a", "b", "c", "d"][..],
            ),
        ] {
            assert_boolean_projection(source, variables);
        }

        let tautology = compile("solve\ns.t.\n    a or true\ndefine\n    a as Boolean").unwrap();
        assert_eq!(tautology.constraints().len(), 0);
        let contradiction =
            compile("solve\ns.t.\n    a and false\ndefine\n    a as Boolean").unwrap();
        assert_eq!(contradiction.constraints().len(), 1);
    }

    #[test]
    fn logic_values_used_in_arithmetic_remain_exact() {
        let source =
            "max 3 * (a and b) - 2 * (a xor b)\ns.t.\n    a or not a\ndefine\n    a, b as Boolean";
        let transformed = transformed(source);
        let linear = Linearizer::linearize(transformed.clone()).unwrap();
        for a in [false, true] {
            for b in [false, true] {
                let assignment = boolean_assignment(&[("a", a), ("b", b)]);
                let expected = eval_exp(&transformed.objective().rhs, &assignment);
                assert_close(
                    solve_fixed_objective(&linear, &assignment, OptimizationType::Min).unwrap(),
                    expected,
                );
                assert_close(
                    solve_fixed_objective(&linear, &assignment, OptimizationType::Max).unwrap(),
                    expected,
                );
            }
        }
    }

    #[test]
    fn exhaustive_abs_objective_matrix() {
        let x_assignments = enumerate_integer_assignments(&[("x", -2..=2)]);
        for (source, exact) in [
            (
                "min abs { x } + 5\ns.t.\n    x = x\ndefine\n    x as Real(-2, 2)",
                false,
            ),
            (
                "max abs { x } - 1\ns.t.\n    x = x\ndefine\n    x as Real(-2, 2)",
                true,
            ),
            (
                "min -abs { x }\ns.t.\n    x = x\ndefine\n    x as Real(-2, 2)",
                true,
            ),
            (
                "max -abs { x }\ns.t.\n    x = x\ndefine\n    x as Real(-2, 2)",
                false,
            ),
            (
                "min -(-abs { x })\ns.t.\n    x = x\ndefine\n    x as Real(-2, 2)",
                false,
            ),
            (
                "min -2 * abs { x }\ns.t.\n    x = x\ndefine\n    x as Real(-2, 2)",
                true,
            ),
            (
                "max -2 * abs { x }\ns.t.\n    x = x\ndefine\n    x as Real(-2, 2)",
                false,
            ),
            (
                "max 3 + abs { x }\ns.t.\n    x = x\ndefine\n    x as Real(-2, 2)",
                true,
            ),
            (
                "max abs { x }\ns.t.\n    x >= -2\n    x <= 2\ndefine\n    x as Real",
                true,
            ),
        ] {
            assert_objective_representation(source, &x_assignments, exact);
        }

        let xy_assignments = enumerate_integer_assignments(&[("x", -2..=2), ("y", -2..=2)]);
        for source in [
            "min abs { x } - abs { y }\ns.t.\n    x = x\ndefine\n    x, y as Real(-2, 2)",
            "max abs { x } - abs { y }\ns.t.\n    x = x\ndefine\n    x, y as Real(-2, 2)",
        ] {
            let linear = compile(source).unwrap();
            assert_eq!(
                linear
                    .variables()
                    .iter()
                    .filter(|name| name.ends_with("_positive"))
                    .count(),
                1
            );
            assert_objective_representation(source, &xy_assignments, false);
        }

        assert_objective_representation(
            "max abs { max { x, y } }\ns.t.\n    x = x\ndefine\n    x, y as Real(-2, 2)",
            &xy_assignments,
            true,
        );
    }

    #[test]
    fn exhaustive_abs_constraint_matrix() {
        let x_assignments = enumerate_integer_assignments(&[("x", -2..=2)]);
        for source in [
            "solve\ns.t.\n    abs { x } <= 1\ndefine\n    x as Real(-2, 2)",
            "solve\ns.t.\n    1 >= abs { x }\ndefine\n    x as Real(-2, 2)",
            "solve\ns.t.\n    abs { x } >= 1\ndefine\n    x as Real(-2, 2)",
            "solve\ns.t.\n    1 <= abs { x }\ndefine\n    x as Real(-2, 2)",
            "solve\ns.t.\n    abs { x } = 1\ndefine\n    x as Real(-2, 2)",
            "solve\ns.t.\n    1 = abs { x }\ndefine\n    x as Real(-2, 2)",
            "solve\ns.t.\n    -abs { x } <= -1\ndefine\n    x as Real(-2, 2)",
            "solve\ns.t.\n    -abs { x } >= -1\ndefine\n    x as Real(-2, 2)",
        ] {
            assert_projection_matches_source(source, &x_assignments);
        }

        let xy_assignments = enumerate_integer_assignments(&[("x", -2..=2), ("y", -2..=2)]);
        for source in [
            "solve\ns.t.\n    abs { x } - abs { y } <= 0\ndefine\n    x, y as Real(-2, 2)",
            "solve\ns.t.\n    0 >= abs { x } - abs { y }\ndefine\n    x, y as Real(-2, 2)",
            "solve\ns.t.\n    abs { x } - abs { y } = 0\ndefine\n    x, y as Real(-2, 2)",
        ] {
            assert_projection_matches_source(source, &xy_assignments);
        }
    }

    #[test]
    fn exhaustive_abs_domains_and_sign_known_cases() {
        let boolean_assignments = enumerate_integer_assignments(&[("b", 0..=1)]);
        let boolean =
            compile("max abs { b }\ns.t.\n    b or not b\ndefine\n    b as Boolean").unwrap();
        assert!(
            boolean
                .variables()
                .iter()
                .all(|name| !name.starts_with("$abs_"))
        );
        assert_objective_representation(
            "max abs { b }\ns.t.\n    b or not b\ndefine\n    b as Boolean",
            &boolean_assignments,
            true,
        );

        let integer_assignments = enumerate_integer_assignments(&[("x", -2..=2)]);
        assert_objective_representation(
            "max abs { x }\ns.t.\n    x = x\ndefine\n    x as IntegerRange(-2, 2)",
            &integer_assignments,
            true,
        );

        for (source, assignments) in [
            (
                "max abs { x }\ns.t.\n    x = x\ndefine\n    x as Real(0, 2)",
                enumerate_integer_assignments(&[("x", 0..=2)]),
            ),
            (
                "max abs { x }\ns.t.\n    x = x\ndefine\n    x as Real(-2, 0)",
                enumerate_integer_assignments(&[("x", -2..=0)]),
            ),
        ] {
            let linear = compile(source).unwrap();
            assert!(
                linear
                    .variables()
                    .iter()
                    .all(|name| !name.starts_with("$abs_"))
            );
            assert_objective_representation(source, &assignments, true);
        }
    }

    #[test]
    fn exhaustive_min_max_objective_matrix() {
        let assignments = enumerate_integer_assignments(&[("x", -2..=2), ("y", -2..=2)]);
        for (source, exact) in [
            (
                "min max { x, y }\ns.t.\n    x = x\ndefine\n    x, y as Real(-2, 2)",
                false,
            ),
            (
                "max max { x, y }\ns.t.\n    x = x\ndefine\n    x, y as Real(-2, 2)",
                true,
            ),
            (
                "max min { x, y }\ns.t.\n    x = x\ndefine\n    x, y as Real(-2, 2)",
                false,
            ),
            (
                "min min { x, y }\ns.t.\n    x = x\ndefine\n    x, y as Real(-2, 2)",
                true,
            ),
            (
                "min -max { x, y }\ns.t.\n    x = x\ndefine\n    x, y as Real(-2, 2)",
                true,
            ),
            (
                "max -max { x, y }\ns.t.\n    x = x\ndefine\n    x, y as Real(-2, 2)",
                false,
            ),
            (
                "max -min { x, y }\ns.t.\n    x = x\ndefine\n    x, y as Real(-2, 2)",
                true,
            ),
            (
                "min -min { x, y }\ns.t.\n    x = x\ndefine\n    x, y as Real(-2, 2)",
                false,
            ),
            (
                "min -(-max { x, y })\ns.t.\n    x = x\ndefine\n    x, y as Real(-2, 2)",
                false,
            ),
            (
                "max 4 + max { x, y }\ns.t.\n    x = x\ndefine\n    x, y as Real(-2, 2)",
                true,
            ),
        ] {
            assert_objective_representation(source, &assignments, exact);
        }
    }

    #[test]
    fn exhaustive_min_max_constraint_matrix() {
        let assignments = enumerate_integer_assignments(&[("x", -2..=2), ("y", -2..=2)]);
        for source in [
            "solve\ns.t.\n    max { x, y } <= 1\ndefine\n    x, y as Real(-2, 2)",
            "solve\ns.t.\n    1 >= max { x, y }\ndefine\n    x, y as Real(-2, 2)",
            "solve\ns.t.\n    max { x, y } >= 1\ndefine\n    x, y as Real(-2, 2)",
            "solve\ns.t.\n    1 <= max { x, y }\ndefine\n    x, y as Real(-2, 2)",
            "solve\ns.t.\n    max { x, y } = 1\ndefine\n    x, y as Real(-2, 2)",
            "solve\ns.t.\n    min { x, y } >= -1\ndefine\n    x, y as Real(-2, 2)",
            "solve\ns.t.\n    -1 <= min { x, y }\ndefine\n    x, y as Real(-2, 2)",
            "solve\ns.t.\n    min { x, y } <= -1\ndefine\n    x, y as Real(-2, 2)",
            "solve\ns.t.\n    -1 >= min { x, y }\ndefine\n    x, y as Real(-2, 2)",
            "solve\ns.t.\n    min { x, y } = -1\ndefine\n    x, y as Real(-2, 2)",
            "solve\ns.t.\n    -max { x, y } <= -1\ndefine\n    x, y as Real(-2, 2)",
            "solve\ns.t.\n    -min { x, y } >= 1\ndefine\n    x, y as Real(-2, 2)",
        ] {
            assert_projection_matches_source(source, &assignments);
        }
    }

    #[test]
    fn exhaustive_min_max_nested_inferred_and_mixed_domains() {
        let xyz_assignments =
            enumerate_integer_assignments(&[("x", -2..=2), ("y", -2..=2), ("z", -2..=2)]);
        assert_objective_representation(
            "max min { max { x, y }, z }\ns.t.\n    x = x\ndefine\n    x, y, z as Real(-2, 2)",
            &xyz_assignments,
            false,
        );
        assert_objective_representation(
            "min min { max { x, y }, z }\ns.t.\n    x = x\ndefine\n    x, y, z as Real(-2, 2)",
            &xyz_assignments,
            true,
        );

        let xy_assignments = enumerate_integer_assignments(&[("x", -2..=2), ("y", -2..=2)]);
        assert_objective_representation(
            "max max { x, y }\ns.t.\n    max { x, y } <= 2\n    x >= -2\n    y >= -2\ndefine\n    x, y as Real",
            &xy_assignments,
            true,
        );
        assert_objective_representation(
            "min min { x, y }\ns.t.\n    min { x, y } >= -2\n    x <= 2\n    y <= 2\ndefine\n    x, y as Real",
            &xy_assignments,
            true,
        );
        assert_objective_representation(
            "max max { x, y }\ns.t.\n    x >= -2\n    x <= 2\n    y = x + 1\n    y <= 2\ndefine\n    x, y as Real",
            &xy_assignments,
            true,
        );

        let mixed_assignments = enumerate_integer_assignments(&[("i", -2..=2), ("x", -1..=2)]);
        assert_objective_representation(
            "max max { i, x }\ns.t.\n    i = i\ndefine\n    i as IntegerRange(-2, 2)\n    x as Real(-1, 2)",
            &mixed_assignments,
            true,
        );
    }

    #[test]
    fn boolean_auxiliary_extensions_match_source_projection() {
        for (source, source_variables) in [
            (
                "solve\ns.t.\n    any { a and b, c and d }\ndefine\n    a, b, c, d as Boolean",
                &["a", "b", "c", "d"][..],
            ),
            (
                "solve\ns.t.\n    (a or b) iff (c and d)\ndefine\n    a, b, c, d as Boolean",
                &["a", "b", "c", "d"][..],
            ),
            (
                "solve\ns.t.\n    not ((a and b) implies (c xor d))\ndefine\n    a, b, c, d as Boolean",
                &["a", "b", "c", "d"][..],
            ),
        ] {
            let transformed = transformed(source);
            let linear = Linearizer::linearize(transformed.clone()).unwrap();
            for variable in linear.domain().values() {
                match variable.get_type() {
                    VariableType::Boolean => {}
                    VariableType::IntegerRange(_, _)
                    | VariableType::NonNegativeReal(_, _)
                    | VariableType::Real(_, _) => {
                        panic!("logic enumeration expected only Boolean variables")
                    }
                }
            }
            for source_mask in 0..(1usize << source_variables.len()) {
                let source_assignment = source_variables
                    .iter()
                    .enumerate()
                    .map(|(index, name)| {
                        (
                            (*name).to_string(),
                            if source_mask & (1 << index) == 0 {
                                0.0
                            } else {
                                1.0
                            },
                        )
                    })
                    .collect::<IndexMap<_, _>>();
                let expected = transformed
                    .constraints()
                    .iter()
                    .all(|constraint| eval_constraint(constraint, &source_assignment));
                let has_extension = (0..(1usize << linear.variables().len())).any(|mask| {
                    let values = (0..linear.variables().len())
                        .map(|index| if mask & (1 << index) == 0 { 0.0 } else { 1.0 })
                        .collect::<Vec<_>>();
                    let source_values_match =
                        source_assignment.iter().all(|(name, expected_value)| {
                            let index = linear
                                .variables()
                                .iter()
                                .position(|variable| variable == name)
                                .expect("missing source Boolean variable");
                            values[index] == *expected_value
                        });
                    source_values_match && linear_model_satisfied(&linear, &values)
                });
                assert_eq!(
                    has_extension, expected,
                    "source:\n{source}\nassignment={source_assignment:?}"
                );
            }
        }
    }

    #[test]
    fn affine_linearization_regressions() {
        assert_compiled_objective(
            "min 2 * x + 3 * y - 4\ns.t.\n    x = x\ndefine\n    x, y as Real(0, 2)",
            &[("x", 1.0), ("y", 2.0)],
            4.0,
        );
        assert_compiled_objective(
            "min x * -2 + y / -4\ns.t.\n    x = x\ndefine\n    x, y as Real(0, 4)",
            &[("x", 1.0), ("y", 4.0)],
            -3.0,
        );
        assert_compiled_objective(
            "max -(x + 2) + 10\ns.t.\n    x = x\ndefine\n    x as Real(0, 4)",
            &[("x", 3.0)],
            5.0,
        );
        assert_compiled_objective(
            "min 3 * x + y * 4\ns.t.\n    x = x\ndefine\n    x, y as Real(0, 4)",
            &[("x", 2.0), ("y", 1.0)],
            10.0,
        );
        assert_compiled_objective(
            "min x / 2 + 7\ns.t.\n    x = x\ndefine\n    x as Real(0, 4)",
            &[("x", 4.0)],
            9.0,
        );

        assert!(matches!(
            compile("min x * y\ns.t.\n    x = x\ndefine\n    x, y as Real(0, 1)").unwrap_err(),
            LinearizationError::NonLinearExpression(_)
        ));
        assert!(matches!(
            compile("min 2 / x\ns.t.\n    x >= 1\ndefine\n    x as Real(1, 2)").unwrap_err(),
            LinearizationError::NonLinearExpression(_)
        ));
        assert!(matches!(
            compile("min x / 0\ns.t.\n    x = x\ndefine\n    x as Real(0, 1)").unwrap_err(),
            LinearizationError::DivisionByZero(_)
        ));
    }

    #[test]
    fn integer_bounds_survive_float_noise_in_propagation() {
        // 1.9 * x <= 1.9 admits x = 1 exactly; the propagated upper bound
        // 1.9 * (1 / 1.9) < 1.0 in f64 must not be floored down to 0.
        let linear =
            compile("max x\ns.t.\n    1.9 * x <= 1.9\ndefine\n    x as IntegerRange(0, 10)")
                .unwrap();
        assert!(feasible_with_assignment(
            &linear,
            &IndexMap::from([("x".to_string(), 1.0)])
        ));
        let solution = solve_milp_lp_problem(&linear).unwrap();
        assert_close(solution.value(), 1.0);
    }

    #[test]
    fn tiny_nonzero_coefficients_are_not_dropped() {
        // 0.000001 * x >= 1 is feasible with x = 1e6; a 1e-5 zero tolerance
        // on coefficients would zero the row and report the model infeasible.
        for source in [
            "min x / 1000000\ns.t.\n    0.000001 * x >= 1\ndefine\n    x as NonNegativeReal",
            "min x / 1000000\ns.t.\n    x * 0.000001 >= 1\ndefine\n    x as NonNegativeReal",
        ] {
            let linear = compile(source).unwrap();
            let solution = solve_milp_lp_problem(&linear)
                .unwrap_or_else(|error| panic!("{source}\nunexpected error: {error:?}"));
            assert_close(solution.value(), 1.0);
        }
    }

    #[test]
    fn division_by_small_nonzero_constant_is_allowed() {
        let linear = compile("min x / 0.000001\ns.t.\n    x >= 1\ndefine\n    x as Real(1, 2)")
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        let solution = solve_milp_lp_problem(&linear).unwrap();
        assert_close(solution.value(), 1_000_000.0);
    }
}
