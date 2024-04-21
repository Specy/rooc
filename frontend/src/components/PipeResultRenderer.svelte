<script lang="ts">
    import {pipeDataDescriptions, PipeDataType, pipeDescriptions, type Pipes, type RoocData} from "@specy/rooc";
    import SyntaxHighlighter from "$cmp/SyntaxHighlighter.svelte";
    import Column from "$cmp/layout/Column.svelte";
    import FaChevronDown from "~icons/fa/chevron-down.svelte";

    export let data: RoocData
    export let pipeStep: Pipes | string
    export let expanded: boolean = false
    console.log(pipeStep, pipeDescriptions[pipeStep], pipeDescriptions, data)
</script>


<div class="pipe-result" class:pipe-result-open={expanded}>
    <button
            on:click={() => expanded = !expanded}
            class="pipe-expand"
    >
        <div class="chevron-icon" class:chevron-icon-expanded={expanded}>
            <FaChevronDown/>
        </div>
        <h2>
            {typeof pipeStep === "string" ? pipeStep : pipeDescriptions[pipeStep].name}
            ({pipeDataDescriptions[data.type].name})
        </h2>
    </button>
    <div class="pipe-content">
        {#if data.type === PipeDataType.String}
            <SyntaxHighlighter language="rooc" source={data.data}
                               style="overflow-y: auto; overflow-x: auto; max-height: 50vh"/>
        {:else if data.type === PipeDataType.Parser}
            <div> Internal ROOC compiler</div>
        {:else if data.type === PipeDataType.PreModel}
            <div> Internal compiled model</div>
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
    </div>
</div>

<style>
    .pipe-expand {
        display: flex;
        align-items: center;
        gap: 1rem;
        background-color: transparent;
        cursor: pointer;
        color: var(--primary-text);
    }

    .pipe-result {
        display: flex;
        flex-direction: column;
        padding: 0.8rem;
        background-color: var(--primary);
        color: var(--primary-text);
        border-radius: 0.4rem;
        border: solid 0.2rem transparent;
    }

    .chevron-icon {
        transition: all 0.2s;
        transform: rotate(-90deg);
    }

    .chevron-icon-expanded {
        transform: rotate(0deg);
    }

    .pipe-result-open {
        border: solid 0.2rem var(--secondary-5);
    }

    .pipe-content {
        display: none;
        flex-direction: column;
        border-top: solid 0.2rem var(--secondary-5);
        margin-top: 0.5rem;
        padding-top: 0.5rem;
    }

    .pipe-result-open .pipe-content {
        display: flex;
        animation: appear 0.2s;
    }

    pre {
        overflow-x: auto;
        max-height: 50vh
    }

    @keyframes appear {
        from {
            opacity: 0;
        }
        to {
            opacity: 1;
        }
    }
</style>