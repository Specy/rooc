<script lang="ts">
    import type {BinarySolution} from "@specy/rooc";
    import Column from "$cmp/layout/Column.svelte";

    export let binarySolution: BinarySolution

</script>

<Column gap="1rem">

    <div class="table-wrapper">
        <table>
            <thead>
            <tr>
                {#each binarySolution.assignment as assignment}
                    <th>
                        {assignment.name}
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
    <div style="font-size: 1.5rem">
        Optimal value: {binarySolution.value}
    </div>
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
  .T{
    color: var(--success);
    font-weight: bold;
  }
  .F{
    color: var(--danger);
    font-weight: bold;

  }
</style>