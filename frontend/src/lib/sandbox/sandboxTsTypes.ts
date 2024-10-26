export function getTsGlobal(){
    return `
declare type SerializedPrimitiveKind =
    | { type: 'Number' }
    | { type: 'Integer' }
    | { type: 'PositiveInteger' }
    | { type: 'String' }
    | { type: 'Iterable', value: SerializedPrimitiveKind }
    | { type: 'Graph' }
    | { type: 'GraphEdge' }
    | { type: 'GraphNode' }
    | { type: 'Tuple', value: SerializedPrimitiveKind[] }
    | { type: 'Boolean' }
    | { type: 'Undefined' }
    | { type: 'Any' }

const Primitive = {
    Number: {type: 'Number'},
    Integer: {type: 'Integer'},
    PositiveInteger: {type: 'PositiveInteger'},
    String: {type: 'String'},
    Iterable: <T extends SerializedPrimitiveKind>(value: T) => ({type: 'Iterable', value} as const),
    Graph: {type: 'Graph'},
    GraphEdge: {type: 'GraphEdge'},
    GraphNode: {type: 'GraphNode'},
    Tuple: <T extends SerializedPrimitiveKind>(value: T[]) => ({type: 'Tuple', value} as const) ,
    Boolean: {type: 'Boolean'},
    Undefined: {type: 'Undefined'},
    Any: {type: 'Any'},
} as const
declare type Primitive = typeof Primitive

declare type ExtractArgTypes<T extends [string, SerializedPrimitiveKind][]> = {
    [K in keyof T]: T[K] extends [string, infer Type extends SerializedPrimitiveKind]
        ? Type extends { type: 'Iterable' }
            ? { type: 'Iterable', value: (SerializedIterable & { type: \`\${Type['value']['type']}s\` }) }
            : SerializedPrimitive & { type: Type['type'] }
    : never;
};

declare type MakeRoocFunction<T extends [string, SerializedPrimitiveKind][]> = {
    name: string;
    description?: string;
    parameters: T;
    returns: SerializedPrimitiveKind;
    type_checker?: (...args: SerializedPrimitiveKind[]) => null | string;
    call: (...args: NoInfer<ExtractArgTypes<T>>) => SerializedPrimitive;
};



declare type SerializedGraphEdge = {
    from: string,
    to: string,
    weight?: number
}



declare type SerializedGraphNode = {
    name: string,
    edges: { [key: string]: SerializedGraphEdge }
}



declare type SerializedGraph = {
    vertices: SerializedGraphNode[]
}

declare type SerializedPrimitive =
    | { type: 'Number', value: number }
    | { type: 'Integer', value: number }
    | { type: 'PositiveInteger', value: number }
    | { type: 'String', value: string }
    | { type: 'Iterable', value: SerializedIterable }
    | { type: 'Graph', value: SerializedGraph }
    | { type: 'GraphEdge', value: SerializedGraphEdge }
    | { type: 'GraphNode', value: SerializedGraphNode }
    | { type: 'Tuple', value: SerializedTuple }
    | { type: 'Boolean', value: boolean }
    | { type: 'Undefined' }


declare type SerializedIterable = 
    | { type: 'Numbers', value: number[] }
    | { type: 'Integers', value: number[] }
    | { type: 'PositiveIntegers', value: number[] }
    | { type: 'Strings', value: string[] }
    | { type: 'Edges', value: SerializedGraphEdge[] }
    | { type: 'Nodes', value: SerializedGraphNode[] }
    | { type: 'Graphs', value: SerializedGraph[] }
    | { type: 'Tuples', value: SerializedTuple[] }
    | { type: 'Booleans', value: boolean[] }
    | { type: 'Iterables', value: SerializedIterable[] }

declare type SerializedTuple = SerializedPrimitive[]

declare function register<const T extends [string, SerializedPrimitiveKind][]>({ name, parameters, returns, type_checker, call, description }: MakeRoocFunction<T>): void
 
declare function GET_FILES(): string[]
    `
}
