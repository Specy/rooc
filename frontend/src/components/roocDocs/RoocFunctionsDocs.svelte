<script lang="ts">
    import SyntaxHighlighter from '$cmp/SyntaxHighlighter.svelte';
    import Card from '$cmp/layout/Card.svelte';
    import Column from '$cmp/layout/Column.svelte';
    import {createRoocFunctionSignature} from '$src/lib/Rooc/RoocUtils';
    import {
        ROOC_RUNTIME_FUNCTIONS,
        RUNTIME_BLOCK_FUNCTIONS,
        RUNTIME_BLOCK_SCOPED_FUNCTIONS,
        pipeDataDescriptions,
        pipeDescriptions
    } from '@specy/rooc';
    import Separator from "$cmp/misc/Separator.svelte";
    import Row from "$cmp/layout/Row.svelte";

    let functions = ROOC_RUNTIME_FUNCTIONS.values();
    let blockFunctions = RUNTIME_BLOCK_FUNCTIONS.values();
    let blockScopedFunctions = RUNTIME_BLOCK_SCOPED_FUNCTIONS.values();
</script>

<h1>Functions</h1>
<Column gap="0.5rem">
    <Column style="margin-bottom: 1rem;" gap="0.4rem">
        They are functions that accept parameters and return a value, you can use them inside blocks,
        assignments or in expressions
        <br/>
        <Card padding="0.8rem">
            <SyntaxHighlighter style="overflow-x: auto" language="rooc" source={'x * len(A) <= 2'}/>
        </Card>
    </Column>
    {#each functions as fun}
        <Card padding="0.8rem" gap="0.5rem">
            <SyntaxHighlighter
                    style="overflow-x: auto"
                    language="typescript"
                    source={createRoocFunctionSignature(fun)}
            />
            {fun.description}
        </Card>
    {/each}
</Column>
<Separator/>
<h1>Block functions</h1>
<Column gap="0.5rem">
    <Column>
        They are blocks which have one or more expressions separated by a comma, they will use those
        expressions to perform a transformation, like the avg (average) block
    </Column>
    {#each blockFunctions as fun}
        <Card padding="0.8rem" gap="0.5rem">
            <SyntaxHighlighter
                    style="overflow-x: auto"
                    language="typescript"
                    source={createRoocFunctionSignature(fun)}
            />
            {fun.description}
        </Card>
    {/each}
</Column>
<Separator/>
<h1>Block scoped functions</h1>
<Column gap="0.5rem">
    <Column gap="0.4rem" style="margin-bottom: 1rem;">
        They are function blocks, it has as parameters one or more iterators over iterable data, they
        will declare a variable (or more using tuples destructuring) for each iterator and then execute
        the block.
        <br/>
        If there are more than one iterators, they will behave as nested iterators, where the first iterator
        is the outermost one
        <br/>
        <Card padding="0.8rem">
            <SyntaxHighlighter style="overflow-x: auto" language="rooc"
                               source={'sum(i in 0..len(A), el in A[i]) { x_i * el }'}/>
        </Card>
    </Column>
    {#each blockScopedFunctions as fun}
        <Card padding="0.8rem" gap="0.5rem">
            <SyntaxHighlighter style="overflow-x: auto" language="typescript"
                               source={createRoocFunctionSignature(fun)}/>
            {fun.description}
        </Card>
    {/each}
</Column>
<h1>
    Execution pipes
</h1>
<Column gap="0.5rem">
    <Column gap="0.4rem" style="margin-bottom: 1rem;">
        Other than the model, you can define an execution pipeline that you can customise to do what you need.
        <br/>
        There are some presets you can choose from, but in general, each pipe step has an input and produces an output,
        each step of the pipeline will be executed one after the other in the order they are defined, and each result
        will be
        shown as the output.
    </Column>
    {#each Object.keys(pipeDescriptions) as pipe}
        <Card padding="0.8rem" gap="0.5rem">
            <Column gap="0.2rem">
                <Row>
                    <div style="width: 8ch">Name: </div><b>{pipeDescriptions[pipe].name}</b>
                </Row>
                    <Row>
                        <div style="width: 8ch">Input: </div><code>{pipeDataDescriptions[pipeDescriptions[pipe].input].name}</code>
                    </Row>
                    <Row>
                        <div style="width: 8ch">Output: </div> <code>{pipeDataDescriptions[pipeDescriptions[pipe].output].name}</code>
                    </Row>
            </Column>
            <div>
                {pipeDescriptions[pipe].description}
            </div>
        </Card>
    {/each}
</Column>


<style>
    code {
        background-color: var(--secondary-10);
        padding: 0rem 0.4rem;
        display: flex;
        align-items: center;
        border-radius: 0.3rem;
    }
</style>