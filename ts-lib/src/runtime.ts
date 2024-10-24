import type {SerializedPrimitiveKind, SerializedPrimitive} from "./pkg/rooc";
import Fuse from 'fuse.js'

export const PrimitiveKind = {
    Number: {type: 'Number'},
    Integer: {type: 'Integer'},
    PositiveInteger: {type: 'PositiveInteger'},
    String: {type: 'String'},
    Iterable: (value: SerializedPrimitiveKind) => ({type: 'Iterable', value}),
    Graph: {type: 'Graph'},
    GraphEdge: {type: 'GraphEdge'},
    GraphNode: {type: 'GraphNode'},
    Tuple: (value: SerializedPrimitiveKind[]) => ({type: 'Tuple', value}),
    Boolean: {type: 'Boolean'},
    Undefined: {type: 'Undefined'},
    Any: {type: 'Any'},
} satisfies Record<
    SerializedPrimitiveKind['type'],
    SerializedPrimitiveKind | ((value: SerializedPrimitiveKind | SerializedPrimitiveKind[]) => SerializedPrimitiveKind)
>

export type ExtractArgTypes<T extends [string, SerializedPrimitiveKind][]> = {
    [K in keyof T]: T[K] extends [string, infer Type extends SerializedPrimitiveKind] ? SerializedPrimitive & {
        type: Type['type']
    } : never;
};




//TODO remember to put this back in whenever they are updated, the runtime is supposed to not import anything from the compiler
export enum PipeDataType {
    String = 0,
    Parser = 1,
    PreModel = 2,
    Model = 3,
    LinearModel = 4,
    StandardLinearModel = 5,
    Tableau = 6,
    OptimalTableau = 7,
    OptimalTableauWithSteps = 8,
    BinarySolution = 9,
    IntegerBinarySolution = 10,
}

export enum Pipes {
    CompilerPipe = 0,
    PreModelPipe = 1,
    ModelPipe = 2,
    LinearModelPipe = 3,
    StandardLinearModelPipe = 4,
    TableauPipe = 5,
    SimplexPipe = 6,
    StepByStepSimplexPipe = 7,
    BinarySolverPipe = 8,
    IntegerBinarySolverPipe = 9,
}

export type NamedParameter = {
    name: string;
    value: SerializedPrimitiveKind;
}
export type RuntimeFunction<T extends NamedParameter[], R extends SerializedPrimitiveKind> = {
    name: string;
    description?: string;
    type: "RuntimeFunction";
    parameters: T;
    returns: R;
}

export function makeRuntimeFunction<T extends NamedParameter[], R extends SerializedPrimitiveKind>(name: string, parameters: T, returns: R, description?: string): RuntimeFunction<T, R> {
    return {name, parameters, returns, type: "RuntimeFunction", description};
}


export const FN_lenOfIterable = makeRuntimeFunction("len", [
        {
            name: "of_iterable",
            value: {
                type: "Iterable",
                value: {
                    type: "Any"
                }
            }
        },
    ],
    {type: "PositiveInteger"},
    "Returns the length of the iterable"
)
export const FN_enumerateArray = makeRuntimeFunction("enumerate", [
        {
            name: "of_iterable",
            value: {
                type: "Iterable",
                value: {
                    type: "Any"
                }
            }
        },
    ],
    {
        type: "Iterable",
        value: {
            type: "Tuple",
            value: [{type: "Any"}, {type: "PositiveInteger"}]
        }
    },
    "Enumerates the iterable, returning a tuple of the element and its index"
)

export const FN_edges = makeRuntimeFunction("edges", [
        {name: "of_graph", value: {type: "Graph"}}
    ],
    {type: "Iterable", value: {type: "GraphEdge"}},
    "Returns the edges of a graph"
)

export const FN_nodes = makeRuntimeFunction("nodes", [
        {name: "of_graph", value: {type: "Graph"}}
    ],
    {type: "Iterable", value: {type: "GraphNode"}},
    "Returns the nodes of a graph"
)
export const FN_neigh_edges = makeRuntimeFunction("neigh_edges", [
        {name: "of_node", value: {type: "GraphNode"}},
    ],
    {type: "Iterable", value: {type: "GraphEdge"}},
    "Returns the neighbour edges of a node"
)

