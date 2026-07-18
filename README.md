<div align="center">
  <h1><code>ROOC</code></h1>
  <img src='./logo-original.png' width='156px'/>
  <p><strong>Optimization modeling language</strong></p>
</div>

[![Crates.io](https://img.shields.io/crates/v/rooc.svg)](https://crates.io/crates/rooc)
[![npm](https://img.shields.io/npm/v/@specy/rooc.svg)](https://www.npmjs.com/package/@specy/rooc)

[Go to the language documentation](https://rooc.specy.app/docs/rooc)

[Go to the library documentation](https://rooc.specy.app/docs/rooc)

[Go to the rooc web modeling platform](https://rooc.specy.app/)

**ROOC** stands for the courses I took in university—*Ricerca Operativa* (Operational Research) and *Ottimizzazione Combinatoria* (Combinatorial Optimization)—which deal with solving optimization models.

## Table of Contents

- [What it is](#what-it-is)
- [Quick start: the fluent Rust API](#quick-start-the-fluent-rust-api)
  - [Variables](#variables)
  - [Constraints and objective](#constraints-and-objective)
  - [Solving and reading the solution](#solving-and-reading-the-solution)
  - [Bring your own solver](#bring-your-own-solver)
  - [Continue solving](#continue-solving)
  - [Exporting to LP format](#exporting-to-lp-format)
- [Alternative: the ROOC modeling language](#alternative-the-rooc-modeling-language)
- [Modeling examples](#modeling-examples)
  - [Dominating set (graphs)](#dominating-set-graphs)
  - [Logic constraints (vertex cover)](#logic-constraints-vertex-cover)
  - [Context-aware `abs`, `min` and `max`](#context-aware-abs-min-and-max)
- [Solvers](#solvers)
- [Using from TypeScript](#using-from-typescript)
- [Implemented features](#implemented-features)
- [License](#license)

## What it is

**ROOC** is a modeling language and library for writing optimization models, transforming them into linear models, and solving them.

There are two ways to build a model:

- The **fluent Rust API** — build the model directly in code with variables, operators and macros. Great for models you generate programmatically.
- The **ROOC modeling language** — write a formal model as a string, with functions, constants, arrays, graphs, tuples, iteration and built-in utilities (for iterating over graphs, edges, arrays, ranges, and more). Great for formal, parameterized models.

Both compile down to a linear model that is solved with the built-in solvers (or your own).

If you just want to solve a problem, use the [web platform](https://rooc.specy.app), the [Rust lib](https://crates.io/crates/rooc), or the [TypeScript lib](https://www.npmjs.com/package/@specy/rooc).

## Quick start: the fluent Rust API

A small production model with boolean and integer variables and logic constraints. The full Rust reference is in the [crate README](./packages/rooc/README.md).

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

### Variables

Declare variables with the `vars!` macro. Every domain is supported, including indexed families:

```rust
vars! { model =>
    a: bool;               // boolean (0/1)
    b: int(0, 10);         // integer in [0, 10]
    c: real;               // real, unbounded
    d: real(-5.0, 5.0);    // real in [-5, 5]
    e: nonneg;             // real >= 0
    f: nonneg(0.0, 100.0); // real in [0, 100]
    xs[5]: bool;           // an indexed family: `xs` is a Vec<Var>, use xs[i]
};

// Or through methods, when the name is computed:
let y = model.add_var("y", VariableType::integer_range(0, 3)); // -> Var
let zs = model.add_vars("z", 4, VariableType::bool());         // -> Vec<Var>
```

`xs[5]` gives you a list you index as `xs[i]`. Names must be unique; declaring the same name twice panics.

### Constraints and objective

Build constraints with `constraint!` and attach them with `with` (one) or `with_all` (many). Comparisons (`<=`, `>=`, `==`, `<`, `>`), logic assertions (`->`, `<->`) and aggregations (`any`, `all`) are all supported, and constraints can be named:

```rust
use rooc::builder::{abs, all, any, max, min, sum};

model
    .maximize(sum(xs.iter().copied()))          // maximize / minimize / satisfy, any position
    .with(constraint!(2.0 * a + b <= 10.0))     // linear comparison
    .with(constraint!(cap: a + b == 1.0))       // named constraint
    .with(constraint!(a -> b))                  // logic: a implies b
    .with(constraint!(a <-> !b))                // logic: a iff not b
    .with_all(vec![constraint!(c <= 5.0), constraint!(c >= 1.0)]);
```

The usual arithmetic and logic operators work directly (`+ - * /`, `& | ^ !`, plus `.implies()` / `.iff()`), and the helpers `sum`, `min`, `max`, `abs`, `all`, `any` live in `rooc::builder`. If you never set an objective, the model is solved for feasibility.

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

### Solving and reading the solution

`solve_with` accepts any solver. Three are built in: `Auto` selects ROOC's safe general-purpose `Microlp` MILP default for every supported model, `Microlp::new()` exposes MIP options, and `Clarabel` is selected explicitly for continuous LPs.

```rust
let solution = model
    .maximize(obj)
    .solve_with(Auto)?;

// Always available:
solution.value();              // objective value (f64)
solution.var_value(x);         // MILPValue for MILP, f64 for real
solution.numeric_value(x);     // the value as a plain f64
solution.eval(&(2.0 * x + y)); // evaluate any expression at the solution

// Available only when the solver's solution supports it:
solution.status();                // if it implements SolveStatus
solution.constraint_value("cap"); // if it implements ConstraintValues
solution.shadow_price("cap");     // if it implements DualValues
```

The built-in solvers provide `status()` and `constraint_value()`, but not duals — `shadow_price` doesn't exist on their solutions. A solver that computes duals implements `DualValues` on its solution type, and then the method appears.

### Bring your own solver

`solve_with` is the extension point: implement `Solver` for any type (even in another crate) and it plugs straight in. A solver returns its own solution type — the simplest choice is the built-in `LpSolution`; extra capabilities come from implementing the optional traits (`DualValues`, `ReducedCosts`, ...) on a solution type of your own.

```rust
use rooc::{Assignment, LinearModel, LpSolution, Solver, SolverError};

struct MySolver;

impl Solver for MySolver {
    type Solution = LpSolution<f64>; // any type implementing `Solution`

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        // Read model.variables(), .constraints(), .objective(), .domain() ...
        let assignment = model
            .variables()
            .iter()
            .map(|name| Assignment { name: name.clone(), value: 0.0 })
            .collect();
        Ok(LpSolution::new(assignment, model.objective_offset(), Default::default()))
    }
}

let solution = model
    .minimize(x)
    .solve_with(MySolver)?;
```

### Continue solving

Solvers that implement the `Reoptimizable` marker (all three built-ins do) let you edit a solved model and solve again:

```rust
use rooc::Reoptimize;

let tighter = solution
    .with(constraint!(x <= 4.0))
    .resolve()?;
```

### Exporting to LP format

Linearize and export to CPLEX LP format explicitly:

```rust
let lp_text = model
    .maximize(obj)
    .with(constraint!(cap: a + b <= 8.0))
    .linearize()?
    .to_lp_format();
```

## Alternative: the ROOC modeling language

Instead of building the model in code, write it as a formal model string, with data, sets, iteration and graphs. Here is the knapsack problem solved through the library with a MILP solver:

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

For more examples of using the Rust lib look at the [examples folder](https://github.com/Specy/rooc/tree/main/packages/rooc/examples), and for model examples [look in the ROOC docs](https://rooc.specy.app/docs/rooc/examples).

## Modeling examples

### Dominating set (graphs)

Given the formal model of the [Dominating set](https://en.wikipedia.org/wiki/Dominating_set) problem, let's model it using graphs:

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

It is compiled down to:

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

The model can then be solved using the `MILP solver` or `Auto solver` pipeline, which will solve the compiled model and find the optimal solution which has value `3` with assignment:

```
F	F	F	F	T	F	F	F	T	T
```

### Logic constraints (vertex cover)

Models can use logic operators over boolean expressions: `and` (alias `&&`), `or` (alias `||`), `not` (alias `!`), `implies` (alias `->`), `iff` (alias `<->`) and `xor`, together with the indexed aggregations `all`, `any` and `xor`. A constraint written without a comparison asserts that the logic expression must hold, and a logic expression can also be used as a 0/1 value inside arithmetic (for example to count how many conditions are satisfied).

Assertions are compiled directionally: affine clauses are emitted directly, while complex branches use only the Boolean witnesses needed to certify the asserted truth value. Logic expressions used inside arithmetic remain exact in both directions. Consequently, each edge clause in the vertex-cover model below becomes one linear row and does not require a root `$or_N` variable.

Here is the [Vertex cover](https://en.wikipedia.org/wiki/Vertex_cover) problem, where the only constraint is the logic clause "at least one endpoint of each edge must be selected":

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

### Context-aware `abs`, `min` and `max`

The absolute value is available as the `abs { }` block, e.g. `min abs { x - 5 }`. Numeric piecewise expressions inspect how their value is consumed. A favorable one-sided context uses the smaller epigraph or hypograph formulation and can preserve a pure LP. An opposite-direction or equality context uses an exact formulation with Boolean selectors.

```lua
max abs { x }
s.t.
    -10 <= x
    x <= 6
define
    x as Real
```

Here the compiler infers `x in [-10, 6]` and derives the exact selector coefficients from that interval. It never inserts an arbitrary Big-M constant. Bounds can come from declarations, affine constraints, and safe implications such as `abs { x } <= c`, `max { ... } <= c` and `min { ... } >= c`. If an exact formulation needs an endpoint that cannot be proven finite, linearization reports an error asking for bounds instead of producing an unsafe model. Sign-known absolute values and dominated `min`/`max` operands are removed without auxiliaries.

## Solvers

Currently in ROOC you can solve any linear model which can be:

- MILP (real, integer, and binary variables)
- Real only

The built-in solvers are `Microlp` (MILP), `Clarabel` (continuous LP), and `Auto`. `Auto` selects ROOC's safe general-purpose `Microlp` MILP default for every supported model; use `Microlp` directly when you need its options, or select `Clarabel` explicitly for continuous LPs. With the builder API you pass them to `solve_with`; with the language API you pass the matching solver function (`solve_milp_lp_problem`, `solve_real_lp_problem_clarabel`, `auto_solver`) to `solve_using`. You can add your own back-end by implementing the `Solver` trait.

## Using from TypeScript

ROOC is compiled to WebAssembly and published as [`@specy/rooc`](https://www.npmjs.com/package/@specy/rooc). See the [library documentation](https://rooc.specy.app/docs/rooc) for usage.

## Implemented Features
- [x] Rust builder API
  - [x] Build models in code with variables, operators and macros
  - [x] `vars!` and `constraint!` macros (including indexed variables and logic operators)
  - [x] Objective and constraints in any order (`with`, `with_all`, `maximize`, `minimize`, `satisfy`)
  - [x] Pluggable solvers via the `Solver` trait
  - [x] Solution readback: values, expression evaluation, status, constraint activity, shadow prices
  - [x] CPLEX LP export
- [x] Language
  - [x] Static block functions (min, max, avg, abs)
  - [x] Logic operators (and, or, not, implies, iff, xor) and aggregations (all, any, xor)
  - [x] Constant Graph definitions
  - [x] Iterators
  - [x] Tuples
  - [x] Iterators utility functions (for graphs, edges, etc)
  - [x] Primitive destructuring
  - [x] Constants and multi dimensional arrays in the formal definition of a problem
  - [x] Other utility functions
  - [x] Error logging and parameter validation 
  - [x] Error traces
  - [x] Primitives Operator overloading (for example, `+` for strings)
  - [x] Definition of variable bounds
  - [x] Javascript defined functions, define js functions to use in the model
- [x] Simplex resolution
  - [x] Linearization of a generic problem (min, max, abs, logic operators)
  - [x] Transformation of a linear problem into the standard form
  - [x] Two step method using artifical variables to find a valid basis for the standard form problem
  - [x] Simplex to find the optimal solution of a standard form linear problem
- [x] Integer and binary problems resolution
  - [x] Integer and binary problem definitions (bounds)
  - [x] Integer solvers
  - [x] Binary problem solution
  - [x] Integer/Binary problem solution
  - [x] MILP problem solution
  - [x] Logic constraints
- [x] UI
  - [x] Compilation to WASM
  - [x] Create and manage your models
  - [x] Automatic compilation to a LATEX block
  - [x] LSP
    - [x] Syntax errors
    - [x] Hover types
    - [x] Type errors
    - [x] Code completion
  - [x] Language documentation 
  - [x] Show the different steps of solving the problem
  - [x] List of modifications from the start of the problem to the end of the solution

# License

- The rooc library is released with the **MPL-2.0** license, as in, the folders `src` and `ts-lib`
- The frontend platform which uses the rooc library is released with the **AGPL-3.0** license, as in, the folder `packages/client`
