#[cfg(test)]
mod parser_tests {
    use indexmap::IndexMap;
    use rooc::RoocParser;
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_parser_problems() {
        let input = "
min sum(u in nodes(G)) { x_u }
s.t. 
    x_v + sum((_, u) in neigh_edges(v)) { x_u } >= 1 for v in nodes(G)
where
    let G = Graph {
        A -> [B, C, D, E, F],
        B -> [A, E, C, D, J],
        C -> [A, B, D, E, I],
        D -> [A, B, C, E, H],
        E -> [A, B, C, D, G],
        F -> [A, G, J],
        G -> [E, F, H],
        H -> [D, G, I],
        I -> [C, H, J],
        J -> [B, F, I]
    }
define
    x_u as Boolean for v in nodes(G), (_, u) in edges(G) 
    x_v as Boolean for v in nodes(G)
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to type check problem");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_parser_variants1() {
        let input = "
        min 1
        s.t.
            1 >= 1
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to type check problem");
    }
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_should_allow_st_variant() {
        let input = "
        min 1
        subject to
            1 >= 1
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to type check problem");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_parser_variants2() {
        let input = "
        min 1
        s.t.
            n >= sum(i in A) { i }
        where
            let B = false
            let n = 1
            let S = \"Hello\"
            let S2 = \"\\\"() => {} _ aiaosjd\"
            let A = [1, 2, 3]
            let empty = []
            let M = [[1, 2], [3, 4]]
            let G = Graph {
                A -> [B, C: 10],
                B -> [],
                C
            }
    ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to type check problem");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_parser_variants3() {
        let input = "
            min 1
            s.t.
                c_j: 
                    1 + sum(el in R, i in 0..(el + 1)) { i } <= 1 for R in M
            where
                let M = [[1, 2], [3, 4]]
                let j = 0
            ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to type check problem");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn empty_where() {
        let input = "
            min 1
            s.t.
                1 <= 1
            where
            ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to type check problem");
    }
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn empty_define() {
        let input = "
            min 1
            s.t.
                1 <= 1
            define
            ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to type check problem");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn empty_where_define() {
        let input = "
            min 1
            s.t.
                1 <= 1
            where
            define
            ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to type check problem");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_parser_variants4() {
        let input = "
            min 1
            s.t.
                c1: 
                    avg(i in A) { i } >= 1
                c_2: min(i in A) { i } >= 1
                c_1_2: 
                    max(i in A) { i } >= 1
                c_{A[0]}: sum(i in A) { i } >= 1
                avg {1, 2, 3} >= 1
                min {1, 2, 3} >= 1
                max {1, 2, 3} >= 1
            where
                let A = [1, 2, 3, 4]
            define
            ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to type check problem");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_prefix_operators() {
        let input = "
            min 1
            s.t.
               10 * -1 >= 1
               x + -1 >= 1
               -1 * (x) >= 1
               2 * -1 * (x) >= 1
            define
                x as Real
            ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to type check problem");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_implicit_multiplication() {
        let input = "
        min 1
        s.t.
            10x >= 1
            10(x + y) + 2 >= 1
            10|x + y| + 2>= 1

            (x + y)10 + 2 >= 1
            (x + y)(z - 2) + 2 >= 1
            (x + y)|x + y| + 2 >= 1
            (x + y)z + 2 >= 1

            |x + y|10 + 2 >= 1
            |x + y|(z - 2) + 2 >= 1
            |x + y||x + y| + 2 >= 1
            |x + y|z + 2 >= 1

            2(x + y)3|x + y|z + 2 >= 1
        where
        define
            x,y,z as Real
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to type check problem");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_satisfiability() {
        let input = "
        solve
        s.t.
            x + y + z = 3
        define
            x, y, z as Boolean
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to type check problem");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_parser_errors1() {
        let input = "
            min 1
            s.t.
                A >= 1
            where
                let A = [1, 2, 3, 4]
            ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect_err("Failed to detect invalid primitive type");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_parser_errors2() {
        let input = "
            min 1
            s.t.
                M >= M[i] for i in 0..len(M)
            where
                let M = [[1, 2], [3, 4]]
            ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect_err("Failed to detect invalid primitive type");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_parser_errors3() {
        let input = "
            min 1
            s.t.
             S >= 1
            where
                let S = \"Hello\"\
            define
            ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect_err("Failed to detect invalid primitive type");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_array_iteration() {
        let input = "
        min 1
            s.t.
            sum(row in C, el in row) { el } <= 0
            sum(i in 0..len(C), el in C[i]) { el } <= 0
            sum(i in 0..len(C), j in 0..len(C[i])) { C[i][j] } <= 0

            sum((row, i) in enumerate(C), el in row) { el + i } <= 0
        where
            let C = [
                [1,0,0],
                [0,1,0],
                [0,0,1]
            ]
        define
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to type check problem");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_graph_functions_call() {
        let input = "
        min 1
        s.t.

            sum((u, v, c) in edges(G)) { x_u * c + x_v * c } <= 0
            sum(n in nodes(G)) { x_n } <= 0
            sum((n, i) in enumerate(nodes(G))) { x_n + 1} <= 0
            sum((_, u) in neigh_edges(n)) { x_u } <= 0 for n in nodes(G)
            sum((_, u) in neigh_edges_of(v, G)) { x_u } <= x_v for (v) in edges(G)
        where
            let G = Graph {
                A -> [B: 10, C],
                B -> [A, C],
                C -> [A, B]
            }
        define
            x_u, x_v as Boolean for (u, v) in edges(G)
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to type check problem");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn non_utf8() {
        let input = "
max 2x_1 + x_2 - x_3
subject to
//write the constraints here
5x_1 - 2x_2 + 8x_3 ≤ 15
8x_1+3x_2 -x_3 ≥ 9
x_1+x_2+x_3≤6
where

define
// define the model's variables here
x_1 as Real
x_2 as Real
x_3 as Real
        ";
        
        //i expect to get an error, and not panic
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect_err("");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_const_decl_1() {
        let input = "
        min 1
        s.t.
            1 <= sum(i in n){ x_i * num}
            2 < sum(i in n){ x_i * numSquared}
            3 > sum(i in n){ x_i * numSquared}
            4 >= sum(i in n){ x_i * numSquared}
            5 = sum(i in n){ x_i * numSquared}
        where
            let G = Graph {
                A -> [B: 10, C],
                B -> [A, C],
                C -> [A, B]
            }
            let n = nodes(G)
            let e = edges(G)
            let num = len(n) + 1
            let numSquared = num * num
        define
            x_i as Boolean for i in n
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to type check problem");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_compound_variable() {
        let input = "
        min 1
        s.t.
            x_u + x_{u + 1} + x_{len(c) + 2} <= x_1 + x_{-1}

        where
            let c = [1, 2, 3, 4, 5]
            let u = 1
        define
            x_u, x_{u + 1}, x_{len(c) + 2}, x_1, x_{-1} as Boolean
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to type check problem");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_no_const_keywords_1() {
        let input = "
        min 1
        s.t.
            1 <= 1
        where
            let nodes = [1]
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect_err("Failed to detect invalid identifier name");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect_err("Failed to detect invalid identifier name");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_no_const_keywords_2() {
        let input = "
        min 1
        s.t.
            sum(len in a){ len } <= 1
        where
            let a = [1]
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect_err("Failed to detect invalid identifier name");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect_err("Failed to detect invalid identifier name");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_no_const_keywords_3() {
        let input = "
        min 1
        s.t.
            1 <= 1 for len in a
        where
            let a = [1]
        ";

        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect_err("Failed to detect invalid identifier name");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect_err("Failed to detect invalid identifier name");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_duplicate_domain() {
        let input = "
        min 1
        s.t.
            x <= PI
        define
            x_u as Real for u in 0..10
            x_v as Boolean for v in 0..10
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect_err("Failed to detect duplicate domain");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_as_assertions() {
        let input = r"
        min 1
        s.t.
            x <= PI / 2
        define
            a as Real
            b as Boolean
            e as NonNegativeReal
            x as IntegerRange(10, 20)
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_comments() {
        let input = "
        //aaaa
        min 1 //test
        /* aaa */
        s.t. //aaa
        /*
            aaa
        */
            x <= /*aasdasd*/ 2
        where //aaaaa
            //aaa
            let y = 3 /*
            aaa */
            let /* asdasd */ z = 4
        define //aaaa
            //aa
            x as /* aaaa */ Real //aaa
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to typecheck problem");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_static_variable_check_1() {
        let input = "
        min 1
        s.t.
            x <= 2
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect_err("Failed to detect undeclared static variable");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect_err("Failed to detect undeclared static variable");
    }
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_static_variable_check_2() {
        let input = "
        min 1
        s.t.
            x <= 2
        define
            x as Real
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to typecheck problem");
    }
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_static_variable_check_3() {
        let input = "
        min 1
        s.t.
            x <= 2
        where
            let x = 2
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform(vec![], &IndexMap::new())
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to typecheck problem");
    }
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_static_variable_check_4() {
        let input = "
        min 1
        s.t.
            x <= 2
        where
            let x = 2
        define
            x as Real
        ";
        //TODO should i add this to the compiler too?
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect_err("Failed to typecheck duplicate");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn should_find_domain_type_error() {
        let input = "
        min 1
        s.t.
            x <= 2
        where
            let a = [1,2,3]
        define
            x as IntegerRange(0, a)
        ";
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect_err("Failed to find domain type error");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_dynamic_var_bounds() {
        let input = "
        min 1
        s.t.
            x_1 <= 2
        where
            let x = 2
            let y = [1, 2, 3, 4]
            let length = len(y)
        define
            x_1 as IntegerRange(0, length)
            y_2 as IntegerRange(0, 10)
            z_3 as IntegerRange(0, len(y))
        ";
        RoocParser::new(input.to_string())
            .type_check(&vec![], &IndexMap::new())
            .expect("Failed to typecheck");
    }
}
