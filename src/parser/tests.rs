#[cfg(test)]
mod parser_tests {
    use crate::RoocParser;

    #[test]
    fn test_parser_problems() {
        let input = "
min sum(u in nodes(G)) { x_u }
s.t. 
    x_v + sum((_, _, u) in neigh_edges(v)) { x_u } >= 1    for v in nodes(G)
where
    G = Graph {
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
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check()
            .expect("Failed to type check problem");
    }
    #[test]
    fn test_parser_variants1() {
        let input = "
        min 1
        s.t.
            1 >= 1
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check()
            .expect("Failed to type check problem");
    }
    #[test]
    fn test_parser_variants2() {
        let input = "
        min 1
        s.t.
            N >= sum(i in A) { i }
        where
            B = false
            N = 1
            S = \"Hello\"
            A = [1, 2, 3]
            M = [[1, 2], [3, 4]]
            G = Graph {
                A -> [B, C: 10],
                B -> [],
                C
            }
    ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check()
            .expect("Failed to type check problem");
    }

    #[test]
    fn test_parser_variants3() {
        let input = "
            min 1 
            s.t.
                1 + sum(el in R, i in 0..(el + 1)) { i } <= 1 for R in M
            where
                M = [[1, 2], [3, 4]]
            ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check()
            .expect("Failed to type check problem");
    }

    #[test]
    fn test_parser_variants4() {
        let input = "
            min 1
            s.t.
                avg(i in A) { i } >= 1
                min(i in A) { i } >= 1
                max(i in A) { i } >= 1
                sum(i in A) { i } >= 1
                avg {1, 2, 3} >= 1
                min {1, 2, 3} >= 1
                max {1, 2, 3} >= 1
            where
                A = [1, 2, 3, 4]
            ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check()
            .expect("Failed to type check problem");
    }
    #[test]
    fn test_prefix_operators() {
        let input = "
            min 1
            s.t.
               10 * -1 >= 1
               x + -1 >= 1
               -1 * (x) >= 1
               2 * -1 * (x) >= 1
            ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check()
            .expect("Failed to type check problem");
    }
    #[test]
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
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check()
            .expect("Failed to type check problem");
    }
    #[test]
    fn test_parser_errors1() {
        let input = "
            min 1
            s.t.
                A >= 1
            where 
                A = [1, 2, 3, 4]
            ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect_err("Failed to detect invalid primitive type");
    }

    #[test]
    fn test_parser_errors2() {
        let input = "
            min 1
            s.t.
                M >= M[i] for i in 0..len(M)
            where 
                M = [[1, 2], [3, 4]]
            ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect_err("Failed to detect invalid primitive type");
    }

    #[test]
    fn test_parser_errors3() {
        let input = "
            min 1
            s.t.
             S >= 1
            where 
                S = \"Hello\"
            ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect_err("Failed to detect invalid primitive type");
    }

    #[test]
    fn test_array_iteration() {
        let input = "
        min 1
            s.t.
            sum(row in C, el in row) { el } <= 0
            sum(i in 0..len(C), el in C[i]) { el } <= 0
            sum(i in 0..len(C), j in 0..len(C[i])) { C[i][j] } <= 0

            sum((row, i) in enumerate(C), el in row) { el + i } <= 0
        where
            C = [
                [1,0,0],
                [0,1,0], 
                [0,0,1]
            ]
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check()
            .expect("Failed to type check problem");
    }

    #[test]
    fn test_graph_functions_call() {
        let input = "
        min 1
        s.t.

            sum((u, c, v) in edges(G)) { x_u * c + x_v * c } <= 0
            sum(n in nodes(G)) { x_n } <= 0
            sum((n, i) in enumerate(nodes(G))) { x_n + 1} <= 0
            sum((_, _, u) in neigh_edges(n)) { x_u } <= 0 for n in nodes(G)
            sum((_, _, u) in neigh_edges_of(v, G)) { x_u } <= x_v for (v) in edges(G)
        where
            G = Graph {
                A -> [B: 10, C],
                B -> [A, C],
                C -> [A, B]
            }
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check()
            .expect("Failed to type check problem");
    }

    #[test]
    fn test_const_decl_1() {
        let input = "
        min 1
        s.t.
            1 <= sum(i in n){ x_i * num}
        where
    
            G = Graph {
                A -> [B: 10, C],
                B -> [A, C],
                C -> [A, B]
            }
            n = nodes(G)
            e = edges(G)
            num = len(n) + 1
            numSquared = num * num
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check()
            .expect("Failed to type check problem");
    }

    #[test]
    fn test_compound_variable() {
        let input = "
        min 1
        s.t. 
            x_u + x_{u + 1} + x_{len(c) + 2} <= x_1 + x_{-1}
            
        where
            c = [1, 2, 3, 4, 5]
            u = 1    
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect("Failed to parse and transform problem");
        RoocParser::new(input.to_string())
            .type_check()
            .expect("Failed to type check problem");
    }
    #[test]
    fn test_no_const_keywords_1(){
        let input = "
        min 1
        s.t.
            1 <= 1
        where
            nodes = [1]
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect_err("Failed to detect invalid primitive type");
        RoocParser::new(input.to_string())
            .type_check()
            .expect_err("Failed to detect invalid primitive type");
    }
    
    #[test]
    fn test_no_const_keywords_2(){
        let input = "
        min 1
        s.t.
            sum(len in a){ len } <= 1
        where
            a = [1]
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect_err("Failed to detect invalid primitive type");
        RoocParser::new(input.to_string())
            .type_check()
            .expect_err("Failed to detect invalid primitive type");
    }
    #[test]
    fn test_no_const_keywords_3(){
        let input = "
        min 1
        s.t.
            1 <= 1 for len in a
        where
            a = [1]
        ";
        RoocParser::new(input.to_string())
            .parse_and_transform()
            .expect_err("Failed to detect invalid primitive type");
        RoocParser::new(input.to_string())
            .type_check()
            .expect_err("Failed to detect invalid primitive type");
    }
}