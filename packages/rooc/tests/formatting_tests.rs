#[cfg(test)]
mod formatting_tests {
    //! Every printed form of a model has to parse back into the same model:
    //! the formatted source, which the editor writes over the user's code,
    //! and the compiled model, shown to inspect what the compiler produced.
    use indexmap::IndexMap;
    use rooc::model_transformer::{Exp, Model};
    use rooc::{BinOp, InputSpan, Linearizer, PreExp, Primitive, RoocParser, Spanned, UnOp};
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    /// An expression reduced to its meaning: chains of an associative
    /// operator are flattened, so `a + (b - c)` and `a + b - c` agree, while
    /// `a - (b - c)` and `a - b - c` do not.
    #[derive(Debug, Clone, PartialEq)]
    enum Canon {
        Num(f64),
        Bool(bool),
        Var(String),
        /// The terms of a sum, `false` for the subtracted ones.
        Sum(Vec<(bool, Canon)>),
        /// The factors of a product, `false` for the divisors.
        Prod(Vec<(bool, Canon)>),
        /// A chain of `and`, `or` or `xor`.
        Chain(BinOp, Vec<Canon>),
        Implies(Box<Canon>, Box<Canon>),
        Iff(Box<Canon>, Box<Canon>),
        Neg(Box<Canon>),
        Not(Box<Canon>),
        Block(String, Vec<Canon>),
    }

    fn terms(exp: Canon) -> Vec<(bool, Canon)> {
        match exp {
            Canon::Sum(terms) => terms,
            exp => vec![(true, exp)],
        }
    }

    fn factors(exp: Canon) -> Vec<(bool, Canon)> {
        match exp {
            Canon::Prod(factors) => factors,
            exp => vec![(true, exp)],
        }
    }

    fn flip(items: Vec<(bool, Canon)>) -> Vec<(bool, Canon)> {
        items.into_iter().map(|(sign, exp)| (!sign, exp)).collect()
    }

    fn binary(op: BinOp, lhs: Canon, rhs: Canon) -> Canon {
        match op {
            BinOp::Add => Canon::Sum([terms(lhs), terms(rhs)].concat()),
            BinOp::Sub => Canon::Sum([terms(lhs), flip(terms(rhs))].concat()),
            BinOp::Mul => Canon::Prod([factors(lhs), factors(rhs)].concat()),
            BinOp::Div => Canon::Prod([factors(lhs), flip(factors(rhs))].concat()),
            BinOp::And | BinOp::Or | BinOp::Xor => {
                let mut items = Vec::new();
                for exp in [lhs, rhs] {
                    match exp {
                        Canon::Chain(inner, chained) if inner == op => items.extend(chained),
                        exp => items.push(exp),
                    }
                }
                Canon::Chain(op, items)
            }
            BinOp::Implies => Canon::Implies(Box::new(lhs), Box::new(rhs)),
            BinOp::Iff => Canon::Iff(Box::new(lhs), Box::new(rhs)),
        }
    }

    fn neg(exp: Canon) -> Canon {
        match exp {
            //a negative literal is parsed as the negation of a number
            Canon::Num(value) => Canon::Num(-value),
            exp => Canon::Neg(Box::new(exp)),
        }
    }

    fn canon_pre(exp: &PreExp) -> Canon {
        match exp {
            PreExp::BinaryOperation(op, lhs, rhs) => binary(**op, canon_pre(lhs), canon_pre(rhs)),
            PreExp::UnaryOperation(op, exp) => match **op {
                UnOp::Neg => neg(canon_pre(exp)),
                UnOp::Not => Canon::Not(Box::new(canon_pre(exp))),
            },
            PreExp::Primitive(p) => match p.value() {
                Primitive::Number(value) => Canon::Num(*value),
                Primitive::Integer(value) => Canon::Num(*value as f64),
                Primitive::PositiveInteger(value) => Canon::Num(*value as f64),
                Primitive::Boolean(value) => Canon::Bool(*value),
                other => panic!("unexpected literal {other}"),
            },
            PreExp::Variable(name) => Canon::Var(name.value().clone()),
            PreExp::BlockFunction(f) => {
                Canon::Block(f.kind.to_string(), f.exps.iter().map(canon_pre).collect())
            }
            other => panic!("unexpected expression {other}"),
        }
    }

    fn canon_chain(op: BinOp, exps: &[Exp], identity: bool) -> Canon {
        exps.iter()
            .map(canon_exp)
            .reduce(|lhs, rhs| binary(op, lhs, rhs))
            .unwrap_or(Canon::Bool(identity))
    }

