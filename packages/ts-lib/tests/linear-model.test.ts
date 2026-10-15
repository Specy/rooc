import { describe, expect, it } from "vitest";

import { RoocRunnablePipe } from "../src/index";
import { PipeDataType, Pipes, type SerializedLinearModel } from "../src/pkg/rooc";

const source = `
max 3x + 2y + 4z
s.t.
    x + z <= 4
    y + z <= 5
define
    x, y, z as NonNegativeReal
`;

describe("Linear models", () => {
    it("store only the nonzero coefficients of each constraint", () => {
        const pipe = new RoocRunnablePipe();
        for (const step of [Pipes.CompilerPipe, Pipes.PreModelPipe, Pipes.ModelPipe, Pipes.LinearModelPipe]) {
            pipe.addPipeByName(step);
        }
        const linear = pipe.run(source).unwrap().at(-1);
        if (linear?.type !== PipeDataType.LinearModel) {
            throw new Error("expected a linear model");
        }

        expect(linear.data.getVariables()).toEqual(["x", "y", "z"]);
        const [first, second] = linear.data.getConstraints();
        expect(first.getIndices()).toEqual([0, 2]);
        expect(first.getValues()).toEqual([1, 1]);
        expect(first.getCoefficients()).toEqual([1, 0, 1]);
        expect(second.getIndices()).toEqual([1, 2]);
        expect(second.getCoefficients()).toEqual([0, 1, 1]);
    });

    it("cross into JavaScript pipes as sparse rows and come back", () => {
        const pipe = new RoocRunnablePipe();
        for (const step of [Pipes.CompilerPipe, Pipes.PreModelPipe, Pipes.ModelPipe, Pipes.LinearModelPipe]) {
            pipe.addPipeByName(step);
        }
        let serialized: SerializedLinearModel | undefined;
        pipe.addPipe((data) => {
            if (data.type !== "LinearModel") {
                throw new Error("expected a linear model");
            }
            serialized = data.value;
            return data;
        });
        pipe.addPipeByName(Pipes.MILPSolverPipe);
        const solution = pipe.run(source).unwrap().at(-1);

        expect(serialized?.constraints.map((c) => [c.indices, c.values])).toEqual([
            [[0, 2], [1, 1]],
            [[1, 2], [1, 1]],
        ]);
        expect(serialized?.constraints[0]).not.toHaveProperty("coefficients");
        // the model handed back by the pipe is read again by the solver
        if (solution?.type !== PipeDataType.MILPSolution) {
            throw new Error("expected a solution");
        }
        expect(solution.data.value).toBeCloseTo(22);
    });
});
