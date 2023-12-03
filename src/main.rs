use rooc::{
    math_enums::{Comparison, OptimizationType},
    parser::{parser::parse_problem_source, transformer::transform_parsed_problem},
    solvers::{
        linear_problem::{Constraint, LinearProblem},
        simplex::{IntoCanonicalTableau, Tableau},
    },
};
use term_table::{row::Row, table_cell::TableCell, Table};

#[allow(unused)]
fn main() {
    let mut tableau = Tableau::new(
        vec![-3.0, -4.0, -7.0, 0.0, 0.0],
        vec![vec![1.0, 3.0, 4.0, 1.0, 0.0], vec![2.0, 1.0, 3.0, 0.0, 1.0]],
        vec![1.0, 2.0],
        vec![3, 4],
        0.0,
        0.0,
        create_variable_names(5),
    );
    let mut tableau = Tableau::new(
        vec![3.0, 4.0, 6.0],
        vec![vec![0.0, 1.0, 1.0], vec![1.0, -1.0, 0.0]],
        vec![0.0, 1.0],
        vec![2, 0],
        0.0,
        0.0,
        create_variable_names(3),
    );
    let mut tableau = Tableau::new(
        vec![-3.0, -4.0, -7.0, 0.0, 0.0],
        vec![vec![1.0, 3.0, 4.0, 1.0, 0.0], vec![2.0, 1.0, 3.0, 0.0, 1.0]],
        vec![1.0, 2.0],
        vec![3, 4],
        0.0,
        0.0,
        create_variable_names(5),
    );
    let mut tableau = Tableau::new(
        vec![-4.0, -0.25, -0.25, -0.25, 0.0, 0.0, 0.0],
        vec![
            vec![2.0, 5.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            vec![3.0, 0.0, 10.0, 0.0, 0.0, 1.0, 0.0],
            vec![12.0, 0.0, 0.0, 25.0, 0.0, 0.0, 1.0],
        ],
        vec![75.0, 250.0, 500.0],
        vec![4, 5, 6],
        0.0,
        0.0,
        create_variable_names(7),
    );

    let linear_problem = LinearProblem::new(
        vec![3.0, 4.0, 6.0],
        OptimizationType::Min,
        0.0,
        vec![
            Constraint::new(vec![1.0, 3.0, 4.0], Comparison::Equal, 1.0),
            Constraint::new(vec![2.0, 1.0, 3.0], Comparison::Equal, 2.0),
        ],
        create_variable_names(3),
    );
    let standard_problem = linear_problem.into_standard_form().unwrap();
    let mut tableau = standard_problem.into_canonical().unwrap();

    let result = tableau.solve(1000);
    match result {
        Ok(optimal_tableau) => {
            let pretty = tableau.to_fractional_tableau();
            let table = pretty.pretty_table();
            let mut cli_table = Table::new();
            let values = optimal_tableau.get_variables_values().clone();
            let mut header = Row::new(values.iter().map(TableCell::new));
            header.cells.push(TableCell::new(
                optimal_tableau.get_tableau().get_current_value(),
            ));
            cli_table.add_row(header);
            let empty: Vec<TableCell> = Vec::new();
            cli_table.add_row(Row::new(empty));
            table.iter().for_each(|row| {
                cli_table.add_row(Row::new(row.iter().map(TableCell::new)));
            });
            println!("{}", cli_table.render());
            println!("Optimal value: {}", optimal_tableau.get_optimal_value());
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }

    //dominant set problem
    let problem = "
    min sum(u in nodes(G)) { x_u }
    s.t. 
        x_v + sum((_, _, u) in neigh_edges(v)) { x_u } >= 1 for v in nodes(G)
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
    "
    .to_string();
    let parsed = parse_problem_source(&problem);
    match parsed {
        Ok(parsed) => {
            println!("{:#?}", parsed);
            let transformed = transform_parsed_problem(&parsed);
            println!("\n\n");
            match transformed {
                Ok(transformed) => println!("{}", transformed.to_string()),
                Err(e) => println!("{}", e.get_traced_error()),
            }
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}

fn create_variable_names(n: usize) -> Vec<String> {
    let mut variables = Vec::new();
    for i in 0..n {
        variables.push(format!("x{}", i));
    }
    variables
}

