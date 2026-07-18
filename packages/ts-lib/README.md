# @specy/rooc

`@specy/rooc` provides the ROOC compiler and solvers as a WebAssembly-backed
TypeScript library. Build models with the type-safe fluent API, or keep using
ROOC source and the lower-level compiler and pipe APIs when you need the full
language.

## Install

```bash
npm install @specy/rooc
```

## Fluent model builder

Declare variables, build expressions, and solve without assembling ROOC source
strings:

```ts
import { ModelBuilder, abs, sum } from "@specy/rooc";

const model = new ModelBuilder();

const { enabled, quantity } = model.vars({
    enabled: model.bool(),
    quantity: model.int(0, 8),
});
const choices = model.var("choice", model.bool().array(4));
const temperature = model.var("temperature", model.real(-10, 50));

const solution = model
    .maximize(
        sum(choices)
            .add(quantity.mul(4))
            .sub(abs(temperature)),
    )
    .with(enabled.implies(choices[0]), "enabledRequiresFirst")
    .with(sum(choices).le(quantity).named("capacity"))
    .solve();

const enabledValue = solution.valueOf(enabled);     // boolean
const quantityValue = solution.valueOf(quantity);   // number
const choiceValues = solution.valuesOf(choices);    // boolean[]
const values = solution.valuesOf({
    enabled,
    quantity,
    choices,
});
// { enabled: boolean; quantity: number; choices: boolean[] }
```

`ModelBuilder` keeps a typed expression tree in TypeScript, serializes the
complete model internally, and sends it through the existing ROOC parser,
linearizer, and WebAssembly solver pipeline.

## Variables and arrays

The available domain descriptors are:

```ts
model.bool();
model.int(-5, 20);
model.real();
model.real(-10, 10);
model.nonNegative();
model.nonNegative(0, 100);
```

Declare a named scalar or array with `var()`:

```ts
const active = model.var("active", model.bool());
const levels = model.var("level", model.int(0, 5).array(3));
```

Or declare several values at once with `vars()`. Its return type preserves the
input keys, scalar types, and array element types:

```ts
const variables = model.vars({
    active: model.bool(),             // BoolVar
    level: model.int(0, 5),           // NumericVar
    selected: model.bool().array(8),  // BoolVar[]
    weights: model.real(0, 1).array(3), // NumericVar[]
});
```

Array members receive stable names such as `selected_0`, `selected_1`, and so
on. Invalid bounds, invalid counts, duplicate names, reserved names, and nested
arrays throw `ModelBuilderError` immediately.

## Expressions and constraints

Numeric expressions support:

```ts
const score = variables.level
    .add(3)
    .sub(variables.active)
    .mul(2)
    .div(4)
    .neg();

model.with(score.le(10));
model.with(score.ge(0).named("scoreRange"));
```

Use `.eq()`, `.le()`, `.lt()`, `.ge()`, or `.gt()` to create a constraint.
Multiplication and division accept finite numeric constants only, keeping
nonlinear products and variable denominators out of the fluent surface.

Boolean variables are also numeric 0/1 expressions, so they can appear in
arithmetic. Only Boolean expressions expose `.and()`, `.or()`, `.xor()`,
`.implies()`, `.iff()`, and `.not()`. TypeScript therefore rejects a numeric
expression passed to Boolean-only logic.

## Block and logic helpers

The fluent package exports the ROOC block helpers:

```ts
import { abs, all, any, max, min, sum } from "@specy/rooc";

const magnitude = abs(score);
const smallest = min([variables.level, 3]);
const largest = max([variables.level, 3]);
const total = sum(variables.selected);
const everySelected = all(variables.selected);
const oneSelected = any(variables.selected);

model
    .with(everySelected.implies(variables.active))
    .with(oneSelected);
```

`sum([])` is `0`, `all([])` is `true`, and `any([])` is `false`. Empty `min`
and `max` inputs are invalid.

## Solving and reading values

`solve()` is synchronous. Auto is the default general-purpose solver;
Microlp can be selected explicitly for mixed-integer models, and Clarabel for
continuous models:

```ts
const automatic = model.solve();
const mixedInteger = model.solve({ solver: "microlp" });

const continuousModel = new ModelBuilder();
const x = continuousModel.var("x", continuousModel.nonNegative());
const continuous = continuousModel
    .maximize(x)
    .with(x.le(10))
    .solve({ solver: "clarabel" });
```

Each call starts a fresh solve from the builder's current state. A solution
provides:

```ts
solution.value();
solution.valueOf(variables.active);       // boolean
solution.valueOf(variables.level);        // number
solution.valuesOf(variables.selected);    // boolean[]
solution.valuesOf(variables);             // recursively mapped object
solution.eval(score);                     // number
solution.eval(variables.active.not());    // boolean
solution.constraintValue("scoreRange");  // number | undefined
```

Handles and expressions belong to their creating model. Combining models or
reading a foreign handle throws. Compilation, linearization, and solver
failures throw `ModelBuilderError`, whose `stage`, `source`, `cause`, and
`context` fields preserve the generated model and available pipeline details.

## Export generated ROOC

Use `toRooc()` to inspect, log, save, or run the exact source generated from the
typed model:

```ts
const generatedSource = model.toRooc();
console.log(generatedSource);
```

Serialization is deterministic. Literal compound names are escaped for ROOC,
and an internal zero-activity constraint keeps unused declarations visible to
the solver and makes objective-only models parser-compatible.

## Compile ROOC source directly

The existing source API remains available for formal models, graph and set
iterations, dynamically authored programs, formatting, and intermediate model
inspection:

```ts
import { RoocParser } from "@specy/rooc";

const source = `
max 3 * x + 2 * y
s.t.
    x + y <= 4
define
    x as Boolean
    y as IntegerRange(0, 4)
`;

const parser = new RoocParser(source);
const preModel = parser.compile().unwrap();
const transformed = preModel.transform().unwrap();

console.log(transformed.stringify());
```

`RoocParser` also exposes formatting, type information, serialization, and
direct compile-and-transform operations.

## Advanced pipe API

Use `RoocRunnablePipe` when an application needs every intermediate pipeline
result or a custom JavaScript pipe step:

```ts
import { RoocRunnablePipe } from "@specy/rooc";
import { Pipes } from "@specy/rooc/runtime";

const source = `
max 3 * x + 2 * y
s.t.
    x + y <= 4
define
    x as Boolean
    y as IntegerRange(0, 4)
`;

const runner = new RoocRunnablePipe();
runner.addPipeByName(Pipes.CompilerPipe);
runner.addPipeByName(Pipes.PreModelPipe);
runner.addPipeByName(Pipes.ModelPipe);
runner.addPipeByName(Pipes.LinearModelPipe);
runner.addPipeByName(Pipes.AutoSolverPipe);

const steps = runner.run(source).unwrap();
const solvedStep = steps[steps.length - 1];
console.log(solvedStep);
```

This API and its existing result shapes are unchanged.

## Public imports

- `@specy/rooc` exports the fluent builder, parser, model wrappers, runnable
  pipes, serialized types, and all existing top-level APIs.
- `@specy/rooc/runtime` exports runtime metadata and pipe names.
- `@specy/rooc/pkg` exports the low-level generated WebAssembly bindings.
- `@specy/rooc/src/runtime` remains available as a compatibility alias for the
  runtime entry point.

For the language itself, see the [ROOC language documentation](https://rooc.specy.app/docs/rooc).
For the Rust API, see the [crate README](../rooc/README.md).

## License

ROOC is released under the **MPL-2.0** license.
