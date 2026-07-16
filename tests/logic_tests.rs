#[cfg(test)]
mod logic_tests {
    use indexmap::IndexMap;
    use rooc::RoocParser;
    use rooc::model_transformer::Exp;
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    fn transformed(source: &str) -> String {
        RoocParser::new(source.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("failed to transform")
            .to_string()
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_transform_logic_operators() {
        let s = transformed("solve\ns.t.\n    (a and b) or not c\ndefine\n    a, b, c as Boolean");
        assert!(s.contains("(a and b) or not c"), "got: {}", s);
        //without parenthesis the precedence gives the same shape
        let s = transformed("solve\ns.t.\n    a and b or not c\ndefine\n    a, b, c as Boolean");
        assert!(s.contains("(a and b) or not c"), "got: {}", s);
        let s = transformed(
            "solve\ns.t.\n    a implies (b iff c) >= 1\ndefine\n    a, b, c as Boolean",
        );
        assert!(s.contains("a implies (b iff c)"), "got: {}", s);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_parse_bare_logic_constraint() {
        let s = transformed("solve\ns.t.\n    a or b\n    not a\ndefine\n    a, b as Boolean");
        assert!(s.contains("a or b"), "got: {}", s);
        //bare constraints also support iteration
        let s = transformed(
            "solve\ns.t.\n    x_i or x_{i + 1} for i in 0..2\ndefine\n    x_i as Boolean for i in 0..3",
        );
        assert!(s.contains("x_0 or x_1"), "got: {}", s);
        assert!(s.contains("x_1 or x_2"), "got: {}", s);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_reject_bare_non_boolean_constraint() {
        let bad = "min x\ns.t.\n    x + 1\ndefine\n    x as NonNegativeReal";
        assert!(
            RoocParser::new(bad.to_string())
                .type_check(&vec![], &IndexMap::new())
                .is_err(),
            "a bare numeric constraint must be rejected"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_hide_true_rhs_in_display() {
        let source = "solve\ns.t.\n    a or b\n    b = true\ndefine\n    a, b as Boolean";
        let parsed = RoocParser::new(source.to_string())
            .parse()
            .expect("failed to parse");
        let display = parsed.to_string();
        assert!(display.contains("a or b"), "got: {}", display);
        assert!(display.contains("b = true"), "got: {}", display);
        assert!(parsed.constraints()[0].is_logic_assertion());
        assert!(!parsed.constraints()[1].is_logic_assertion());

        let model = RoocParser::new(source.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("failed to transform");
        assert!(model.constraints()[0].is_logic_assertion());
        assert!(!model.constraints()[1].is_logic_assertion());
        let transformed = model.to_string();
        assert!(transformed.contains("    a or b\n"), "got: {}", transformed);
        assert!(transformed.contains("b = 1"), "got: {}", transformed);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_parse_logic_blocks_and_aliases() {
        //scoped variants expand over the iteration
        let s = transformed(
            "solve\ns.t.\n    any(i in 0..3) { x_i }\ndefine\n    x_i as Boolean for i in 0..3",
        );
        assert!(s.contains("x_0 or x_1 or x_2"), "got: {}", s);
        let s = transformed(
            "solve\ns.t.\n    all(i in 0..2) { x_i }\ndefine\n    x_i as Boolean for i in 0..2",
        );
        assert!(s.contains("x_0 and x_1"), "got: {}", s);
        //block variants and their aliases
        let s = transformed(
            "solve\ns.t.\n    all{a, b}\n    conjunction{a, b}\n    any{a, b}\n    disjunction{a, b}\n    xor{a, b}\n    exclusive_disjunction{a, b}\ndefine\n    a, b as Boolean",
        );
        assert!(s.contains("a and b"), "got: {}", s);
        assert!(s.contains("a or b"), "got: {}", s);
        assert!(s.contains("a xor b"), "got: {}", s);
        //n-ary xor is the parity fold of binary xor
        let s = transformed("solve\ns.t.\n    xor{a, b, c}\ndefine\n    a, b, c as Boolean");
        assert!(s.contains("(a xor b) xor c"), "got: {}", s);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_type_check_logic_blocks() {
        //logic blocks require boolean bodies
        let bad =
            "solve\ns.t.\n    any(i in 0..3) { x_i + 1 }\ndefine\n    x_i as Boolean for i in 0..3";
        assert!(
            RoocParser::new(bad.to_string())
                .type_check(&vec![], &IndexMap::new())
                .is_err(),
            "logic block over numbers must be rejected"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_fold_constant_logic_in_simplify() {
        //absorbing constants
        let e = Exp::And(vec![Exp::Number(1.0), Exp::Number(0.0)]).simplify();
        assert!(matches!(e, Exp::Number(n) if n == 0.0), "got: {}", e);
        let e = Exp::Or(vec![Exp::Number(0.0), Exp::Number(1.0)]).simplify();
        assert!(matches!(e, Exp::Number(n) if n == 1.0), "got: {}", e);
        //identity constants are dropped
        let e = Exp::And(vec![Exp::Variable("x".to_string()), Exp::Number(1.0)]).simplify();
        assert!(matches!(&e, Exp::Variable(v) if v == "x"), "got: {}", e);
        let e = Exp::Or(vec![Exp::Variable("x".to_string()), Exp::Number(0.0)]).simplify();
        assert!(matches!(&e, Exp::Variable(v) if v == "x"), "got: {}", e);
        //not flips constants
        let e = Exp::Not(Exp::Number(0.0).to_box()).simplify();
        assert!(matches!(e, Exp::Number(n) if n == 1.0), "got: {}", e);
        //binary logic over constants folds
        let e = Exp::Implies(Exp::Number(1.0).to_box(), Exp::Number(0.0).to_box()).simplify();
        assert!(matches!(e, Exp::Number(n) if n == 0.0), "got: {}", e);
        let e = Exp::Iff(Exp::Number(0.0).to_box(), Exp::Number(0.0).to_box()).simplify();
        assert!(matches!(e, Exp::Number(n) if n == 1.0), "got: {}", e);
        let e = Exp::Xor(Exp::Number(1.0).to_box(), Exp::Number(0.0).to_box()).simplify();
        assert!(matches!(e, Exp::Number(n) if n == 1.0), "got: {}", e);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_flatten_nested_logic_in_simplify() {
        //and(and(a, b), c) becomes and(a, b, c)
        let e = Exp::And(vec![
            Exp::And(vec![
                Exp::Variable("a".to_string()),
                Exp::Variable("b".to_string()),
            ]),
            Exp::Variable("c".to_string()),
        ])
        .simplify();
        assert!(
            matches!(&e, Exp::And(exps) if exps.len() == 3),
            "got: {}",
            e
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_normalize_binop_logic_in_simplify() {
        //api-built BinOp logic expressions become the structural variants
        use rooc::BinOp;
        let e = Exp::BinOp(
            BinOp::And,
            Exp::Variable("a".to_string()).to_box(),
            Exp::Variable("b".to_string()).to_box(),
        )
        .simplify();
        assert!(
            matches!(&e, Exp::And(exps) if exps.len() == 2),
            "got: {}",
            e
        );
    }
}
