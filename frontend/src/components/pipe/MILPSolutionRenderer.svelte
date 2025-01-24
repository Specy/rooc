<script lang="ts">
    import type {LpSolution, MILPValue, VarValue} from "@specy/rooc";
    import Column from "$cmp/layout/Column.svelte";
    import Var from "$cmp/pipe/Var.svelte";
    import {formatNum} from "$cmp/pipe/utils";
    import {textDownloader} from "$lib/utils";
    import Button from "$cmp/inputs/Button.svelte";
    import Row from "$cmp/layout/Row.svelte";

    interface Props {
        milpSolution: LpSolution<MILPValue | VarValue>;
    }

    let {milpSolution}: Props = $props();

</script>

<Column gap="1rem">

    <div class="table-wrapper">
        <table>
            <thead>
            <tr>
                {#each milpSolution.assignment as assignment}
                    <th>
                        <Var value={assignment.name}/>
                    </th>
                {/each}
            </tr>
            </thead>
            <tbody>
            <tr>
                {#each milpSolution.assignment as assignment}
                    <td
                            class={assignment.value.type === 'Bool' ? assignment.value.value ? 'T' : 'F' : "int"}
                    >
                        {assignment.value.type === 'Bool' ? assignment.value.value ? 'T' : 'F' : undefined}
                        {assignment.value.type === "Int" ? assignment.value.value : undefined}
                        {assignment.value.type === "Real" ? formatNum(assignment.value.value) : undefined}
                    </td>
                {/each}
            </tr>
            </tbody>
        </table>

    </div>
    <Row justify="between" wrap align="center" gap="0.5rem">
        <div style="font-size: 1.5rem">
            Optimal value: {formatNum(milpSolution.value)}
        </div>
        <Button
                on:click={() => textDownloader(JSON.stringify({
                value: milpSolution.value,
                assignment: Object.fromEntries(milpSolution.assignment.map(({name, value}) => [name, value]))
                }, null, 4),'solution.json')}
        >
            Download solution
        </Button>
    </Row>
</Column>

<style lang="scss">
  .table-wrapper {
    background-color: var(--secondary);
    border-radius: 0.5rem;
    overflow: hidden;
    overflow-x: auto;
    overflow-y: auto;
    border: solid 0.1rem var(--secondary-10);
  }

  table {
    min-width: 100%;
    border-collapse: collapse;
  }

  thead {
    background-color: var(--secondary-5);

    th {
      border: solid 0.1rem var(--secondary-10);
    }
  }

  th, td {
    padding: 0.3rem;
    text-align: center;
    border: solid 0.1rem var(--secondary-5);
  }

  th {
    font-weight: bold;
  }

  .T {
    color: var(--success);
    font-weight: bold;
  }

  .F {
    color: var(--danger);
    font-weight: bold;

  }
</style>