    fn canon_exp(exp: &Exp) -> Canon {
        match exp {
            Exp::Number(value) => Canon::Num(*value),
            Exp::Variable(name) => Canon::Var(name.clone()),
            Exp::BinOp(op, lhs, rhs) => binary(*op, canon_exp(lhs), canon_exp(rhs)),
            Exp::Xor(lhs, rhs) => binary(BinOp::Xor, canon_exp(lhs), canon_exp(rhs)),
            Exp::Implies(lhs, rhs) => binary(BinOp::Implies, canon_exp(lhs), canon_exp(rhs)),
            Exp::Iff(lhs, rhs) => binary(BinOp::Iff, canon_exp(lhs), canon_exp(rhs)),
            Exp::And(exps) => canon_chain(BinOp::And, exps, true),
            Exp::Or(exps) => canon_chain(BinOp::Or, exps, false),
            Exp::Not(exp) | Exp::UnOp(UnOp::Not, exp) => Canon::Not(Box::new(canon_exp(exp))),
            Exp::UnOp(UnOp::Neg, exp) => neg(canon_exp(exp)),
            Exp::Abs(exp) => Canon::Block("abs".to_string(), vec![canon_exp(exp)]),
            Exp::Min(exps) => Canon::Block("min".to_string(), exps.iter().map(canon_exp).collect()),
            Exp::Max(exps) => Canon::Block("max".to_string(), exps.iter().map(canon_exp).collect()),
        }
    }

    /// Parses an expression on its own, as the objective of a model.
    fn parse_exp(exp: &str) -> Result<PreExp, String> {
        RoocParser::new(format!("min {exp}\ns.t.\n    1 >= 1"))
            .parse()
            .map(|model| model.objective().rhs.clone())
            .map_err(|e| format!("{e:?}"))
    }

    fn span() -> InputSpan {
        InputSpan::default()
    }

    fn pre_var(index: u8) -> PreExp {
        PreExp::Variable(Spanned::new(((b'a' + index) as char).to_string(), span()))
    }

    /// Every expression with exactly `ops` operators, whose leaves are made by
    /// `leaf` in order starting from `first`, paired with the next leaf index.
    fn trees<T: Clone>(
        ops: usize,
        first: u8,
        leaf: &dyn Fn(u8) -> T,
        unary: &[&dyn Fn(T) -> T],
        binary: &[&dyn Fn(T, T) -> T],
    ) -> Vec<(T, u8)> {
        if ops == 0 {
            return vec![(leaf(first), first + 1)];
        }
        let mut result = Vec::new();
        for make in unary {
            for (exp, next) in trees(ops - 1, first, leaf, unary, binary) {
                result.push((make(exp), next));
            }
        }
        for make in binary {
            for lhs_ops in 0..ops {
                for (lhs, next) in trees(lhs_ops, first, leaf, unary, binary) {
                    for (rhs, last) in trees(ops - 1 - lhs_ops, next, leaf, unary, binary) {
                        result.push((make(lhs.clone(), rhs), last));
                    }
                }
            }
        }
        result
    }

    fn pre_unary(op: UnOp) -> impl Fn(PreExp) -> PreExp {
        move |exp| PreExp::UnaryOperation(Spanned::new(op, span()), exp.to_boxed())
    }

    fn pre_binary(op: BinOp) -> impl Fn(PreExp, PreExp) -> PreExp {
        move |lhs, rhs| {
            PreExp::BinaryOperation(Spanned::new(op, span()), lhs.to_boxed(), rhs.to_boxed())
        }
    }

    const ARITHMETIC: [BinOp; 4] = [BinOp::Add, BinOp::Sub, BinOp::Mul, BinOp::Div];
    const ALL_BINARY: [BinOp; 9] = [
        BinOp::Add,
        BinOp::Sub,
        BinOp::Mul,
        BinOp::Div,
        BinOp::And,
        BinOp::Or,
        BinOp::Xor,
        BinOp::Implies,
        BinOp::Iff,
    ];

