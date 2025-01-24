<script lang="ts">
    import type {LpSolution} from "@specy/rooc";
    import Column from "$cmp/layout/Column.svelte";
    import Var from "$cmp/pipe/Var.svelte";
    import Row from "$cmp/layout/Row.svelte";
    import Button from "$cmp/inputs/Button.svelte";
    import {textDownloader} from "$lib/utils";
    import {formatNum} from "$cmp/pipe/utils";

    interface Props {
        binarySolution: LpSolution<boolean>;
    }

    let {binarySolution}: Props = $props();

</script>

<Column gap="1rem">
    <div class="table-wrapper">
        <table>
            <thead>
            <tr>
                {#each binarySolution.assignment as assignment}
                    <th>
                        <Var value={assignment.name}/>
                    </th>
                {/each}
            </tr>
            </thead>
            <tbody>
            <tr>
                {#each binarySolution.assignment as assignment}
                    <td
                            class={assignment.value ? 'T' : 'F'}
                    >
                        {assignment.value ? 'T' : 'F'}
                    </td>
                {/each}
            </tr>
            </tbody>
        </table>

    </div>
    <Row justify="between" wrap align="center" gap="0.5rem">
        <div style="font-size: 1.5rem">
            Optimal value: {formatNum(binarySolution.value)}
        </div>
        <Button
                on:click={() => textDownloader(JSON.stringify({
                value: binarySolution.value,
                assignment: Object.fromEntries(binarySolution.assignment.map(({name, value}) => [name, value]))
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