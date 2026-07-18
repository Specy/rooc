<div align="center">
  <h1><code>ROOC</code></h1>
  <img src="https://raw.githubusercontent.com/Specy/rooc/main/logo-original.png" width="156px" alt="ROOC logo" />
  <p><strong>A mixed-integer linear optimization library and modeling language</strong></p>
</div>

[![Crates.io](https://img.shields.io/crates/v/rooc.svg)](https://crates.io/crates/rooc)

[Language documentation](https://rooc.specy.app/docs/rooc) · [Web platform](https://rooc.specy.app/)

ROOC lets you build linear and mixed-integer models with a fluent Rust API or write them in the ROOC modeling language.

## Quick start

```rust
use rooc::builder::any;
use rooc::{Microlp, ModelBuilder, constraint, vars};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = ModelBuilder::new();

    vars! { model =>
        make_a: bool;
        make_b: bool;
        make_c: bool;
        material: int(0, 8);
    };

    let solution = model
        .maximize(6.0 * make_a + 5.0 * make_b + 4.0 * make_c - material)
        .with(constraint!(2.0 * make_a + 3.0 * make_b + make_c <= material))
        .with(constraint!(make_a -> make_b))
        .with(constraint!(any(vec![make_a, make_c])))
        .solve_with(Microlp::new())?;

    println!("objective = {}", solution.value());
    println!("make_a = {:?}", solution.var_value(make_a));
    println!("material = {:?}", solution.var_value(material));
    Ok(())
}
```

## Build a model

### Variables

Declare variables with the `vars!` macro:

```rust
use rooc::{ModelBuilder, vars};

let mut model = ModelBuilder::new();
vars! { model =>
    a: bool;
    b: int(0, 10);
    c: real;
    d: real(-5.0, 5.0);
    e: nonneg;
    f: nonneg(0.0, 100.0);
    xs[5]: bool;
};
```

Use `xs[i]` to access an indexed family. For computed names, use the methods directly:

```rust,ignore
let y = model.add_var("y", VariableType::integer_range(0, 3));
let zs = model.add_vars("z", 4, VariableType::bool());
```

### Expressions and constraints

Use normal arithmetic and boolean operators with variables and numbers. `sum`, `min`, `max`, `abs`, `all`, and `any` are available from `rooc::builder`.

```rust,ignore
use rooc::builder::{abs, all, any, max, min, sum};

let total = sum(xs.iter().copied());
let high = max(vec![a, b]);
let low = min(vec![a, b]);
let distance = abs(c - 5.0);
let required = all(vec![a, b]);
let implication = a.implies(b);
```

Add constraints with `constraint!` and `with`:

```rust,ignore
model
    .with(constraint!(2.0 * a + b <= 10.0))
    .with(constraint!(cap: a + b == 1.0))
    .with(constraint!(a -> b))
    .with(constraint!(a <-> !b))
    .with_all(vec![
        constraint!(c <= 5.0),
        constraint!(c >= 1.0),
    ]);
```

A constraint without a comparison asserts a boolean expression. Use `with_all` when a loop produces several constraints:

```rust
use rooc::{ModelBuilder, constraint, vars};

let mut model = ModelBuilder::new();
vars! { model => xs[3]: nonneg; }

for (x, capacity) in xs.iter().copied().zip([1.0, 2.0, 3.0]) {
    model = model.with(constraint!(x <= capacity));
}

let model = model.satisfy();
```

Set an objective with `maximize`, `minimize`, or `satisfy`:

```rust,ignore
model.maximize(sum(xs.iter().copied()));
model.minimize(a + b);
model.satisfy();
```

The builder returns a new value after each call. Assign it back when you build constraints in a loop.

## Solve a model

### Built-in solvers

Pass a solver to `solve_with`:

| Solver | Use for |
| --- | --- |
| `Auto` | Safe general-purpose MILP default |
| `Microlp::new()` | Mixed-integer models and configurable MIP options |
| `Clarabel` | Continuous models |

```rust,ignore
use rooc::Auto;

let solution = model
    .maximize(objective)
    .solve_with(Auto)?;
```

`Auto` selects ROOC's safe general-purpose MILP default and uses Microlp for every supported model. Use `Microlp::new()` when you need MIP options:

```rust,ignore
use rooc::Microlp;
use std::time::Duration;

let solver = Microlp::new()
    .with_mip_gap(0.0)
    .with_time_limit(Duration::from_secs(5));
```

### Read the solution

Read values back through the variables used to build the model:

```rust,ignore
solution.value();
solution.var_value(x);
solution.numeric_value(x);
solution.eval(&(2.0 * x + y));
```

## Export a model

Call `to_lp_format` on a linearized model to create CPLEX LP text:

```rust,ignore
let lp_text = model
    .maximize(3.0 * x + 2.0 * y + z)
    .with(constraint!(cap: 2.0 * x + y + z <= 8.0))
    .linearize()?
    .to_lp_format();
```

## Use the ROOC language

The language is useful for models that use data, iteration, or graphs:

```rust
use rooc::{RoocSolver, solve_milp_lp_problem};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = "max sum((value, i) in enumerate(values)) { value * x_i }
s.t.
    sum((weight, i) in enumerate(weights)) { weight * x_i } <= capacity
where
    let weights = [10, 60, 30, 40, 30, 20, 20, 2]
    let values = [1, 10, 15, 40, 60, 90, 100, 15]
    let capacity = 102
define
    x_i as Boolean for i in 0..len(weights)";

    let solver = RoocSolver::try_new(source.to_string())?;
    let solution = solver.solve_using(solve_milp_lp_problem)?;
    println!("{}", solution);
    Ok(())
}
```

The language supports collections, graphs, indexed constraints, boolean logic, and the `abs { }`, `min { }`, and `max { }` blocks.

```lua
max abs { x }
s.t.
    -10 <= x
    x <= 6
define
    x as Real
```

ROOC derives the bounds needed to model these expressions from declarations and constraints. If a required finite bound is unavailable, compilation reports an error instead of guessing a Big-M value.

## Build a linear model directly

If your application already has coefficient vectors, use `LinearModel` directly:

```rust
use rooc::{
    Comparison, LinearModel, OptimizationType, VariableType, solve_real_lp_problem_clarabel,
};

let mut model = LinearModel::new();
model.add_variable("x1", VariableType::non_negative_real());
model.add_variable("x2", VariableType::real());
model.add_constraint(vec![1.0, 1.0], Comparison::LessOrEqual, 5.0);
model.set_objective(vec![1.0, 2.0], OptimizationType::Max);

let solution = solve_real_lp_problem_clarabel(&model).unwrap();
println!("{}", solution);
```

For complete language examples, see the [language documentation](https://rooc.specy.app/docs/rooc).

## License

ROOC is released under the **MPL-2.0** license.
