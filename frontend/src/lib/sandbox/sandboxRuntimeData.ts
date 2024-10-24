

export const SANDBOX_RUNTIME_DATA = `
const Primitive = {
    Number: { type: 'Number' },
    Integer: { type: 'Integer' },
    PositiveInteger: { type: 'PositiveInteger' },
    String: { type: 'String' },
    Iterable: (value) => ({ type: 'Iterable', value }),
    Graph: { type: 'Graph' },
    GraphEdge: { type: 'GraphEdge' },
    GraphNode: { type: 'GraphNode' },
    Tuple: (value) => ({ type: 'Tuple', value }),
    Boolean: { type: 'Boolean' },
    Undefined: { type: 'Undefined' },
    Any: { type: 'Any' },
};

`