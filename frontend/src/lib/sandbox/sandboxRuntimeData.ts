

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
//these apis use eval so better define them in the sandbox
register({
    name: "map_js",
    description: "Maps over an array with the provided js function, you need to specify the return type",
    parameters: [
        ['of_arr', Primitive.Iterable(Primitive.Any)],
        ['as_type', Primitive.String],
        ['js_code', Primitive.String]
    ],
    returns: (_, [v1, v2]) => {
        return { type: 'Iterable', value: { type: v2.value } };
    },
    call: (arr, asVal, code) => {
        let fn = eval(code.value);
        let mapped = arr.value.value.map(fn);
        return { type: "Iterable", value: { type: \`\${asVal.value}s\`, value: mapped } };
    }
});

register({
    name: "filter_js",
    description: "Filter over an array with the provided js function",
    parameters: [
        ['of_arr', Primitive.Iterable(Primitive.Any)],
        ['js_code', Primitive.String]
    ],
    returns: ([arr]) => {
        return { type: 'Iterable', value: arr.value };
    },
    call: (arr, code) => {
        let type = arr.value.type;
        let fn = eval(code.value);
        let mapped = arr.value.value.filter(fn);
        return { type: "Iterable", value: { type, value: mapped } };
    }
});

register({
    name: "find_js",
    description: "Finds an element inside an array, or undefined otherwise",
    parameters: [
        ['of_arr', Primitive.Iterable(Primitive.Any)],
        ['js_code', Primitive.String]
    ],
    returns: ([arr]) => {
        return { type: arr.value.type };
    },
    call: (arr, code) => {
        let type = arr.value.type;
        let fn = eval(code.value);
        let found = arr.value.value.find(fn);
        if (found === undefined) {
            return { type: "Undefined" };
        }
        return { type: type.slice(0, type.length - 1), value: found };
    }
});



`