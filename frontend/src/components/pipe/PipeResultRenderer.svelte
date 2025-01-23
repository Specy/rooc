<script lang="ts">
    import {pipeDataDescriptions, PipeDataType, pipeDescriptions, type Pipes, type RoocData} from "@specy/rooc";
    import SyntaxHighlighter from "$cmp/SyntaxHighlighter.svelte";
    import ExpandableContainer from "$cmp/layout/ExpandableContainer.svelte";
    import PipeOptimalTableauRenderer from "$cmp/pipe/PipeOptimalTableauRenderer.svelte";
    import PipeTableauRenderer from "$cmp/pipe/PipeTableauRenderer.svelte";
    import PipeOptimalTableauWithStepsRenderer from "$cmp/pipe/PipeOptimalTableauWithStepsRenderer.svelte";
    import BinarySolutionRenderer from "$cmp/pipe/BinarySolutionRenderer.svelte";
    import RealSolutionRenderer from "$cmp/pipe/RealSolutionRenderer.svelte";
    import MILPSolutionRenderer from "$cmp/pipe/MILPSolutionRenderer.svelte";
    import {getDataOfPipe, getDescriptionOfPipe} from "$lib/appPipes/pipeDescriptions";

    interface Props {
        data: RoocData;
        pipeStep: Pipes | string;
        expanded?: boolean;
        id: string;
    }

    let {data, pipeStep, expanded = $bindable(false), id}: Props = $props();


</script>

<ExpandableContainer
        bind:expanded
        disabled={data.type === PipeDataType.Parser || data.type === PipeDataType.PreModel}
>
    {#snippet title()}
        <h2 {id}>
            {typeof pipeStep === "string" ? pipeStep : getDescriptionOfPipe(pipeStep).name}

        </h2>
    {/snippet}
    <div style="margin: 0.5rem 0">
        {getDataOfPipe(data.type).description}
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
        <PipeTableauRenderer tableau={data.data}/>
    {:else if data.type === PipeDataType.OptimalTableau}
        <PipeOptimalTableauRenderer tableau={data.data} showSteps={false}/>
    {:else if data.type === PipeDataType.OptimalTableauWithSteps}
        <PipeOptimalTableauWithStepsRenderer data={data.data}/>
    {:else if data.type === PipeDataType.BinarySolution}
        <BinarySolutionRenderer binarySolution={data.data}/>
    {:else if data.type === PipeDataType.IntegerBinarySolution}
        <MILPSolutionRenderer milpSolution={data.data}/>
    {:else if data.type === PipeDataType.MILPSolution}
        <MILPSolutionRenderer milpSolution={data.data}/>
    {:else if data.type === PipeDataType.RealSolution}
        <RealSolutionRenderer realSolution={data.data}/>
    {/if}
</ExpandableContainer>


<style>

    pre {
        overflow-x: auto;
        max-height: 50vh
    }

</style>