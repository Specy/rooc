use rooc::{simplex::Tableau, lexer::{Lexer, Token}};
use term_table::{row::Row, table_cell::TableCell, Table};

fn main() {
    let mut tableau = Tableau::new(
        vec![-3.0, -4.0, -7.0, 0.0, 0.0],
        vec![vec![1.0, 3.0, 4.0, 1.0, 0.0], vec![2.0, 1.0, 3.0, 0.0, 1.0]],
        vec![1.0, 2.0],
        vec![3, 4],
        0.0,
    );
    let mut tableau = Tableau::new(
        vec![3.0, 4.0, 6.0],
        vec![vec![0.0, 1.0, 1.0], vec![1.0, -1.0, 0.0]],
        vec![0.0, 1.0],
        vec![2, 0],
        0.0,
    );
    let mut tableau = Tableau::new(
        vec![-3.0, -4.0, -7.0, 0.0, 0.0],
        vec![vec![1.0, 3.0, 4.0, 1.0, 0.0], vec![2.0, 1.0, 3.0, 0.0, 1.0]],
        vec![1.0, 2.0],
        vec![3, 4],
        0.0,
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
    );
    let result = tableau.solve(1000);
    match result {
        Ok(optimal_tableau) => {
            let pretty = tableau.to_fractional_tableau();
            let table = pretty.pretty_table();
            let mut cli_table = Table::new();
            let values = optimal_tableau.get_variables_values().clone();
            let mut header = Row::new(
                values
                    .iter()
                    .map(TableCell::new)
            );
            header.cells.push(TableCell::new(optimal_tableau.get_optimal_value()));
            cli_table.add_row(header);
            let empty: Vec<TableCell> = Vec::new();
            cli_table.add_row(Row::new(empty));
            table.iter().for_each(|row| {
                cli_table.add_row(Row::new(row.iter().map(TableCell::new)));
            });
            //println!("{}", cli_table.render());
            //println!("Optimal value: {}", optimal_tableau.get_optimal_value());
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }

    let problem = "
    max 3x + |x - 2| + 5 - 4(x1 + 8)
    s.t.
        x1 + x2 <= 10
        x1 + 2x2 >= 5
        |2x1 - 5x2| <= 10
        x1, x2 >= 0
    ";
    let lexer = Lexer::new(problem);
    let tokens = lexer.get_tokens();
    //print the tokens and new line after a /n
    for token in tokens.unwrap() {
        match token {
            Token::Newline => println!(),
            _ => print!("{:?} ", token),
        }
    }
}