export const FN_neigh_edges_of = makeRuntimeFunction("neigh_edges_of", [
        {name: "of_node_name", value: {type: "String"}},
        {name: "in_graph", value: {type: "Graph"}},
    ],
    {type: "Iterable", value: {type: "GraphEdge"}},
    "Returns the neighbour edges of a node name in a graph"
)
export const FN_rangeArray = makeRuntimeFunction("range", [
        {
            name: "from",
            value: {
                type: "Integer"
            }
        },
        {
            name: "to",
            value: {
                type: "Integer"
            }
        },
        {
            name: "to_inclusive",
            value: {
                type: "Boolean"
            }
        },
    ],
    {
        type: "Iterable",
        value: {
            type: "Integer"
        }
    },
    "Returns an iterable of integers from `from` to `to` (inclusive or exclusive)"
);


export const ROOC_RUNTIME_FUNCTIONS = new Map<string, RuntimeFunction<NamedParameter[], SerializedPrimitiveKind>>([
    [FN_lenOfIterable.name, FN_lenOfIterable],
    [FN_enumerateArray.name, FN_enumerateArray],
    [FN_edges.name, FN_edges],
    [FN_nodes.name, FN_nodes],
    [FN_neigh_edges.name, FN_neigh_edges],
    [FN_neigh_edges_of.name, FN_neigh_edges_of],
    [FN_rangeArray.name, FN_rangeArray],
])

export type RuntimeBlockScopedFunction = {
    type: "RuntimeBlockScopedFunction";
    name: string;
    description: string;
}

function makeRuntimeBlockScopedFunctionEntry(name: string, description: string): [string, RuntimeBlockScopedFunction] {
    return [name, {type: "RuntimeBlockScopedFunction", name, description}]
}

export const RUNTIME_BLOCK_SCOPED_FUNCTIONS = new Map([
    makeRuntimeBlockScopedFunctionEntry("sum", "Expands the inner expression into a sum of all elements"),
    makeRuntimeBlockScopedFunctionEntry("prod", "Expands the inner expression into a product of all elements"),
    makeRuntimeBlockScopedFunctionEntry("min", "Expands the inner expression into the minimum of all elements"),
    makeRuntimeBlockScopedFunctionEntry("max", "Expands the inner expression into the maximum of all elements"),
    makeRuntimeBlockScopedFunctionEntry("avg", "Expands the inner expression into the average of all elements"),
])

export type RuntimeBlockFunction = {
    type: "RuntimeBlockFunction";
    name: string;
    description: string;
}

function makeRuntimeBlockFunctionEntry(name: string, description: string): [string, RuntimeBlockFunction] {
    return [name, {type: "RuntimeBlockFunction", name, description}]
}

export const RUNTIME_BLOCK_FUNCTIONS = new Map([
    makeRuntimeBlockFunctionEntry("min", "Computes the inner expression as the minimum of all elements"),
    makeRuntimeBlockFunctionEntry("max", "Computes the inner expression as the maximum of all elements"),
    makeRuntimeBlockFunctionEntry("avg", "Computes the inner expression as the average of all elements"),
])

export const RUNTIME_FUNCTIONS = ROOC_RUNTIME_FUNCTIONS.values()
export const RUNTIME_BLOCK_SCOPED_FUNCTION_NAMES = RUNTIME_BLOCK_SCOPED_FUNCTIONS.values()
export const RUNTIME_SCOPED_FUNCTION_NAMES = RUNTIME_BLOCK_FUNCTIONS.values()

export const ROOC_RUNTIME_TOKENS = [
    ...RUNTIME_FUNCTIONS,
    ...RUNTIME_BLOCK_SCOPED_FUNCTION_NAMES,
    ...RUNTIME_SCOPED_FUNCTION_NAMES,
]

