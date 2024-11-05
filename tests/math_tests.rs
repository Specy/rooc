#[cfg(test)]
mod math_tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;
    use rooc::{
        math_enums::{Comparison, OptimizationType},
        operators::{BinOp, UnOp},
    };

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_operator_parse() {
        let bin_ops = ["+", "-", "*", "/"];
        let bin_ops_enum = [BinOp::Add, BinOp::Sub, BinOp::Mul, BinOp::Div];
        for (i, op) in bin_ops.iter().enumerate() {
            assert_eq!(
                op.parse::<BinOp>().expect("Failed to parse"),
                bin_ops_enum[i]
            );
        }
        let un_ops = ["-"];
        let un_ops_enum = [UnOp::Neg];
        for (i, op) in un_ops.iter().enumerate() {
            assert_eq!(op.parse::<UnOp>().expect("Failed to parse"), un_ops_enum[i]);
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_math_enums_parse() {
        let comparisons = ["<=", ">=", "="];
        let comparisons_enum = [
            Comparison::LessOrEqual,
            Comparison::GreaterOrEqual,
            Comparison::Equal,
        ];
        for (i, op) in comparisons.iter().enumerate() {
            assert_eq!(
                op.parse::<Comparison>().expect("Failed to parse"),
                comparisons_enum[i]
            );
        }
        let optimization_types = ["min", "max"];
        let optimization_types_enum = [OptimizationType::Min, OptimizationType::Max];
        for (i, op) in optimization_types.iter().enumerate() {
            assert_eq!(
                op.parse::<rooc::math_enums::OptimizationType>()
                    .expect("Failed to parse"),
                optimization_types_enum[i]
            );
        }
    }
}
