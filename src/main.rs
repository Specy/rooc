use term_table::{row::Row, Table, table_cell::TableCell};

use rooc::{
    RoocParser,
    solvers::simplex::{IntoCanonicalTableau, Tableau},
};
use rooc::math::math_enums::{Comparison, OptimizationType};
use rooc::solvers::simplex::OptimalTableau;
use rooc::transformers::linear_model::{LinearConstraint, LinearModel};
use rooc::transformers::linearizer::Linearizer;

#[allow(unused)]
fn main() {
    /*
    for some reason this does not generate a valid basis during solution:
            x + 3y + 4z = 1
            2x + y + 3z = 2
    but this does:
            2x + y + 3z = 2
            x + 3y + 4z = 1
     */
    let source = r#"
        min 3x + 4y + 6z
        s.t.
            2x + y + 3z = 2
            x + 3y + 4z = 1
        define
            x,y,z as Real
    "#
    .to_string();
    let parser = RoocParser::new(source.clone());
    let parsed = parser.parse_and_transform();
    match parsed {
        Ok(parsed) => {
            let linear = Linearizer::linearize(parsed).expect("Failed to linearize");
            println!("\n{}", linear);
            let model = linear.into_standard_form().expect("Failed to standardize");
            println!("\n{}", model);
            let mut tableau = model
                .into_canonical()
                .expect("Failed to transform to canonical tableau");
            let optimal_tableau = tableau.solve(1000).expect("Failed to solve");
            print_tableau(tableau, optimal_tableau);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}

fn print_tableau(tableau: Tableau, optimal_tableau: OptimalTableau) {
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