const fuzzer = new Fuse(ROOC_RUNTIME_TOKENS, {
    keys: ["name"],
    threshold: 0.3,
    includeMatches: true,
    isCaseSensitive: false,
    shouldSort: true,
})


export function findRoocCompletionTokens(text: string) {
    return fuzzer.search(text).map(r => r.item)
}

export type PossibleCompletionToken = typeof ROOC_RUNTIME_TOKENS[number]

export function findRoocExactToken(text: string): PossibleCompletionToken | undefined {
    return ROOC_RUNTIME_TOKENS.find(t => t.name === text)
}


export type BuiltinType = {
    type: SerializedPrimitiveKind;
    description: string;
}

function makeBuiltinTypeEntry(type: SerializedPrimitiveKind, description: string): [string, BuiltinType] {
    return [type.type, {type, description}]
}

export const BUILTIN_TYPES_MAP = new Map([
    makeBuiltinTypeEntry({type: "Boolean"}, "A boolean value. can be defiend by `true` or `false`"),
    makeBuiltinTypeEntry({type: "Integer"}, "A integer"),
    makeBuiltinTypeEntry({type: "PositiveInteger"}, "A positive integer"),
    makeBuiltinTypeEntry({type: "Number"}, "A floating point number"),
    makeBuiltinTypeEntry({type: "String"}, "A string, can be defined by `\"...\"`"),
    makeBuiltinTypeEntry({type: "Any"}, "Any value"),
    makeBuiltinTypeEntry({
        type: "Iterable",
        value: {type: "Any"}
    }, "An iterable of a value, usually an array, defined as a comma separated list of values in square brackets `[]`, e.g. `[1,2,3]`"),
    makeBuiltinTypeEntry({type: "Tuple", value: []}, "A tuple of values"),
    makeBuiltinTypeEntry({type: "Graph"}, "A graph, can be defined as `Graph { ... }` where inside the brackets there are a list of nodes and it's adjacent edges, e.g. \n```rust\nGraph {\n    A -> [B:10, C],\n    B -> [C:2],\n    C\n}\n```"),
    makeBuiltinTypeEntry({type: "GraphNode"}, "A node of a graph"),
    makeBuiltinTypeEntry({type: "GraphEdge"}, "An edge of a graph"),
])
export const BUILTIN_TYPE = BUILTIN_TYPES_MAP.values()
const ALL = [
    ...ROOC_RUNTIME_TOKENS,
    ...BUILTIN_TYPE,
]
export const documentationFuzzer = new Fuse(ALL, {
    keys: ["name", "type", "description"],
    threshold: 0.5,
    includeMatches: true,
    isCaseSensitive: false,
    shouldSort: true,
})

export function findRoocDocumentation(text: string) {
    return documentationFuzzer.search(text).map(r => r.item)
}

function makePipeDescriptionEntry(type: Pipes, name: string, description: string, input: PipeDataType, output: PipeDataType) {
    return {
        type,
        name,
        description,
        input,
        output
    } as PipeDescription
}

