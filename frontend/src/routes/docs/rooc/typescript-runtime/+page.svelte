<script lang="ts">
    import Page from '$cmp/layout/Page.svelte';
    import Card from "$cmp/layout/Card.svelte";
    import SyntaxHighlighter from "$cmp/SyntaxHighlighter.svelte";
</script>

<svelte:head>
    <title>
        Rooc docs - Runtime
    </title>
    <meta name="description" content="The documentation of ROOC, with syntax, function definitions and examples"/>
</svelte:head>

<Page cropped padding='1rem' mobilePadding='1rem' gap="1rem">
    <h1>
        Typescript runtime
    </h1>
    <p>
        You can extend the runtime of ROOC by defining your own functions.
        <br/>
        In the project editor, click the "Show runtime" button, and a typescript editor will appear below the model
        editor.
        <br/>
        <br/>
        You can define a new function by calling the "register" function and passing the name of the function, the types
        of the parameters, the return type and the function itself.
        <br/>
        <br/>
        As the runtime is not executed in javascript itself, but rather a WASM compiled rust code, you need to
        serialize/deserialize the data to/from the runtime as a JSON object.
        <br/>
        <br/>
        Utility constants are provided to help you with this task. One example is the "Primitive" type, which you can
        use to define the type of the parameters and the return type of the function.
        <br/>
        <br/>
        <Card padding="1rem">
            <SyntaxHighlighter language="typescript" source={`const Primitive = {
    Number: {type: 'Number'},
    Integer: {type: 'Integer'},
    PositiveInteger: {type: 'PositiveInteger'},
    String: {type: 'String'},
    Iterable: (value: Primitive) => ({type: 'Iterable', value}),
    Graph: {type: 'Graph'},
    GraphEdge: {type: 'GraphEdge'},
    GraphNode: {type: 'GraphNode'},
    Tuple: (value: Primitive[]) => ({type: 'Tuple', value}) ,
    Boolean: {type: 'Boolean'},
    Undefined: {type: 'Undefined'},
    Any: {type: 'Any'},
}`} style="overflow-x: auto;"/>

        </Card>
        <br/>
        Here is an example on how to register a function that will sum all the elements of an array:
        <br/>
        <br/>
        <Card padding="1rem">
            <SyntaxHighlighter language="typescript" source={`register({
    name: 'sum',
    description: 'Sum all the elements of an array',
    //this is an array of tuples, the first element is the name of the parameter, the second is the type
    parameters: [['of_array', Primitive.Iterable(Primitive.Number)]],
    returns: Primitive.Number,
    call: (arr) => {
       const count = arr.value.value.reduce((acc, curr) => acc + curr.value, 0);
       return {type: "Number", value: count}
    }
})`} style="overflow-x: auto;"/>
        </Card>
        <br/>

        If you need more advanced usecases, the "returns" field can either be a constant value or a function that will
        calculate what the return type will be. If there is an error in this function, the "Any" type will be used.
        Here is an example of using the "returns" field as a function:
        <br/>
        <br/>
        <Card padding="1rem">
            <SyntaxHighlighter language="typescript" source={`register({
    name: "filter_js",
    description: "Filter over an array with the provided js function",
    parameters: [
        ['of_arr', Primitive.Iterable(Primitive.Any)],
        ['js_code', Primitive.String]
    ],
    //the first array is the type of the parameters when the function is called
    //the second array is the static values that are passed to the function, in this case, the code if it's inlined as a string
    returns: ([arr], [_, code]) => {
        return { type: 'Iterable', value: arr.value };
    },
    call: (arr, code) => {
        let type = arr.value.type;
        let fn = eval(code.value);
        let mapped = arr.value.value.filter(fn);
        return { type: "Iterable", value: { type, value: mapped } };
    }
});`} style="overflow-x: auto;"/>
        </Card>
        <br/>
        The serialized data is a JSON like object that has a "type" field which tells you the type of the data, and a
        "value" field which is the actual value of the data. Here are the type definitions for it:
        <br/>
        <br/>
        <Card padding="1rem">
            <SyntaxHighlighter language="typescript" source={`type SerializedPrimitive =
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

type SerializedGraphEdge = {
    from: string,
    to: string,
    weight?: number
}

type SerializedGraphNode = {
    name: string,
    edges: Record<string, SerializedGraphEdge>
}

type SerializedGraph = {
    vertices: SerializedGraphNode[]
}

type SerializedTuple = SerializedPrimitive[]


type SerializedIterable =
    | { type: 'Numbers', value: number[] }
    | { type: 'Integers', value: number[] }
    | { type: 'PositiveIntegers', value: number[] }
    | { type: 'Strings', value: string[] }
    | { type: 'Edges', value: SerializedGraphEdge[] }
    | { type: 'Nodes', value: SerializedGraphNode[] }
    | { type: 'Graphs', value: SerializedGraph[] }
    | { type: 'Tuples', value: SerializedTuple[] }
    | { type: 'Booleans', value: boolean[] }
    | { type: 'Iterables', value: SerializedIterable[] }`} style="overflow-x: auto;"/>
        </Card>
    </p>

    <h2>
        Useful functions
    </h2>
    <p>
        Inside of the typescript runtime there are some utility functions defined in the "Rooc" namespace.
        <br/>
        <br/>
        Using the global <code>GET_FILES()</code> function, you can get the contents (as strings) of the files that
        are in the project (you can select them with the "attach files" button).
        <br/>
        <br/>
        In the <code>Rooc.parseCsvTable</code> and <code>Rooc.parseCsvObject</code> you can pass a CSV string and it
        will return an array of arrays or an array of objects respectively, of the parsed CSV string.
        Internally it uses the <a href="https://csv.js.org/parse/distributions/browser_esm/">csv</a> library to parse
        the
        CSV string, so look at the documentation of that library for more information.
        <br/>
        <br/>
        Other useful functions is the integration of <a href="https://github.com/dagrejs/graphlib/wiki">graphlib</a>
        library
        to work with graphs. You can use the <code>Rooc.Graph</code> class to create a graph manually, or using the
        <code>Rooc.fromSerializedGraph</code>
        function to create a graph from a rooc serialized graph.
        <br/>
        <br/>
        You can also find algorithms to use over graphs in the <code>Rooc.GraphAlgorithms</code>, look at the graphlib
        library for more information.

    </p>
</Page>


<style>
    @import "../common.scss";

    code {
        background-color: var(--primary);
        border-radius: 0.3rem;
        font-size: 0.9rem;
        color: var(--accent-15);
        padding: 0.1rem 0.3rem;
    }
</style>