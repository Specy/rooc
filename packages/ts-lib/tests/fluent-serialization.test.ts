import { describe, expect, it } from "vitest";

import { ModelBuilder, abs, all, any, max, min, sum } from "../src/index";

describe("ModelBuilder serialization", () => {
    it("serializes named scalar and array variables deterministically", () => {
        const model = new ModelBuilder();
        const { enabled, quantity, choices } = model.vars({
            enabled: model.bool(),
            quantity: model.int(0, 8),
            choices: model.bool().array(2),
        });
        const temperature = model.var(
            "temperature",
            model.real(-10, 50),
        );

        model
            .maximize(
                sum(choices)
                    .add(quantity.mul(4))
                    .sub(abs(temperature)),
            )
            .with(enabled.implies(choices[0]))
            .with(sum(choices).le(quantity).named("capacity"));

        const expected = `max (((\\choices_0 + \\choices_1) + (quantity * 4)) - abs { temperature })
s.t.
    (enabled implies \\choices_0)
    capacity: (\\choices_0 + \\choices_1) <= quantity
define
    enabled as Boolean
    quantity as IntegerRange(0, 8)
    \\choices_0 as Boolean
    \\choices_1 as Boolean
    temperature as Real(-10, 50)`;

        expect(model.toRooc()).toBe(expected);
        expect(model.toRooc()).toBe(model.toRooc());
    });

    it("serializes block functions and the full operator surface", () => {
        const blocks = new ModelBuilder();
        const { a, b, x, y } = blocks.vars({
            a: blocks.bool(),
            b: blocks.bool(),
            x: blocks.real(-2, 2),
            y: blocks.real(-2, 2),
        });

        blocks
            .minimize(max([abs(x), min([x, y])]))
            .with(all([a, b]).or(any([a.not(), b])));

        expect(blocks.toRooc()).toContain(
            "max { abs { x }, min { x, y } }",
        );

        const operators = new ModelBuilder();
        const operatorVars = operators.vars({
            x: operators.real(-5, 5),
            y: operators.real(-5, 5),
        });
        operators
            .minimize(operatorVars.x.div(2).add(operatorVars.y.neg()))
            .withAll([
                operatorVars.x.eq(operatorVars.y).named("equal"),
                operatorVars.x.lt(4).named("strictUpper"),
                operatorVars.x.ge(-4).named("lower"),
                operatorVars.y.gt(-5).named("strictLower"),
            ]);

        expect(operators.toRooc()).toContain("min ((x / 2) + (-y))");
        expect(operators.toRooc()).toContain("equal: x = y");
        expect(operators.toRooc()).toContain("strictUpper: x < 4");
        expect(operators.toRooc()).toContain("lower: x >= -4");
        expect(operators.toRooc()).toContain("strictLower: y > -5");

        operators.maximize(operatorVars.x).satisfy();
        expect(operators.toRooc()).toMatch(/^solve\ns\.t\./);
    });

    it("uses parser-safe identities and keeps otherwise unused variables", () => {
        const identities = new ModelBuilder();
        identities
            .maximize(sum([]))
            .with(all([]), "emptyAll")
            .with(any([]).not(), "emptyAny");

        expect(identities.toRooc()).toContain("max 0");
        expect(identities.toRooc()).toContain("emptyAll: true");
        expect(identities.toRooc()).toContain("emptyAny: (not false)");

        const unused = new ModelBuilder();
        unused.vars({
            lone: unused.bool(),
            group: unused.int(0, 3).array(2),
        });

        expect(unused.toRooc()).toMatch(/^solve\ns\.t\./);
        expect(unused.toRooc()).toContain("$__rooc_ts_keep:");
    });

    it("formats finite numbers and every supported domain without exponents", () => {
        const scientific = new ModelBuilder();
        const tiny = scientific.var("tiny_value", scientific.real(0, 1));
        scientific.maximize(tiny.mul(1e-7)).with(tiny.le(1e21));

        expect(scientific.toRooc()).toContain("0.0000001");
        expect(scientific.toRooc()).toContain("1000000000000000000000");

        const scientificDomain = new ModelBuilder();
        const domainValue = scientificDomain.var(
            "domain_value",
            scientificDomain.real(1e-7, 1e21),
        );
        scientificDomain.maximize(domainValue);

        expect(scientificDomain.toRooc()).toContain(
            "Real(0.0000001, 1000000000000000000000)",
        );
        expect(scientificDomain.toRooc()).toContain(
            "$__rooc_ts_keep: 0 * \\domain_value = 0",
        );

        const everyDomain = new ModelBuilder();
        everyDomain.vars({
            truth: everyDomain.bool(),
            count: everyDomain.int(-2, 3),
            free: everyDomain.real(),
            bounded: everyDomain.real(-1.5, 2.5),
            positive: everyDomain.nonNegative(),
            positiveBounded: everyDomain.nonNegative(0.25, 4.5),
        });

        for (const domain of [
            "truth as Boolean",
            "count as IntegerRange(-2, 3)",
            "free as Real",
            "bounded as Real(-1.5, 2.5)",
            "positive as NonNegativeReal",
            "positiveBounded as NonNegativeReal(0.25, 4.5)",
        ]) {
            expect(everyDomain.toRooc()).toContain(domain);
        }
    });

    it("preserves special own-property keys returned by vars", () => {
        const model = new ModelBuilder();
        const definitions = Object.create(null) as Record<
            string,
            ReturnType<ModelBuilder["bool"]>
        >;
        definitions.__proto__ = model.bool();

        const variables = model.vars(definitions);

        expect(
            Object.prototype.hasOwnProperty.call(variables, "__proto__"),
        ).toBe(true);
    });

    it("rejects invalid names, definitions, domains, and expressions", () => {
        const model = new ModelBuilder();
        const { enabled, quantity } = model.vars({
            enabled: model.bool(),
            quantity: model.int(0, 8),
        });

        expect(() => model.var("max", model.bool())).toThrow("reserved");
        expect(() => model.var("abs", model.bool())).toThrow("reserved");
        expect(() => model.var("E", model.bool())).toThrow("reserved");
        expect(() => model.var("Infinity", model.bool())).toThrow("reserved");
        expect(() => model.var("$__rooc_ts_user", model.bool())).toThrow(
            "reserved",
        );
        expect(() => model.var("enabled", model.bool())).toThrow(
            "already exists",
        );
        expect(() => model.bool().array(-1)).toThrow("array");
        expect(() => model.bool().array(Number.NaN)).toThrow("array");
        expect(() =>
            (
                model.bool().array(1) as unknown as {
                    array(count: number): unknown;
                }
            ).array(2),
        ).toThrow("nested");
        expect(() => model.var("invalidDefinition", {} as never)).toThrow(
            "definition",
        );
        expect(() => model.int(4, 2)).toThrow("bounds");
        expect(() => model.int(0, 2.5)).toThrow("safe integers");
        expect(() => new ModelBuilder().real(0, Number.POSITIVE_INFINITY)).toThrow(
            "finite",
        );
        expect(() => new ModelBuilder().nonNegative(-1, 2)).toThrow(
            "non-negative",
        );
        expect(() => min([])).toThrow("empty");
        expect(() => max([])).toThrow("empty");
        expect(() => model.with(quantity.div(0).le(1))).toThrow("zero");
        expect(() => quantity.add(Number.NaN)).toThrow("finite");

        const other = new ModelBuilder();
        const foreign = other.var("foreign", other.bool());
        expect(() => model.with(enabled.and(foreign))).toThrow(
            "different model",
        );
        expect(() => other.maximize(quantity)).toThrow("different model");
    });

    it("rejects duplicate constraint names", () => {
        const model = new ModelBuilder();
        const x = model.var("x", model.real(0, 1));
        model.with(x.le(1).named("cap"));

        expect(() => model.with(x.ge(0).named("cap"))).toThrow(
            "already exists",
        );
    });
});
