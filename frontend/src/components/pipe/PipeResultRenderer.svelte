<script lang="ts">
    import { PipeDataType, Pipes, type RoocData} from "@specy/rooc";
    import SyntaxHighlighter from "$cmp/SyntaxHighlighter.svelte";
    import ExpandableContainer from "$cmp/layout/ExpandableContainer.svelte";
    import PipeOptimalTableauRenderer from "$cmp/pipe/PipeOptimalTableauRenderer.svelte";
    import PipeTableauRenderer from "$cmp/pipe/PipeTableauRenderer.svelte";
    import PipeOptimalTableauWithStepsRenderer from "$cmp/pipe/PipeOptimalTableauWithStepsRenderer.svelte";
    import BinarySolutionRenderer from "$cmp/pipe/BinarySolutionRenderer.svelte";
    import RealSolutionRenderer from "$cmp/pipe/RealSolutionRenderer.svelte";
    import MILPSolutionRenderer from "$cmp/pipe/MILPSolutionRenderer.svelte";
    import {getDataOfPipe, getDescriptionOfPipe} from "$lib/appPipes/pipeDescriptions";
    import Button from "$cmp/inputs/Button.svelte";
    import {toast} from "$stores/toastStore";

    interface Props {
        data: RoocData;
        pipeStep: Pipes | string;
        expanded?: boolean;
        id: string;
    }

    let {data, pipeStep, expanded = $bindable(false), id}: Props = $props();

    function detectRoocOrCplex(code: string){
        if(code.trim().toLowerCase().endsWith("end")){
            return 'cplex';
        }else{
            return 'rooc';
        }
    }

</script>

<ExpandableContainer
        bind:expanded
        disabled={data.type === PipeDataType.Parser || data.type === PipeDataType.PreModel}
>
    {#snippet title()}
        <h2 {id} title="{getDataOfPipe(data.type).description}">
            {typeof pipeStep === "string" ? pipeStep : getDescriptionOfPipe(pipeStep).name}
        </h2>
    {/snippet}
    {#if data.type === PipeDataType.String}
        <SyntaxHighlighter language={detectRoocOrCplex(data.data)} source={data.data}
                           style="overflow-y: auto; overflow-x: auto; max-height: 50vh"/>
        <Button
                on:click={() => {
                    navigator.clipboard.writeText(data.data)
                    toast.logPill("Copied to clipboard")
                }}
                style="margin-top: 0.5rem; margin-left: auto"
        >
            Copy
        </Button>
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