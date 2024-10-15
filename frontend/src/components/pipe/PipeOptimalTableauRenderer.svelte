<script lang="ts">
    import {type OptimalTableau} from "@specy/rooc";
    import PipeTableauRenderer from "$cmp/pipe/PipeTableauRenderer.svelte";
    import Column from "$cmp/layout/Column.svelte";
    import PipeVariablesRenderer from "$cmp/pipe/PipeVariablesRenderer.svelte";

    export let tableau: OptimalTableau

    $: baseTableau = tableau.getTableau()
</script>


<Column gap="1rem">
    <PipeTableauRenderer tableau={baseTableau}/>
    <div style="font-size: 1.5rem">
        Optimal value: {tableau.getOptimalValue()}
    </div>
    <PipeVariablesRenderer
            {tableau}
    />
</Column>
