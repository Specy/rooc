import { describe, expect, it } from "vitest";

import { RoocParser } from "../src/index";
import { Pipes, WasmPipeRunner } from "../src/pkg/rooc";

describe("TypeScript bindings", () => {
    it("runs the compiler and solver pipeline under Node", () => {
        const source = `
//maximize the value of the bag
max sum((value, i) in enumerate(values)) { value * x_i }
s.t.
    //make sure that the selected items do not go over the bag's capacity
    sum((weight, i) in enumerate(weights)) { weight * x_i } <= capacity
where
    let weights = [10, 60, 30, 40, 30, 20, 20, 2]
    let values = [1, 10, 15, 40, 60, 90, 100, 15]
    let capacity = 102
define
    x_i as Boolean for i in 0..len(weights)
`;
        const pipes = [
            Pipes.CompilerPipe,
            Pipes.PreModelPipe,
            Pipes.ModelPipe,
            Pipes.LinearModelPipe,
            Pipes.MILPSolverPipe,
        ];

        const runner = WasmPipeRunner.new_wasm();
        for (const pipe of pipes) {
            runner.add_step_by_name(pipe);
        }
        const result = runner.wasm_run_from_string(source, [], []);

        expect(result).toHaveLength(pipes.length + 1);
        expect(result.at(-1)).toBeDefined();
    });

    it("serializes logic and abs through both compilation stages", () => {
        const source = `
min abs { x }
s.t.
    a or b
    b = true
define
    x as Real(-1, 1)
    a, b as Boolean
`;
        const parser = new RoocParser(source);
        const preModel = parser.compile().unwrap().serialize();
        const serializedAbs = preModel.objective.rhs;
        const serializedOr = preModel.constraints[0].lhs;

        expect(serializedAbs.type).toBe("BlockFunction");
        if (serializedAbs.type !== "BlockFunction") {
            throw new Error("expected a serialized block function");
        }
        expect(serializedAbs.value.value.kind.type).toBe("Abs");
        expect(serializedOr.type).toBe("BinaryOperation");
        if (serializedOr.type !== "BinaryOperation") {
            throw new Error("expected a serialized binary operation");
        }
        expect(serializedOr.value[0].value.type).toBe("Or");
        expect(preModel.constraints[0].is_logic_assertion).toBe(true);
        expect(preModel.constraints[1].is_logic_assertion).toBe(false);

        const transformed = parser.compileAndTransform().unwrap().serialize();

        expect(transformed.objective.rhs.type).toBe("Abs");
        expect(transformed.constraints[0].lhs.type).toBe("Or");
        expect(transformed.constraints[0].is_logic_assertion).toBe(true);
        expect(transformed.constraints[1].is_logic_assertion).toBe(false);
    });
});
