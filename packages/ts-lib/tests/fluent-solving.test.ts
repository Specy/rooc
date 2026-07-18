import { describe, expect, it } from "vitest";

import {
    ModelBuilder,
    ModelBuilderError,
    abs,
    all,
    any,
    max,
    min,
    sum,
} from "../src/index";

function getModelBuilderError(run: () => unknown): ModelBuilderError {
    let thrown: unknown;
    try {
        run();
    } catch (error) {
        thrown = error;
    }

    expect(thrown).toBeInstanceOf(ModelBuilderError);
    return thrown as ModelBuilderError;
}

describe("ModelBuilder solving", () => {
    it("solves a mixed-integer model and reads typed values", () => {
        const model = new ModelBuilder();
        const variables = model.vars({
            a: model.bool(),
            b: model.bool(),
            c: model.bool(),
            material: model.int(0, 8),
        });
        const solution = model
            .maximize(
                variables.a
                    .mul(6)
                    .add(variables.b.mul(5))
                    .add(variables.c.mul(4))
                    .sub(variables.material),
            )
            .with(
                variables.a
                    .mul(2)
                    .add(variables.b.mul(3))
                    .add(variables.c)
                    .le(variables.material)
                    .named("material_cap"),
            )
            .with(variables.a.implies(variables.b))
            .with(any([variables.a, variables.c]))
            .solve();

        expect(typeof solution.valueOf(variables.a)).toBe("boolean");
        expect(typeof solution.valueOf(variables.material)).toBe("number");
        expect(solution.constraintValue("material_cap")).toBeDefined();
        expect(solution.eval(variables.a.implies(variables.b))).toBe(true);
        expect(solution.eval(sum([]))).toBe(0);
        expect(solution.eval(all([]))).toBe(true);
        expect(solution.eval(any([]))).toBe(false);

        const nested = solution.valuesOf({
            chosen: [variables.a, variables.b, variables.c],
            details: { material: variables.material },
        });
        expect(nested.chosen).toHaveLength(3);
        expect(nested.details.material).toBe(
            solution.valueOf(variables.material),
        );
    });

    it("solves Boolean operators through the microlp backend", () => {
        const model = new ModelBuilder();
        const variables = model.vars({
            a: model.bool(),
            b: model.bool(),
            c: model.bool(),
            d: model.bool(),
        });
        const solution = model
            .maximize(sum([variables.a, variables.b, variables.c, variables.d]))
            .with(variables.a.xor(variables.b))
            .with(variables.a.implies(variables.c))
            .with(variables.c.iff(variables.d))
            .with(variables.a.not().and(variables.b))
            .with(
                all([
                    variables.a.or(variables.b),
                    any([variables.c, variables.d]),
                ]),
            )
            .solve({ solver: "microlp" });

        expect(solution.eval(variables.a.xor(variables.b))).toBe(true);
    });

    it("solves piecewise and continuous expressions", () => {
        const piecewise = new ModelBuilder();
        const piecewiseVariables = piecewise.vars({
            x: piecewise.real(-3, 3),
            y: piecewise.real(-2, 2),
        });
        const piecewiseSolution = piecewise
            .maximize(
                min([
                    abs(piecewiseVariables.x),
                    max([piecewiseVariables.x, piecewiseVariables.y]),
                ]),
            )
            .with(piecewiseVariables.x.le(2))
            .with(piecewiseVariables.y.le(1))
            .solve();

        expect(Number.isFinite(piecewiseSolution.value())).toBe(true);
        expect(piecewiseSolution.value()).toBeCloseTo(2, 6);

        const continuous = new ModelBuilder();
        const continuousVariables = continuous.vars({
            x: continuous.nonNegative(),
            y: continuous.nonNegative(),
        });
        const continuousSolution = continuous
            .maximize(
                continuousVariables.x.add(continuousVariables.y.mul(2)),
            )
            .with(continuousVariables.x.add(continuousVariables.y).le(10))
            .solve({ solver: "clarabel" });

        expect(continuousSolution.value()).toBeCloseTo(20, 6);
        expect(
            continuousSolution.valueOf(continuousVariables.y),
        ).toBeCloseTo(10, 6);
    });

    it("keeps unused variables in feasibility models but hides internals", () => {
        const model = new ModelBuilder();
        const variables = model.vars({
            used: model.int(4, 7),
            unused: model.bool(),
            unusedGroup: model.bool().array(2),
        });
        const solution = model.solve();
        const selected = solution.valuesOf(variables);

        expect(selected.used).toBeGreaterThanOrEqual(4);
        expect(selected.used).toBeLessThanOrEqual(7);
        expect(typeof selected.unused).toBe("boolean");
        expect(selected.unusedGroup).toHaveLength(2);
        expect(solution.constraintValue("$__rooc_ts_keep")).toBeUndefined();
    });

    it("solves objective-only and escaped-name models without mutation", () => {
        const objectiveOnly = new ModelBuilder();
        const x = objectiveOnly.var("x", objectiveOnly.real(0, 1));
        objectiveOnly.maximize(x);
        const source = objectiveOnly.toRooc();

        expect(objectiveOnly.solve().valueOf(x)).toBeCloseTo(1, 6);
        expect(objectiveOnly.toRooc()).toBe(source);

        const compoundNames = new ModelBuilder();
        const compound = compoundNames.var(
            "$choice_value",
            compoundNames.int(2, 2),
        );
        expect(compoundNames.solve().valueOf(compound)).toBe(2);
    });

    it("rejects expressions from another model during readback", () => {
        const solvedModel = new ModelBuilder();
        const own = solvedModel.var("own", solvedModel.bool());
        const solution = solvedModel.maximize(own).solve();
        const foreignModel = new ModelBuilder();
        const foreign = foreignModel.var("foreign", foreignModel.bool());

        expect(getModelBuilderError(() => solution.valueOf(foreign)).stage).toBe(
            "construction",
        );
        expect(getModelBuilderError(() => solution.eval(foreign.not())).stage).toBe(
            "construction",
        );
    });

    it("retains source and pipeline context for linearization failures", () => {
        const model = new ModelBuilder();
        const unbounded = model.var("x", model.real());
        model.maximize(abs(unbounded));
        const source = model.toRooc();

        const error = getModelBuilderError(() => model.solve());

        expect(error.stage).toBe("linearization");
        expect(error.source).toBe(source);
        expect(error.context?.length).toBeGreaterThanOrEqual(4);
    });

    it("reports backend and unknown-solver failures at the right stage", () => {
        const discreteClarabel = new ModelBuilder();
        const discrete = discreteClarabel.var(
            "discrete",
            discreteClarabel.bool(),
        );
        discreteClarabel.maximize(discrete).with(discrete.eq(1));

        const solverError = getModelBuilderError(() =>
            discreteClarabel.solve({ solver: "clarabel" }),
        );
        expect(solverError.stage).toBe("solving");
        expect(solverError.source).toBe(discreteClarabel.toRooc());

        const unknownSolver = new ModelBuilder();
        const unknownSolverError = getModelBuilderError(() =>
            unknownSolver.solve({ solver: "unknown" as "auto" }),
        );
        expect(unknownSolverError.stage).toBe("construction");
    });
});
