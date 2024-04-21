<script lang="ts">
    import {pipeDataDescriptions, PipeDataType, pipeDescriptions, type Pipes, type RoocData} from "@specy/rooc";
    import SyntaxHighlighter from "$cmp/SyntaxHighlighter.svelte";
    import Column from "$cmp/layout/Column.svelte";
    import ExpandableContainer from "$cmp/layout/ExpandableContainer.svelte";

    export let data: RoocData
    export let pipeStep: Pipes | string
    export let expanded: boolean = false
</script>

<ExpandableContainer bind:expanded>
    <h2 slot="title">
        {typeof pipeStep === "string" ? pipeStep : pipeDescriptions[pipeStep].name}
        ({pipeDataDescriptions[data.type].name})
    </h2>
    <div style="margin: 0.5rem 0">
        {pipeDataDescriptions[data.type].description}
    </div>
    {#if data.type === PipeDataType.String}
        <SyntaxHighlighter language="rooc" source={data.data}
                           style="overflow-y: auto; overflow-x: auto; max-height: 50vh"/>
    {:else if data.type === PipeDataType.Parser}
        <b> Internal ROOC compiler</b>
    {:else if data.type === PipeDataType.PreModel}
        <b> Internal compiled model</b>
    {:else if data.type === PipeDataType.Model}
        <SyntaxHighlighter language="rooc" source={data.data.stringify()}
                           style="overflow-y: auto; overflow-x: auto; max-height: 50vh"/>
    {:else if data.type === PipeDataType.LinearModel}
        <SyntaxHighlighter language="rooc" source={data.data.stringify()}
                           style="overflow-y: auto; overflow-x: auto; max-height: 50vh"/>
    {:else if data.type === PipeDataType.StandardLinearModel}
        <SyntaxHighlighter language="rooc" source={data.data.stringify()}
                           style="overflow-y: auto; overflow-x: auto; max-height: 50vh"/>
    {:else if data.type === PipeDataType.Tableau}
        <pre>{data.data.stringify()}</pre>

    {:else if data.type === PipeDataType.OptimalTableau}
        <pre>{data.data.getTableau().stringify()}</pre>

        <Column>
            Optimal value: {data.data.getOptimalValue()}
        </Column>
    {/if}
</ExpandableContainer>


<style>

    pre {
        overflow-x: auto;
        max-height: 50vh
    }

</style>