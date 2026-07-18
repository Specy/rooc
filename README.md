<div align="center">
  <h1><code>ROOC</code></h1>
  <img src="./logo-original.png" width="156px" alt="ROOC logo" />
  <p><strong>Optimization modeling language and solver</strong></p>
</div>

[![Crates.io](https://img.shields.io/crates/v/rooc.svg)](https://crates.io/crates/rooc)
[![npm](https://img.shields.io/npm/v/@specy/rooc.svg)](https://www.npmjs.com/package/@specy/rooc)

[Language documentation](https://rooc.specy.app/docs/rooc) · [Web platform](https://rooc.specy.app/) · [Rust crate](./packages/rooc/README.md) · [TypeScript package](./packages/ts-lib/README.md)

ROOC is a library and modeling language for linear and mixed-integer optimization. Define a model in Rust or in ROOC source, then solve it with the built-in solvers.

## Choose an interface

- Use the [fluent Rust API](./packages/rooc/README.md) when your application constructs models in code.
- Use the [fluent TypeScript API](./packages/ts-lib/README.md) for typed browser and JavaScript models backed by the WebAssembly solvers.
- Use the ROOC language when a formal, data-driven model is easier to read and maintain.
- Use the [web platform](https://rooc.specy.app/) to write and run ROOC models in the browser.

## Rust quick start

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

The [crate README](./packages/rooc/README.md) covers variables, expressions, constraints, LP export, and direct linear models.

## Choose a solver

Use `Auto` for ROOC's safe general-purpose MILP default. It uses Microlp for every supported model. Use `Microlp::new()` when you need MIP options such as a time limit or mip gap. Select `Clarabel` explicitly for a continuous model.

## ROOC language

ROOC source is useful for models built from data, sets, graphs, and iteration:

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

The language supports arithmetic, boolean logic, indexed constraints, collections, graphs, and the `abs { }`, `min { }`, and `max { }` blocks. See the [language documentation](https://rooc.specy.app/docs/rooc) for the complete syntax and examples.

## TypeScript and web

`@specy/rooc` provides a type-safe fluent builder over the existing WebAssembly
compiler, linearizer, and solvers:

```ts
import { ModelBuilder, sum } from "@specy/rooc";

const model = new ModelBuilder();
const { selected, capacity } = model.vars({
    selected: model.bool().array(4),
    capacity: model.int(0, 4),
});

const solution = model
    .maximize(sum(selected))
    .with(sum(selected).le(capacity))
    .solve();

console.log(solution.valuesOf({ selected, capacity }));
```

The source compiler and pipe-by-pipe APIs remain available for formal ROOC
programs and intermediate pipeline inspection. See the
[TypeScript README](./packages/ts-lib/README.md) for variables, expressions,
solver selection, typed readback, generated source, and compatibility imports.

## License

The Rust library is released under [MPL-2.0](./packages/rooc/Cargo.toml). The web client is released under AGPL-3.0.
