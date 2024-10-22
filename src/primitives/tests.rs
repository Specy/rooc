#[cfg(test)]
mod primitive_tests {
    use crate::RoocParser;

    #[test]
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
            .parse_and_transform()
            .expect("Failed to parse");
    }
}
