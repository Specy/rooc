#[cfg(test)]
mod primitive_tests {
    use indexmap::IndexMap;
    use rooc::RoocParser;
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_spreadability() {
        let source = "
        min 1
        s.t.
            sum((u,v,c) in edges(G)){ (x_u + x_v)*c } <= 1
            sum((first, second) in A){ first + second } <= 1
            sum((el, j) in enumerate(A[i])){ el * j } <= 1 for i in 0..len(A)
            where 
                let G = Graph {
                    A -> [B],
                    B
                }
                let A = [
                    [1, 2],
                    [3, 4]
                ]
            define
                x_u, x_v as Boolean for (u, v) in edges(G)  
        ";
        RoocParser::new(source.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_apply_logic_ops_to_booleans() {
        use rooc::{ApplyOp, BinOp, Primitive, UnOp};
        let t = Primitive::Boolean(true);
        let f = Primitive::Boolean(false);
        assert_eq!(
            t.apply_binary_op(BinOp::And, &f).unwrap(),
            Primitive::Boolean(false)
        );
        assert_eq!(
            t.apply_binary_op(BinOp::Or, &f).unwrap(),
            Primitive::Boolean(true)
        );
        assert_eq!(
            t.apply_binary_op(BinOp::Xor, &t).unwrap(),
            Primitive::Boolean(false)
        );
        assert_eq!(
            t.apply_binary_op(BinOp::Implies, &f).unwrap(),
            Primitive::Boolean(false)
        );
        assert_eq!(
            f.apply_binary_op(BinOp::Implies, &f).unwrap(),
            Primitive::Boolean(true)
        );
        assert_eq!(
            t.apply_binary_op(BinOp::Iff, &t).unwrap(),
            Primitive::Boolean(true)
        );
        assert_eq!(
            t.apply_unary_op(UnOp::Not).unwrap(),
            Primitive::Boolean(false)
        );
        //unary minus follows the 0/1 arithmetic view of booleans
        assert_eq!(
            t.apply_unary_op(UnOp::Neg).unwrap(),
            Primitive::Number(-1.0)
        );
        assert_eq!(f.apply_unary_op(UnOp::Neg).unwrap(), Primitive::Number(0.0));
        //booleans participate in arithmetic as 0/1
        assert_eq!(
            t.apply_binary_op(BinOp::Add, &Primitive::Number(2.0))
                .unwrap(),
            Primitive::Number(3.0)
        );
        assert_eq!(
            Primitive::Integer(7)
                .apply_binary_op(BinOp::Mul, &f)
                .unwrap(),
            Primitive::Integer(0)
        );
        assert_eq!(
            Primitive::PositiveInteger(7)
                .apply_binary_op(BinOp::Mul, &f)
                .unwrap(),
            Primitive::PositiveInteger(0)
        );
        assert_eq!(
            Primitive::Integer(7)
                .apply_binary_op(BinOp::Div, &t)
                .unwrap(),
            Primitive::Number(7.0)
        );
        assert_eq!(
            Primitive::PositiveInteger(7)
                .apply_binary_op(BinOp::Div, &t)
                .unwrap(),
            Primitive::Number(7.0)
        );
        //numbers reject logic operators
        assert!(
            Primitive::Number(1.0)
                .apply_binary_op(BinOp::And, &Primitive::Number(1.0))
                .is_err()
        );
    }
}
