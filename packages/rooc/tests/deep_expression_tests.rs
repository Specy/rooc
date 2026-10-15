//! Aggregations over many terms must not be bounded by the stack. The passes
//! over expressions are recursive, so the compiler builds long sums, products
//! and logic chains as balanced trees, which nest only log2(n) levels deep.

#[cfg(test)]
mod deep_expression_tests {
    use indexmap::IndexMap;
    use rooc::model_transformer::{Exp, Model};
    use rooc::{Comparison, Linearizer, RoocParser};
    use std::collections::HashMap;
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    fn transformed(source: &str) -> Model {
        RoocParser::new(source.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("failed to transform source model")
    }

    /// The nesting depth of the expression, walked with an explicit stack so
    /// that measuring a deep tree cannot overflow.
    fn depth(root: &Exp) -> usize {
        let mut max_depth = 0;
        let mut pending = vec![(root, 1usize)];
        while let Some((exp, depth)) = pending.pop() {
            max_depth = max_depth.max(depth);
            match exp {
                Exp::Number(_) | Exp::Variable(_) => {}
                Exp::Abs(inner) | Exp::Not(inner) | Exp::UnOp(_, inner) => {
                    pending.push((inner, depth + 1));
                }
                Exp::Min(exps) | Exp::Max(exps) | Exp::And(exps) | Exp::Or(exps) => {
                    pending.extend(exps.iter().map(|exp| (exp, depth + 1)));
                }
                Exp::Xor(lhs, rhs)
                | Exp::Implies(lhs, rhs)
                | Exp::Iff(lhs, rhs)
                | Exp::BinOp(_, lhs, rhs) => {
                    pending.push((lhs, depth + 1));
                    pending.push((rhs, depth + 1));
                }
            }
        }
        max_depth
    }

    /// The variables of the expression, from left to right.
    fn variables(root: &Exp) -> Vec<String> {
        let mut names = Vec::new();
        let mut pending = vec![root];
        while let Some(exp) = pending.pop() {
            match exp {
                Exp::Number(_) => {}
                Exp::Variable(name) => names.push(name.clone()),
                Exp::Abs(inner) | Exp::Not(inner) | Exp::UnOp(_, inner) => pending.push(inner),
                Exp::Min(exps) | Exp::Max(exps) | Exp::And(exps) | Exp::Or(exps) => {
                    pending.extend(exps.iter().rev());
                }
                Exp::Xor(lhs, rhs)
                | Exp::Implies(lhs, rhs)
                | Exp::Iff(lhs, rhs)
                | Exp::BinOp(_, lhs, rhs) => {
                    pending.push(rhs);
                    pending.push(lhs);
                }
            }
        }
        names
    }

    fn names(prefix: &str, n: usize) -> Vec<String> {
        (0..n).map(|i| format!("{prefix}_{i}")).collect()
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn aggregations_nest_logarithmically() {
        // 2^12 terms, so a balanced tree has 12 levels of operators above its leaves
        let n = 4096;
        let model = transformed(&format!(
            "min sum(i in 0..{n}) {{ x_i }}
s.t.
    avg(i in 0..{n}) {{ x_i }} <= prod(i in 0..{n}) {{ 1 }}
    xor(i in 0..{n}) {{ b_i }}
define
    x_i as NonNegativeReal for i in 0..{n}
    b_i as Boolean for i in 0..{n}"
        ));

        let objective = &model.objective().rhs;
        assert!(depth(objective) <= 13);
        assert_eq!(variables(objective), names("x", n));

        let [average, parity] = model.constraints().as_slice() else {
            panic!("expected two constraints");
        };
        // the average is the sum divided by the number of terms
        assert!(depth(average.lhs()) <= 14);
        assert_eq!(variables(average.lhs()), names("x", n));
        assert!(depth(average.rhs()) <= 13);
        assert!(depth(parity.lhs()) <= 13);
        assert_eq!(variables(parity.lhs()), names("b", n));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn large_aggregations_linearize() {
        // a sum this long used to overflow the stack while being linearized
        let n = 10_000;
        let model = transformed(&format!(
            "min sum(i in 0..{n}) {{ x_i }} + abs {{ sum(i in 0..{n}) {{ y_i }} }}
s.t.
    sum(i in 0..{n}) {{ x_i }} >= 1
    all(i in 0..{n}) {{ b_i }}
define
    x_i as NonNegativeReal for i in 0..{n}
    y_i as Real(-1, 1) for i in 0..{n}
    b_i as Boolean for i in 0..{n}"
        ));
        let linear = Linearizer::linearize(model).expect("failed to linearize");

        let index = linear
            .variables()
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect::<HashMap<_, _>>();
        let has_ones = |coefficient: &dyn Fn(usize) -> f64, prefix: &str| {
            names(prefix, n)
                .iter()
                .all(|name| coefficient(index[name.as_str()]) == 1.0)
        };

        // the absolute value is minimized through a single auxiliary variable
        assert_eq!(linear.variables().len(), 3 * n + 1);
        assert!(has_ones(&|i| linear.objective()[i], "x"));
        assert_eq!(linear.objective()[index["$abs_0"]], 1.0);
        assert!(
            linear.constraints().iter().any(|constraint| {
                *constraint.constraint_type() == Comparison::GreaterOrEqual
                    && constraint.rhs() == 1.0
                    && constraint.indices().len() == n
                    && has_ones(&|i| constraint.coefficient(i), "x")
            }),
            "missing the sum over x"
        );
        // all(b) is lowered to the single row b_0 + ... + b_(n-1) = n
        assert!(
            linear.constraints().iter().any(|constraint| {
                *constraint.constraint_type() == Comparison::Equal
                    && constraint.rhs() == n as f64
                    && constraint.indices().len() == n
                    && has_ones(&|i| constraint.coefficient(i), "b")
            }),
            "missing the conjunction over b"
        );
    }
}
