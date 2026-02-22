use indexmap::IndexMap;
use rooc::{
    Constant, FunctionContextMap, IterableKind, Primitive, RoocSolver, solve_binary_lp_problem,
};

fn main() {
    let source = "max sum((value, i) in enumerate(values)) { value * x_i }
s.t.
    sum((weight, i) in enumerate(weights)) { weight * x_i } <= capacity
where
    let weights = [10, 60, 30, 40, 30, 20, 20, 2]
    let values = [1, 10, 15, 40, 60, 90, 100, 15]
    let capacity = 102
define
    x_i as Boolean for i in 0..len(weights)";
    let solver = RoocSolver::try_new(source.to_string()).unwrap();
    let solution = solver.solve_using(solve_binary_lp_problem).unwrap();
    println!("{}", solution)
}

#[allow(dead_code)]
fn main_with_data() {
    let source = "
max sum((value, i) in enumerate(values)) { value * x_i }
s.t.
    sum((weight, i) in enumerate(weights)) { weight * x_i } <= capacity
define
    x_i as Boolean for i in 0..len(weights)";
    let constants = vec![
        Constant::from_primitive(
            "weights",
            IterableKind::Integers(vec![10, 60, 30, 40, 30, 20, 20, 2]).into_primitive(),
        ),
        Constant::from_primitive(
            "values",
            IterableKind::Integers(vec![1, 10, 15, 40, 60, 90, 100, 15]).into_primitive(),
        ),
        Constant::from_primitive("capacity", Primitive::Integer(102)),
    ];
    let solver = RoocSolver::try_new(source.to_string()).unwrap();
    //in case you want to define your own functions that will be used during compilation
    let fns: FunctionContextMap = IndexMap::new();
    let solution = solver
        .solve_with_data_using(solve_binary_lp_problem, constants, &fns)
        .unwrap();
    println!("{}", solution)
}
