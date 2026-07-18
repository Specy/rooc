<div align="center">
  <h1><code>ROOC</code></h1>
  <img src='https://raw.githubusercontent.com/Specy/rooc/main/logo-original.png' width='156px'/>
  <p><strong>A mixed-integer linear optimization modeling language and solver</strong></p>
</div>

[![Crates.io](https://img.shields.io/crates/v/rooc.svg)](https://crates.io/crates/rooc)

[Library documentation](https://rooc.specy.app/docs/rooc) · [Language documentation](https://rooc.specy.app/docs/rooc) · [Web modeling platform](https://rooc.specy.app/)

ROOC lets you build optimization models through a **fluent Rust API** or with the **ROOC modeling language**. It supports arithmetic and logic constraints, as well as a set of utilities to simplify modeling. Everything is then linearized and solved with a solver. With the fluent API, choose a solver explicitly: `Auto` selects ROOC's safe general-purpose `Microlp` MILP default, while `Microlp::new()` lets you configure MIP options. You can also select `Clarabel` for continuous LPs or provide your own solver.

## Table of Contents

- [Quick start (fluent API)](#quick-start-fluent-api)
- [Building models](#building-models)
  - [Variables](#variables)
  - [Expressions and operators](#expressions-and-operators)
  - [Constraints](#constraints)
  - [Objective](#objective)
- [Solving](#solving)
  - [Built-in solvers](#built-in-solvers)
  - [Reading the solution](#reading-the-solution)
  - [Writing your own solver](#writing-your-own-solver)
  - [Continue solving](#continue-solving)
  - [Lower-level access](#lower-level-access)
- [Exporting to LP format](#exporting-to-lp-format)
- [Alternative: the ROOC modeling language](#alternative-the-rooc-modeling-language)
  - [Iteration and data](#iteration-and-data)
  - [Graphs and logic](#graphs-and-logic)
  - [`abs`, `min` and `max`](#abs-min-and-max)
  - [Compiling and solving](#compiling-and-solving)
- [Modeling examples](#modeling-examples)
  - [Dominating set (graphs)](#dominating-set-graphs)
- [Directly building a linear model](#directly-building-a-linear-model)

## Quick start (fluent API)

A small production model with boolean and integer variables and logic constraints:

```rust
use rooc::builder::any;
use rooc::{Microlp, ModelBuilder, constraint, vars};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = ModelBuilder::new();

    // Variables: three yes/no product decisions and one integer quantity.
    vars! { model =>
        make_a: bool;        // manufacture product A?
        make_b: bool;        // manufacture product B?
        make_c: bool;        // manufacture product C?
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

    // Read the results.
    println!("objective = {}", solution.value());
    println!("make_a    = {:?}", solution.var_value(make_a).unwrap());
    println!("material  = {:?}", solution.var_value(material).unwrap());
    Ok(())
}
```

The rest of this section explains each piece.

## Building models

### Variables

Declare variables with the `vars!` macro. The name on the left of `:` becomes a variable you can use directly:

```rust
use rooc::{ModelBuilder, VariableType, vars};

let mut model = ModelBuilder::new();
vars! { model =>
    a: bool;               // boolean (0/1)
    b: int(0, 10);         // integer in [0, 10]
    c: real;               // real, unbounded
    d: real(-5.0, 5.0);    // real in [-5, 5]
    e: nonneg;             // real >= 0
    f: nonneg(0.0, 100.0); // real in [0, 100]
    xs[5]: bool;           // an indexed family: `xs` is a Vec<Var>, use xs[i]
};
```

`xs[5]` declares an indexed family: `xs` is a list you index as `xs[i]` inside expressions and constraints. The count can be any expression.

You can also declare variables through methods (handy when the name is computed):

```ignore
let y = model.add_var("y", VariableType::integer_range(0, 3)); // -> Var
let zs = model.add_vars("z", 4, VariableType::bool());         // -> Vec<Var> (z_0..z_3)
```

Variable names must be unique; declaring the same name twice panics.

### Expressions and operators

Variables and expressions support the usual arithmetic and logic operators, mixed with plain numbers:

```ignore
use rooc::builder::{abs, all, any, max, min, sum};

let e = 2.0 * a + b - 3 * c;      // arithmetic
let total = sum(xs.iter().copied()); // sum of an iterator of variables/expressions
let hi = max(vec![a, b]);          // max{ a, b }
let lo = min(vec![a, b]);          // min{ a, b }
let gap = abs(c - 5.0);            // abs{ c - 5 }

// Logic (also `&`, `|`, `^`, `!` operators, plus `.implies()` / `.iff()` methods)
let clause = any(vec![a, b]);      // a or b
let every = all(vec![a, b]);       // a and b
let cond = a.implies(b);           // a -> b
```

`abs`, `min` and `max` are linearized automatically, using bounds inferred from the model.

### Constraints

Build constraints with the `constraint!` macro and add them with `with` (one) or `with_all` (several):

```ignore
model
    .with(constraint!(2.0 * a + b <= 10.0))     // <=, >=, ==, <, > comparisons
    .with(constraint!(cap: a + b == 1.0))       // named constraint (read back by name)
    .with(constraint!(a -> b))                  // logic assertion: a implies b
    .with(constraint!(a <-> !b))                // logic assertion: a iff not b
    .with(constraint!(any(vec![a, b])))         // logic aggregation must hold
    .with_all(vec![
        constraint!(c <= 5.0),
        constraint!(c >= 1.0),
    ]);                                          // add many at once
```

A `constraint!` without a comparison (like `a -> b`) asserts that the logic expression must hold. Use `with_all` to add several constraints at once, for example ones built in a loop.

`with`, `with_all`, `maximize`, `minimize`, and `satisfy` take ownership of the builder and return it. That keeps fluent chains concise; when generating constraints in a loop, assign the returned builder back:

```rust
use rooc::{ModelBuilder, constraint, vars};

let mut model = ModelBuilder::new();
vars! { model => xs[3]: nonneg; }

for (x, capacity) in xs.iter().copied().zip([1.0, 2.0, 3.0]) {
    model = model.with(constraint!(x <= capacity));
}

let _model = model.satisfy();
```

### Objective

Set the objective with `maximize`, `minimize`, or `satisfy` (feasibility only). It can go anywhere, before or after constraints:

```ignore
model.maximize(sum(xs.iter().copied())); // maximize
model.minimize(a + b);                    // minimize
model.satisfy();                          // feasibility: find any feasible point
```

Setting the objective twice overrides it. If you never set one, the model is solved as a feasibility problem.

## Solving

### Built-in solvers

`solve_with` accepts any solver. Three are built in:

| Solver | Value type | Use for |
| --- | --- | --- |
| `Microlp` | `MILPValue` | mixed-integer models (boolean, integer, real) |
| `Clarabel` | `f64` | purely continuous (real) models |
| `Auto` | `MILPValue` | safe general-purpose default; uses `Microlp` for every supported model |

```ignore
use rooc::{Auto, Clarabel, Microlp};

let solution = model
    .maximize(obj)
    .solve_with(Microlp::new())?;
```

Use `Auto` when you want ROOC's safe general-purpose default. Use `Microlp::new()` when you need its MIP options, and select `Clarabel` explicitly for a continuous LP. The microlp solver takes optional settings:

```ignore
use std::time::Duration;

let solver = Microlp::new()
    .with_mip_gap(0.0)
    .with_time_limit(Duration::from_secs(5));
```

### Reading the solution

Read values back using the variables. These are always available:

```ignore
solution.value();              // objective value (f64)
solution.var_value(x);         // Option<Value>: MILPValue for MILP, f64 for real
solution.numeric_value(x);     // Option<f64>: the value as a plain number
solution.eval(&(2.0 * x + y)); // f64: evaluate any expression at the solution
```

Other methods exist only when the solver's solution provides them, so you can't call one the solver can't answer:

```ignore
solution.status();                // if the solution implements SolveStatus
solution.constraint_value("cap"); // if it implements ConstraintValues
solution.shadow_price("cap");     // if it implements DualValues
solution.reduced_cost("x");       // if it implements ReducedCosts
```

The built-in solvers implement `SolveStatus` and `ConstraintValues`, so `status()` and `constraint_value()` work. They do not implement `DualValues` or `ReducedCosts`, so `shadow_price` and `reduced_cost` do not exist on their solutions at all. A solver that computes duals implements `DualValues` on its solution type, and then `shadow_price` appears.

### Writing your own solver

Implement `Solver` for any type and it works with `solve_with`. A solver returns its own solution type; the simplest choice is to reuse the built-in `LpSolution`:

```rust
use rooc::{Assignment, LinearModel, LpSolution, Solver, SolverError};

struct MySolver;

impl Solver for MySolver {
    type Solution = LpSolution<f64>; // any type implementing `Solution`

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        // Read the problem: model.variables(), .constraints(), .objective() ...
        let assignment = model
            .variables()
            .iter()
            .map(|name| Assignment { name: name.clone(), value: 0.0 })
            .collect();
        Ok(LpSolution::new(assignment, model.objective_offset(), Default::default()))
    }
}
```

To expose extra capabilities, give the solver its own solution type and implement the matching trait on it. For example, to provide duals:

```ignore
use rooc::{DualValues, Solution};

struct MySolution { /* your data */ }

impl Solution for MySolution {
    type Value = f64;
    fn objective_value(&self) -> f64 { /* ... */ }
    fn var_value(&self, variable: &str) -> Option<f64> { /* ... */ }
}

impl DualValues for MySolution {
    fn shadow_price(&self, constraint: &str) -> Option<f64> { /* ... */ }
}
```

Now `shadow_price` is available on solutions from that solver, and only from it.

### Continue solving

A solver that implements the `Reoptimizable` marker lets you edit a solved model and solve again. Import `Reoptimize`, add constraints to the solution, and `resolve`:

```ignore
use rooc::Reoptimize;

let solution = model
    .maximize(x)
    .solve_with(Microlp::new())?;

let tighter = solution
    .with(constraint!(x <= 4.0))
    .resolve()?;
```

`Microlp`, `Clarabel` and `Auto` support this, re-solving the edited model.

### Lower-level access

The builder can also hand you the intermediate representations:

```ignore
let model_ir = builder.into_model(); // the model, same as the language compiles to
let linear = builder.linearize()?;   // the linearized model, ready for a solver function
```

## Exporting to LP format

Call `to_lp_format` on a linearized model to get a CPLEX LP string. It is an explicit export (not `Display`):

```ignore
let lp_text = model
    .maximize(3.0 * x + 2.0 * y + z)
    .with(constraint!(cap: 2.0 * x + y + z <= 8.0))
    .linearize()?
    .to_lp_format();
```

```text
Maximize
 obj: 3 x + 2 y + z
Subject To
 cap: 2 x + y + z <= 8
Bounds
 0 <= y <= 5
 0 <= z <= 8
Binary
 x
General
 y
End
```

Booleans go in the `Binary` section, integers in `General`, and non-default continuous bounds in `Bounds`. The bounds reflect the linearized model, so they can be tighter than declared while staying equivalent.

## Alternative: the ROOC modeling language

Instead of building the model in code, you can write it as a formal model in the ROOC language and compile it. This is often clearer for large or parameterized models: you describe the problem once, over sets and data, and the compiler expands it into the concrete linear model.

A model has up to four sections:

- the objective: `min` / `max` of an expression (or `solve` for feasibility),
- `s.t.`: the constraints,
- `where`: data and constants (numbers, arrays, matrices, graphs, ...),
- `define`: the variable domains.

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

### Iteration and data

Aggregations iterate over data. `sum((value, i) in enumerate(values)) { value * x_i }` sums the body over a set (with tuple destructuring), and a trailing `for i in 0..n` expands one constraint into one row per element. Data lives in the `where` section numbers, arrays, matrices, constants and utilities like `enumerate`, `len` and ranges keep the model concise. Indexed variables such as `x_i` are created by the `define` section.

### Graphs and logic

Graphs are first-class, with helpers like `nodes(G)`, `edges(G)` and `neigh_edges(v)`. Constraints can use logic operators: `and` (`&&`), `or` (`||`), `not` (`!`), `implies` (`->`), `iff` (`<->`) and `xor`, plus the aggregations `all`, `any` and `xor`. A constraint written without a comparison asserts that the logic expression must hold, and a logic expression can also be used as a 0/1 value inside arithmetic.

Here is [vertex cover](https://en.wikipedia.org/wiki/Vertex_cover) "at least one endpoint of each edge must be selected" as a single logic clause expanded over the edges of a graph:

```lua
min sum(v in nodes(G)) { x_v }
s.t.
    x_u or x_v for (u, v) in edges(G)
where
    let G = Graph {
        A -> [B, C],
        B -> [D],
        C -> [D],
        D -> [E],
        E
    }
define
    x_v as Boolean for v in nodes(G)
```

Each edge clause compiles to a single linear row; affine logic assertions do not introduce an extra Boolean witness variable.

### `abs`, `min` and `max`

The absolute value is the `abs { }` block (for example `min abs { x - 5 }`), and `min { ... }` / `max { ... }` blocks work the same way. They are linearized using bounds inferred from declarations and constraints, so a favorable one-sided context can stay a pure LP, and no arbitrary Big-M constant is ever inserted. If an exact formulation needs a bound that cannot be proven finite, compilation reports an error asking for bounds instead of producing an unsafe model.

```lua
max abs { x }
s.t.
    -10 <= x
    x <= 6
define
    x as Real
```

### Compiling and solving

`solve_using` takes any `Fn(&LinearModel) -> Result<_, _>` solver function (`solve_milp_lp_problem`, `solve_real_lp_problem_clarabel`, `auto_solver`, or your own). To pass data and custom functions separately, use `solve_with_data_using(func, constants, &fns)`.

For the full language reference like functions, tuples, multi-dimensional arrays, primitive destructuring, variable bounds and JavaScript-defined functions, see the [language documentation](https://rooc.specy.app/docs/rooc).

## Modeling examples

### Dominating set (graphs)

The [dominating set](https://en.wikipedia.org/wiki/Dominating_set) problem, modeled over a graph with the iteration helpers `nodes(G)` and `neigh_edges(v)`: pick the fewest nodes so that every node is itself picked or adjacent to a picked node.

```lua
min sum(u in nodes(G)) { x_u }
s.t.
    x_v + sum((_, u) in neigh_edges(v)) { x_u } >= 1 for v in nodes(G)
where
    let G = Graph {
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
define
    x_u, x_v as Boolean for v in nodes(G), (_, u) in neigh_edges(v)
```

It compiles down to one row per node:

```lua
min x_A + x_B + x_C + x_D + x_E + x_F + x_G + x_H + x_I + x_J
s.t.
        x_A + x_B + x_D + x_C + x_F + x_E >= 1
        x_B + x_D + x_E + x_J + x_C + x_A >= 1
        x_C + x_B + x_D + x_I + x_A + x_E >= 1
        x_D + x_E + x_H + x_C + x_A + x_B >= 1
        x_E + x_B + x_D + x_C + x_A + x_G >= 1
        x_F + x_J + x_G + x_A >= 1
        x_G + x_E + x_F + x_H >= 1
        x_H + x_D + x_I + x_G >= 1
        x_I + x_J + x_H + x_C >= 1
        x_J + x_F + x_I + x_B >= 1
```

The `Microlp` or `Auto` solver then finds the optimal solution, which has value `3`.

## Directly building a linear model

If you already have a linear problem in coefficient form, you can build a `LinearModel` directly and call a solver function:

```rust
use rooc::{Comparison, LinearModel, OptimizationType, VariableType, solve_real_lp_problem_clarabel};

let mut model = LinearModel::new();
model.add_variable("x1", VariableType::non_negative_real());
model.add_variable("x2", VariableType::real());
model.add_constraint(vec![1.0, 1.0], Comparison::LessOrEqual, 5.0); // x1 + x2 <= 5
model.set_objective(vec![1.0, 2.0], OptimizationType::Max);         // max x1 + 2 x2

let solution = solve_real_lp_problem_clarabel(&model).unwrap();
println!("{}", solution);
```

## License

The ROOC library is released under the **MPL-2.0** license.
