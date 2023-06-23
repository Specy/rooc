use rooc::simplex::Tableau;


fn main() { 
    let mut tableau = Tableau::new(
        vec![-3.0, -4.0, -7.0, 0.0, 0.0],
        vec![
            vec![1.0,3.0,4.0,1.0,0.0],
            vec![2.0,1.0,3.0,0.0,1.0],
        ],
        vec![1.0, 2.0],
        vec![3, 4],
    );
    let mut tableau = Tableau::new(
        vec![3.0, 4.0, 6.0],
        vec![
            vec![0.0, 1.0, 1.0],
            vec![1.0, -1.0, 0.0],
        ],
        vec![0.0, 1.0],
        vec![2, 0],
    );
    let mut tableau = Tableau::new(
        vec![-3.0, -4.0, -7.0, 0.0, 0.0],
        vec![
            vec![1.0,3.0,4.0,1.0,0.0],
            vec![2.0,1.0,3.0,0.0, 1.0],
        ],
        vec![1.0,2.0],
        vec![3,4],
    );
    let result = tableau.solve(1000);
    match result{
        Ok(optimal_tableau) => {
            println!("X*={:?} \nv={}", optimal_tableau.get_variables_values().clone(), optimal_tableau.get_optimal_value());
            let pretty = tableau.to_fractional_tableau();
            println!("{}", pretty.pretty_string())
        },
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
