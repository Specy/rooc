import {
    BoolVar,
    BooleanExpression,
    Constraint,
    ModelBuilder,
    NumericExpression,
    NumericVar,
    VariableDefinition,
    all,
    any,
    sum,
} from "../src/index";

type Equal<A, B> =
    (<T>() => T extends A ? 1 : 2) extends
    (<T>() => T extends B ? 1 : 2) ? true : false;
type Expect<T extends true> = T;

const model = new ModelBuilder();
const declared = model.vars({
    enabled: model.bool(),
    quantity: model.int(0, 8),
    choices: model.bool().array(4),
    weights: model.real(0, 1).array(3),
});
const temperature = model.var("temperature", model.real(-10, 50));

type EnabledIsBool = Expect<Equal<typeof declared.enabled, BoolVar>>;
type QuantityIsNumeric = Expect<Equal<typeof declared.quantity, NumericVar>>;
type ChoicesAreBool = Expect<Equal<typeof declared.choices, BoolVar[]>>;
type WeightsAreNumeric = Expect<Equal<typeof declared.weights, NumericVar[]>>;
type TemperatureIsNumeric = Expect<Equal<typeof temperature, NumericVar>>;
type BoolDefinitionIsScalar = Expect<
    Equal<ReturnType<typeof model.bool>, VariableDefinition<"boolean", false>>
>;
type BoolArrayDefinition = Expect<
    Equal<
        ReturnType<ReturnType<typeof model.bool>["array"]>,
        VariableDefinition<"boolean", true>
    >
>;

const arithmetic: NumericExpression = declared.enabled
    .add(declared.quantity)
    .sub(temperature)
    .mul(2)
    .div(4)
    .neg();
const logic: BooleanExpression = declared.enabled
    .and(declared.choices[0])
    .or(false)
    .xor(declared.choices[1])
    .implies(declared.choices[2])
    .iff(true)
    .not();
const comparison: Constraint = arithmetic.le(sum(declared.weights));
const booleanSum: NumericExpression = sum([declared.enabled, declared.quantity]);
const conjunction: BooleanExpression = all(declared.choices);
const disjunction: BooleanExpression = any(declared.choices);

model.with(logic).with(comparison).maximize(booleanSum);

// @ts-expect-error numeric expressions cannot use Boolean operators
declared.quantity.and(declared.enabled);
// @ts-expect-error all accepts Boolean inputs only
all([declared.enabled, declared.quantity]);
// @ts-expect-error any accepts Boolean inputs only
any([declared.quantity]);
// @ts-expect-error bare numeric expressions are not constraints
model.with(declared.quantity);
// @ts-expect-error multiplication is limited to numeric constants
declared.quantity.mul(temperature);
// @ts-expect-error array descriptors cannot be nested
model.bool().array(2).array(3);

void arithmetic;
void conjunction;
void disjunction;
type _Assertions =
    | EnabledIsBool
    | QuantityIsNumeric
    | ChoicesAreBool
    | WeightsAreNumeric
    | TemperatureIsNumeric
    | BoolDefinitionIsScalar
    | BoolArrayDefinition;
