# @specy/rooc

`@specy/rooc` provides TypeScript bindings for the ROOC WebAssembly compiler and solver. Use it in a browser or JavaScript application to compile and solve ROOC source.

## Install

```bash
npm install @specy/rooc
```

## Compile and solve a model

Build a pipeline, then run it with ROOC source:

```ts
import { RoocRunnablePipe } from "@specy/rooc";
import { Pipes } from "@specy/rooc/runtime";

const source = [
    "max 3 * x + 2 * y",
    "s.t.",
    "    x + y <= 4",
    "define",
    "    x as Boolean",
    "    y as IntegerRange(0, 4)",
].join("\n");

const runner = new RoocRunnablePipe();
runner.addPipeByName(Pipes.CompilerPipe);
runner.addPipeByName(Pipes.PreModelPipe);
runner.addPipeByName(Pipes.ModelPipe);
runner.addPipeByName(Pipes.LinearModelPipe);
runner.addPipeByName(Pipes.AutoSolverPipe);

const steps = runner.run(source).unwrap();
const solution = steps.at(-1);
console.log(solution);
```

`AutoSolverPipe` selects ROOC's safe general-purpose MILP default, powered by Microlp for every supported model.

## Parse and inspect a model

Use `RoocParser` when you want to parse, format, type-check, transform, or serialize source:

```ts
import { RoocParser } from "@specy/rooc";

const source = [
    "solve",
    "s.t.",
    "    x >= 0",
    "define",
    "    x as Real",
].join("\n");

const parser = new RoocParser(source);
const preModel = parser.compile().unwrap();
const model = preModel.transform().unwrap();

console.log(model.stringify());
```

## Public imports

- `@specy/rooc` provides the TypeScript wrapper classes.
- `@specy/rooc/runtime` provides runtime metadata and pipe names.
- `@specy/rooc/pkg` provides the low-level generated WebAssembly API.

For the ROOC language, see the [language documentation](https://rooc.specy.app/docs/rooc). For the Rust API, see the [crate README](../rooc/README.md).

## License

ROOC is released under the **MPL-2.0** license.
