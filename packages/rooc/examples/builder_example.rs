use rooc::builder::any;
use rooc::{Microlp, ModelBuilder, constraint, vars};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Small production model built with the builder API\n");

    let mut model = ModelBuilder::new();

    // Variables: three yes/no product decisions and one integer quantity.
    vars! { model =>
        make_a: bool;       // manufacture product A?
        make_b: bool;       // manufacture product B?
        make_c: bool;       // manufacture product C?
        material: int(0, 8); // batches of raw material to buy (0..8)
    };

    let solution = model
        // Maximize the value of the products made, minus the material cost.
        .maximize(6.0 * make_a + 5.0 * make_b + 4.0 * make_c - material)
        // Each product consumes material, so we must buy enough to cover them.
        .with(constraint!(2.0 * make_a + 3.0 * make_b + make_c <= material))
        // Making A requires also making B.
        .with(constraint!(make_a -> make_b))
        // At least one of A or C must be made.
        .with(constraint!(any(vec![make_a, make_c])))
        // Solve with the built-in MILP solver.
        .solve_with(Microlp::new())?;

    // Read results back with the variable handles.
    println!("objective = {}", solution.value());
    println!("make_a    = {:?}", solution.var_value(make_a).unwrap());
    println!("make_b    = {:?}", solution.var_value(make_b).unwrap());
    println!("make_c    = {:?}", solution.var_value(make_c).unwrap());
    println!("material  = {:?}", solution.var_value(material).unwrap());

    Ok(())
}
