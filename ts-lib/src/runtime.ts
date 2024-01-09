import { SerializedPrimitiveKind } from "./pkg/rooc";
import Fuse from 'fuse.js'
export type NamedParameter = {
    name: string;
    value: SerializedPrimitiveKind;
}
export type RuntimeFunction<T extends NamedParameter[], R extends SerializedPrimitiveKind> = {
    name: string;
    type: "RuntimeFunction";
    parameters: T;
    returnType: R;
}
export function makeRuntimeFunction<T extends NamedParameter[], R extends SerializedPrimitiveKind>(name: string, parameters: T, returnType: R): RuntimeFunction<T, R> {
    return { name, parameters, returnType, type: "RuntimeFunction" };
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
    { type: "PositiveInteger" }
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
            value: [{ type: "Any" }, { type: "PositiveInteger" }]
        }
    }
)

export const FN_edges = makeRuntimeFunction("edges", [
    { name: "of_graph", value: { type: "Graph" } }
],
    { type: "Iterable", value: { type: "GraphEdge" } }
)

export const FN_nodes = makeRuntimeFunction("nodes", [
    { name: "of_graph", value: { type: "Graph" } }
],
    { type: "Iterable", value: { type: "GraphNode" } }
)
export const FN_neigh_edges = makeRuntimeFunction("neigh_edges", [
    { name: "of_node", value: { type: "GraphNode" } },
],
    { type: "Iterable", value: { type: "GraphEdge" } }
)

export const FN_neigh_edges_of = makeRuntimeFunction("neigh_edges_of", [
    { name: "of_node_name", value: { type: "String" } },
    { name: "in_graph", value: { type: "Graph" } },
],
    { type: "Iterable", value: { type: "GraphEdge" } }
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
    }
);


export const ROOC_RUNTIME_FUNCTIONS = new Map<string, RuntimeFunction<any, any>>([
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
    return [name, { type: "RuntimeBlockScopedFunction", name, description }]
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
return [name, { type: "RuntimeBlockFunction", name, description }]
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
    isCaseSensitive: true,
    shouldSort: true,
})

export type PossibleCompletionToken = RuntimeBlockFunction | RuntimeBlockScopedFunction | RuntimeFunction<any, any>

export function findCompletion(text: string) {
    return fuzzer.search(text).map(r => r.item)
}
export function findExact(text: string): PossibleCompletionToken | undefined {
    return ROOC_RUNTIME_TOKENS.find(t => t.name === text)
}