type PipeDescription = {
    type: Pipes;
    name: string;
    description: string;
    input: PipeDataType;
    output: PipeDataType;
}
export const pipeDescriptions = {
    [Pipes.CompilerPipe]: makePipeDescriptionEntry(
        Pipes.CompilerPipe,
        "Compiler",
        "Compiles the code",
        PipeDataType.String,
        PipeDataType.Parser
    ),
    [Pipes.PreModelPipe]: makePipeDescriptionEntry(
        Pipes.PreModelPipe,
        "Pre Model",
        "Generates a model from the compiler output",
        PipeDataType.Parser,
        PipeDataType.PreModel
    ),
    [Pipes.ModelPipe]: makePipeDescriptionEntry(
        Pipes.ModelPipe,
        "Model",
        "Run the Pre Model to generate the static model",
        PipeDataType.PreModel,
        PipeDataType.Model
    ),
    [Pipes.LinearModelPipe]: makePipeDescriptionEntry(
        Pipes.LinearModelPipe,
        "Linear model",
        "Transforms the model into a linear model",
        PipeDataType.Model,
        PipeDataType.LinearModel
    ),
    [Pipes.StandardLinearModelPipe]: makePipeDescriptionEntry(
        Pipes.StandardLinearModelPipe,
        "Standard linear model",
        "Transforms the linear model into a model in standard form",
        PipeDataType.LinearModel,
        PipeDataType.StandardLinearModel
    ),
    [Pipes.TableauPipe]: makePipeDescriptionEntry(
        Pipes.TableauPipe,
        "Tableau for simplex",
        "Transforms the standard linear model into a tableau that can be used in the simplex algorithm, it creates artificial variables to find the initial basis",
        PipeDataType.StandardLinearModel,
        PipeDataType.Tableau
    ),
    [Pipes.SimplexPipe]: makePipeDescriptionEntry(
        Pipes.SimplexPipe,
        "Simplex solver",
        "Runs the simplex algorithm to find the optimal solution",
        PipeDataType.Tableau,
        PipeDataType.OptimalTableau
    ),
    [Pipes.StepByStepSimplexPipe]: makePipeDescriptionEntry(
        Pipes.StepByStepSimplexPipe,
        "Simplex solver with steps",
        "Runs the simplex algorithm to find the optimal solution and returns the tableau at each step",
        PipeDataType.Tableau,
        PipeDataType.OptimalTableauWithSteps
    ),
    [Pipes.BinarySolverPipe]: makePipeDescriptionEntry(
        Pipes.BinarySolverPipe,
        "Binary solver",
        "Runs a binary solver to find the optimal solution, the variables must be binary",
        PipeDataType.LinearModel,
        PipeDataType.BinarySolution
    ),
    [Pipes.IntegerBinarySolverPipe]: makePipeDescriptionEntry(
        Pipes.IntegerBinarySolverPipe,
        "Integer binary solver",
        "Runs a binary and integer solver to find the optimal solution, the variables must be binary or integer",
        PipeDataType.LinearModel,
        PipeDataType.IntegerBinarySolution
    ),
} satisfies Record<Pipes, PipeDescription>

function makePipeDataEntry(type: PipeDataType, name: string, description: string) {
    return {
        type,
        name,
        description
    } as PipeDataDescription
}


type PipeDataDescription = {
    type: PipeDataType;
    name: string;
    description: string;
}
export const pipeDataDescriptions = {
    [PipeDataType.String]: makePipeDataEntry(PipeDataType.String, "String", "A string"),
    [PipeDataType.Parser]: makePipeDataEntry(PipeDataType.Parser, "Parser", "The ROOC parser"),
    [PipeDataType.PreModel]: makePipeDataEntry(PipeDataType.PreModel, "Pre Model", "The parsed model"),
    [PipeDataType.Model]: makePipeDataEntry(PipeDataType.Model, "Model", "The compiled model"),
    [PipeDataType.LinearModel]: makePipeDataEntry(PipeDataType.LinearModel, "Linear Model", "The linear model"),
    [PipeDataType.StandardLinearModel]: makePipeDataEntry(PipeDataType.StandardLinearModel, "Standard Linear Model", "The linear model in standard form"),
    [PipeDataType.Tableau]: makePipeDataEntry(PipeDataType.Tableau, "Tableau", "The tableau for the simplex algorithm"),
    [PipeDataType.OptimalTableau]: makePipeDataEntry(PipeDataType.OptimalTableau, "Optimal Tableau", "The tableau after running the simplex algorithm"),
    [PipeDataType.OptimalTableauWithSteps]: makePipeDataEntry(PipeDataType.OptimalTableauWithSteps, "Optimal Tableau with Steps", "The tableau at each step of the simplex algorithm"),
    [PipeDataType.BinarySolution]: makePipeDataEntry(PipeDataType.BinarySolution, "Binary Solution", "The optimal solution of a binary model"),
    [PipeDataType.IntegerBinarySolution]: makePipeDataEntry(PipeDataType.IntegerBinarySolution, "Integer Binary Solution", "The optimal solution of a binary or integer model"),
} satisfies Record<PipeDataType, PipeDataDescription>