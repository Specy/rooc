#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;
#[cfg(test)]
mod primitive_tests {
    use indexmap::IndexMap;
    use rooc::RoocParser;

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
}
