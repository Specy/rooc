use indexmap::IndexMap;
use rooc::Linearizer;
use rooc::RoocParser;
use rooc::solve_integer_binary_lp_problem;

fn main() {
    let source = "
max sum((value, i) in enumerate(values)) { value * x_i }
s.t.
    sum((weight, i) in enumerate(weights)) { weight * x_i } <= capacity
where
    let weights = [10, 60, 30, 40, 30, 20, 20, 2]
    let values = [1, 10, 15, 40, 60, 90, 100, 15]
    let capacity = 102
define
    x_i as Boolean for i in 0..len(weights)";

    let rooc = RoocParser::new(source.to_string());
    let parsed = rooc.parse().unwrap();
    let model = parsed.transform(vec![], &IndexMap::new()).unwrap();
    let linear = Linearizer::linearize(model).unwrap();
    let solution = solve_integer_binary_lp_problem(&linear).unwrap();
    println!("{}", solution)
}