    /// Checks that `printed` parses back into `expected` and is a fixed point
    /// of the formatter.
    fn assert_round_trip(printed: &str, expected: &Canon) {
        let parsed = parse_exp(printed)
            .unwrap_or_else(|e| panic!("printed expression does not parse: {printed}\n{e}"));
        assert_eq!(
            &canon_pre(&parsed),
            expected,
            "printing changed the meaning: {printed}"
        );
        assert_eq!(parsed.to_string(), printed, "formatting is not stable");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn formatted_expressions_keep_their_meaning() {
        let unary = [pre_unary(UnOp::Neg), pre_unary(UnOp::Not)];
        let unary = unary
            .iter()
            .map(|f| f as &dyn Fn(PreExp) -> PreExp)
            .collect::<Vec<_>>();
        let binary = ALL_BINARY.map(pre_binary);
        let binary = binary
            .iter()
            .map(|f| f as &dyn Fn(PreExp, PreExp) -> PreExp)
            .collect::<Vec<_>>();
        for ops in 0..=3 {
            for (tree, _) in trees(ops, 0, &pre_var, &unary, &binary) {
                assert_round_trip(&tree.to_string(), &canon_pre(&tree));
            }
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn negative_literals_are_formatted_as_negations() {
        //the parser never produces negative literals, but constants and
        //compiled expressions hold them
        let literal = |index: u8| {
            PreExp::Primitive(Spanned::new(
                Primitive::Integer(-(index as i64) - 1),
                span(),
            ))
        };
        let unary = [pre_unary(UnOp::Neg)];
        let unary = unary
            .iter()
            .map(|f| f as &dyn Fn(PreExp) -> PreExp)
            .collect::<Vec<_>>();
        let binary = ARITHMETIC.map(pre_binary);
        let binary = binary
            .iter()
            .map(|f| f as &dyn Fn(PreExp, PreExp) -> PreExp)
            .collect::<Vec<_>>();
        for ops in 0..=2 {
            for (tree, _) in trees(ops, 0, &literal, &unary, &binary) {
                assert_round_trip(&tree.to_string(), &canon_pre(&tree));
            }
        }
    }

    /// The positions of every pair of matching parentheses in `s`.
    fn paren_pairs(s: &str) -> Vec<(usize, usize)> {
        let mut open = Vec::new();
        let mut pairs = Vec::new();
        for (i, c) in s.char_indices() {
            match c {
                '(' => open.push(i),
                ')' => pairs.push((open.pop().expect("balanced parentheses"), i)),
                _ => {}
            }
        }
        pairs
    }

    /// Checks that removing any pair of parentheses from `printed` either
    /// breaks it or changes its meaning.
    fn assert_no_redundant_parentheses(printed: &str, expected: &Canon) {
        for (open, close) in paren_pairs(printed) {
            let without = format!(
                "{}{}{}",
                &printed[..open],
                &printed[open + 1..close],
                &printed[close + 1..]
            );
            if let Ok(parsed) = parse_exp(&without) {
                assert_ne!(
                    &canon_pre(&parsed),
                    expected,
                    "redundant parentheses in {printed}, {without} means the same"
                );
            }
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn arithmetic_is_formatted_without_redundant_parentheses() {
        let unary = [pre_unary(UnOp::Neg)];
        let unary = unary
            .iter()
            .map(|f| f as &dyn Fn(PreExp) -> PreExp)
            .collect::<Vec<_>>();
        let binary = ARITHMETIC.map(pre_binary);
        let binary = binary
            .iter()
            .map(|f| f as &dyn Fn(PreExp, PreExp) -> PreExp)
            .collect::<Vec<_>>();
        for ops in 0..=3 {
            for (tree, _) in trees(ops, 0, &pre_var, &unary, &binary) {
                assert_no_redundant_parentheses(&tree.to_string(), &canon_pre(&tree));
            }
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn compiled_arithmetic_has_no_redundant_parentheses() {
        let unary: [&dyn Fn(Exp) -> Exp; 1] = [&|exp: Exp| Exp::UnOp(UnOp::Neg, exp.to_box())];
        let binary = ARITHMETIC
            .map(|op| move |lhs: Exp, rhs: Exp| Exp::BinOp(op, lhs.to_box(), rhs.to_box()));
        let binary = binary
            .iter()
            .map(|f| f as &dyn Fn(Exp, Exp) -> Exp)
            .collect::<Vec<_>>();
        for ops in 0..=3 {
            for (tree, _) in trees(ops, 0, &exp_var, &unary, &binary) {
                assert_no_redundant_parentheses(&tree.to_string(), &canon_exp(&tree));
            }
        }
    }

    fn exp_var(index: u8) -> Exp {
        Exp::Variable(((b'a' + index) as char).to_string())
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn compiled_expressions_keep_their_meaning() {
        let unary: [&dyn Fn(Exp) -> Exp; 2] = [
            &|exp: Exp| Exp::UnOp(UnOp::Neg, exp.to_box()),
            &|exp: Exp| Exp::Not(exp.to_box()),
        ];
        let arithmetic = ARITHMETIC
            .map(|op| move |lhs: Exp, rhs: Exp| Exp::BinOp(op, lhs.to_box(), rhs.to_box()));
        let mut binary = arithmetic
            .iter()
            .map(|f| f as &dyn Fn(Exp, Exp) -> Exp)
            .collect::<Vec<_>>();
        let logic: [&dyn Fn(Exp, Exp) -> Exp; 5] = [
            &|lhs, rhs| Exp::And(vec![lhs, rhs]),
            &|lhs, rhs| Exp::Or(vec![lhs, rhs]),
            &|lhs: Exp, rhs: Exp| Exp::Xor(lhs.to_box(), rhs.to_box()),
            &|lhs: Exp, rhs: Exp| Exp::Implies(lhs.to_box(), rhs.to_box()),
            &|lhs: Exp, rhs: Exp| Exp::Iff(lhs.to_box(), rhs.to_box()),
        ];
        binary.extend(logic);
        let negative = |index: u8| Exp::Number(-(index as f64) - 1.5);
        for (leaf, max_ops) in [(&exp_var as &dyn Fn(u8) -> Exp, 3), (&negative, 2)] {
            for ops in 0..=max_ops {
                for (tree, _) in trees(ops, 0, leaf, &unary, &binary) {
                    let printed = tree.to_string();
                    let parsed = parse_exp(&printed).unwrap_or_else(|e| {
                        panic!("compiled expression does not parse: {printed}\n{e}")
                    });
                    assert_eq!(
                        canon_pre(&parsed),
                        canon_exp(&tree),
                        "printing changed the meaning: {printed}"
                    );
                }
            }
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn compiled_logic_chains_of_any_length_keep_their_meaning() {
        let (a, b, c) = (exp_var(0), exp_var(1), exp_var(2));
        let cases = [
            Exp::And(vec![]),
            Exp::Or(vec![]),
            Exp::And(vec![a.clone()]),
            Exp::Or(vec![a.clone(), b.clone(), c.clone()]),
            Exp::And(vec![a.clone(), Exp::Or(vec![b.clone(), c.clone()])]),
            Exp::And(vec![Exp::And(vec![a.clone(), b.clone()]), c.clone()]),
            Exp::BinOp(
                BinOp::Add,
                a.clone().to_box(),
                Exp::And(vec![b.clone()]).to_box(),
            ),
            Exp::BinOp(
                BinOp::Mul,
                a.clone().to_box(),
                Exp::Or(vec![b.clone(), c.clone()]).to_box(),
            ),
            Exp::BinOp(BinOp::And, a.clone().to_box(), Exp::Or(vec![b, c]).to_box()),
            Exp::Not(Exp::And(vec![]).to_box()),
        ];
        for tree in cases {
            let printed = tree.to_string();
            let parsed = parse_exp(&printed)
                .unwrap_or_else(|e| panic!("compiled expression does not parse: {printed}\n{e}"));
            assert_eq!(
                canon_pre(&parsed),
                canon_exp(&tree),
                "printing changed the meaning: {printed}"
            );
        }
    }

    const DEFINE: &str = "define\n    x, y, z, w as Real\n    a, b, c, d as Boolean";

    /// The first line of the formatted model, which holds the objective.
    fn formatted_objective(exp: &str) -> String {
        let source = format!("min {exp}\ns.t.\n    x >= 0\n{DEFINE}");
        let formatted = RoocParser::new(source).format().expect("failed to format");
        formatted.lines().next().unwrap().to_string()
    }

    fn compile(source: &str) -> Model {
        RoocParser::new(source.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .unwrap_or_else(|e| panic!("failed to compile:\n{source}\n{e}"))
    }

    /// The first line of the compiled model, which holds the objective.
    fn compiled_objective(exp: &str) -> String {
        let model = compile(&format!("min {exp}\ns.t.\n    x >= 0\n{DEFINE}"));
        model.to_string().lines().next().unwrap().to_string()
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn formatter_keeps_only_meaningful_parentheses() {
        for (source, expected) in [
            ("x - (y - z)", "x - (y - z)"),
            ("x - (y + z)", "x - (y + z)"),
            ("x + (y - z)", "x + y - z"),
            ("x + (y + z)", "x + y + z"),
            ("(x - y) - z", "x - y - z"),
            ("x / (y * z)", "x / (y * z)"),
            ("x / (y / z)", "x / (y / z)"),
            ("x * (y / z)", "x * y / z"),
            ("x / 2y", "x / (2 * y)"),
            ("x - 2(y - z)", "x - 2 * (y - z)"),
            ("(x + y) * z", "(x + y) * z"),
            ("-(x + y)", "-(x + y)"),
            ("-(-x)", "-(-x)"),
            ("x - -y", "x - -y"),
            ("-x * y", "-x * y"),
            ("x + (a and b)", "x + (a and b)"),
            ("2.0 * x", "2.0 * x"),
        ] {
            assert_eq!(formatted_objective(source), format!("min {expected}"));
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn formatter_groups_mixed_logic_operators() {
        for (source, expected) in [
            ("a and b or c", "(a and b) or c"),
            ("a or b and c", "a or (b and c)"),
            ("a and (b and c)", "a and b and c"),
            ("a xor (b xor c)", "a xor b xor c"),
            ("a implies b implies c", "a implies (b implies c)"),
            ("(a implies b) implies c", "(a implies b) implies c"),
            ("a iff b iff c", "(a iff b) iff c"),
            ("a iff (b implies c)", "a iff (b implies c)"),
            ("(a implies b) iff c", "(a implies b) iff c"),
            ("not (a and b)", "not (a and b)"),
            ("not a and b", "not a and b"),
        ] {
            assert_eq!(formatted_objective(source), format!("min {expected}"));
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn compiled_model_keeps_only_meaningful_parentheses() {
        for (source, expected) in [
            ("x - (y - z)", "x - (y - z)"),
            ("x / (y * z)", "x / (y * z)"),
            ("x - 2(y - z)", "x - 2 * (y - z)"),
            ("x + (a and b)", "x + (a and b)"),
            ("2 * (a or b)", "2 * (a or b)"),
            (
                "sum(i in 0..3) { x } - sum(i in 0..3) { y }",
                "x + x + x - (y + y + y)",
            ),
            ("sum(i in 0..3) { x - y }", "x - y + x - y + x - y"),
            ("sum(i in 0..2) { sum(j in 0..2) { x } }", "x + x + x + x"),
            ("avg { x, y, z }", "(x + y + z) / 3"),
            (
                "abs { x - y } - min { x, y - z }",
                "abs { x - y } - min { x, y - z }",
            ),
            (
                "(a or b) + (not a or c) + (b or not c) + (not b)",
                "(a or b) + (not a or c) + (b or not c) + not b",
            ),
        ] {
            assert_eq!(compiled_objective(source), format!("min {expected}"));
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn compiled_model_flattens_logic_chains() {
        let model = compile(
            "solve\ns.t.\n    a or b or c\n    xor { a, b, c, d }\n    a and b or c\ndefine\n    a, b, c, d as Boolean",
        );
        let shown = model.to_string();
        let constraints = shown
            .lines()
            .skip(2)
            .take(3)
            .map(str::trim)
            .collect::<Vec<_>>();
        assert_eq!(
            constraints,
            vec!["a or b or c", "a xor b xor c xor d", "(a and b) or c"]
        );
    }

    /// Compares two compiled models by meaning, constraint by constraint.
    fn assert_same_model(first: &Model, second: &Model, shown: &str) {
        assert_eq!(
            canon_exp(&first.objective().rhs),
            canon_exp(&second.objective().rhs),
            "objective changed meaning:\n{shown}"
        );
        assert_eq!(first.constraints().len(), second.constraints().len());
        for (lhs, rhs) in first.constraints().iter().zip(second.constraints()) {
            assert_eq!(lhs.name(), rhs.name());
            assert_eq!(lhs.constraint_type(), rhs.constraint_type());
            assert_eq!(lhs.is_logic_assertion(), rhs.is_logic_assertion());
            assert_eq!(
                canon_exp(lhs.lhs()),
                canon_exp(rhs.lhs()),
                "constraint changed meaning:\n{shown}"
            );
            assert_eq!(
                canon_exp(lhs.rhs()),
                canon_exp(rhs.rhs()),
                "constraint changed meaning:\n{shown}"
            );
        }
    }

    const MODELS: [&str; 7] = [
        //diet problem
        "min sum((cost, i) in enumerate(C)) { cost * x_i }\ns.t.\n    min_{nutrient[j]}: sum(i in 0..f) { a[i][j] * x_i } >= Nmin[j] for j in 0..len(Nmin)\n    max_{nutrient[j]}: sum(i in 0..f) { a[i][j] * x_i } <= Nmax[j] for j in 0..len(Nmax)\nwhere\n    let C = [1.5, 0.5, 2.0]\n    let nutrient = [\"protein\", \"carbs\", \"fats\"]\n    let Nmin = [50, 200, 0]\n    let Nmax = [150, 300, 70]\n    let a = [[30, 0, 5], [2, 45, 0], [2, 15, 20]]\n    let f = len(a)\ndefine\n    x_i as NonNegativeReal(1, 5) for i in 0..3",
        //dominating set, in logic form
        "min sum(u in nodes(G)) { x_u }\ns.t.\n    x_v or any((_, u) in neigh_edges(v)) { x_u } for v in nodes(G)\nwhere\n    let G = Graph { A -> [B, C], B -> [A], C -> [A, D], D -> [C] }\ndefine\n    x_u, x_v as Boolean for v in nodes(G), (_, u) in neigh_edges(v)",
        //maximum satisfiability, logic values used in arithmetic
        "max (a or b) + (not a or c) + (b or not c) + (not b)\ns.t.\n    a or b or c\ndefine\n    a, b, c as Boolean",
        //project selection with dependencies
        "max sum((p, i) in enumerate(profits)) { p * x_i }\ns.t.\n    budget: sum((c, i) in enumerate(costs)) { c * x_i } <= maxBudget\n    x_1 implies x_0\n    not (x_2 and x_3)\n    x_0 or x_3\nwhere\n    let profits = [120, 80, 100, 90]\n    let costs = [50, 30, 40, 45]\n    let maxBudget = 100\ndefine\n    x_i as Boolean for i in 0..len(profits)",
        //flow conservation, a difference of sums
        "min sum(i in 0..3) { c[i] * x_i }\ns.t.\n    sum(i in 0..2) { x_i } - sum(i in 1..3) { x_i } = k - (x_0 - x_2)\n    x_0 - (x_1 - x_2) <= 4 / (k * 2)\nwhere\n    let c = [3, 2, 4]\n    let k = -2\ndefine\n    x_i as Real for i in 0..3",
        //nested sums and products of differences
        "max sum(i in 0..2, j in 0..2) { x_i_j } - sum(i in 0..2) { sum(j in 0..2) { y_i_j - x_i_j } }\ns.t.\n    prod(i in 0..2) { 2 - k } * sum(j in 0..2) { x_0_j - y_0_j } <= 10\nwhere\n    let k = 0.5\ndefine\n    x_i_j, y_i_j as NonNegativeReal for i in 0..2, j in 0..2",
        //satisfaction problem
        "solve\ns.t.\n    a implies (b iff not c)\n    (a implies b) iff c\n    all(i in 0..3) { x_i or a }\ndefine\n    a, b, c as Boolean\n    x_i as Boolean for i in 0..3",
    ];

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn formatted_models_compile_to_the_same_model() {
        for source in MODELS {
            let formatted = RoocParser::new(source.to_string()).format().unwrap();
            assert_same_model(&compile(source), &compile(&formatted), &formatted);
            let again = RoocParser::new(formatted.clone()).format().unwrap();
            assert_eq!(again, formatted, "formatting is not stable");
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn compiled_models_compile_back_to_the_same_model() {
        for source in MODELS {
            let first = compile(source);
            let shown = first.to_string();
            assert_same_model(&first, &compile(&shown), &shown);
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn satisfaction_problems_print_a_bare_solve() {
        let source = "solve\ns.t.\n    a or b\ndefine\n    a, b as Boolean";
        let formatted = RoocParser::new(source.to_string()).format().unwrap();
        assert!(formatted.starts_with("solve\ns.t.\n"), "got: {formatted}");
        let model = compile(source);
        let shown = model.to_string();
        assert!(shown.starts_with("solve\ns.t.\n"), "got: {shown}");
        let linear = Linearizer::linearize(model).expect("failed to linearize");
        let linear_shown = linear.to_string();
        assert!(
            linear_shown.starts_with("solve\ns.t.\n"),
            "got: {linear_shown}"
        );
        for printed in [formatted, shown, linear_shown] {
            compile(&printed);
        }
    }
}
