import { ModelBuilder } from "../src/index";

type Equal<A, B> =
    (<T>() => T extends A ? 1 : 2) extends
    (<T>() => T extends B ? 1 : 2) ? true : false;
type Expect<T extends true> = T;

const model = new ModelBuilder();
const variables = model.vars({
    enabled: model.bool(),
    quantity: model.int(0, 8),
    choices: model.bool().array(3),
});
const solution = model.maximize(variables.quantity).solve();

type BoolValue = Expect<
    Equal<
        ReturnType<typeof solution.valueOf<typeof variables.enabled>>,
        boolean
    >
>;
type NumericValue = Expect<
    Equal<
        ReturnType<typeof solution.valueOf<typeof variables.quantity>>,
        number
    >
>;

const mapped = solution.valuesOf(variables);
type MappedValues = Expect<
    Equal<
        typeof mapped,
        {
            enabled: boolean;
            quantity: number;
            choices: boolean[];
        }
    >
>;

const numericEval: number = solution.eval(
    variables.quantity.add(variables.enabled),
);
const booleanEval: boolean = solution.eval(variables.enabled.not());

void numericEval;
void booleanEval;
type _Assertions = BoolValue | NumericValue | MappedValues;
