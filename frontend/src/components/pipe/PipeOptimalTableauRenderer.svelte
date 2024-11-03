<script lang="ts">
    import type { OptimalTableau} from "@specy/rooc";
    import PipeTableauRenderer from "$cmp/pipe/PipeTableauRenderer.svelte";
    import Column from "$cmp/layout/Column.svelte";
    import PipeVariablesRenderer from "$cmp/pipe/PipeVariablesRenderer.svelte";
    import {formatNum} from "$cmp/pipe/utils";

    interface Props {
        tableau: OptimalTableau;
        showSteps?: boolean;
    }

    let {
        tableau,
        showSteps = false
    }: Props = $props();

    let baseTableau = $derived(tableau.getTableau())
</script>


<Column gap="1rem">
    {#if showSteps}
        <PipeTableauRenderer tableau={baseTableau}/>
    {/if}
    <div style="font-size: 1.5rem">
        Optimal value: {formatNum(tableau.getOptimalValue())}
    </div>
    <PipeVariablesRenderer
            {tableau}
    />
</Column>
