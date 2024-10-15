<script lang="ts">
    import type {SimplexTableau} from "@specy/rooc";
    import {formatNum} from "$cmp/pipe/utils";

    export let tableau: SimplexTableau

    $: a = tableau.getAMatrix()
    $: b = tableau.getBVector()
    $: c = tableau.getCVector()
    $: currentVal = tableau.getCurrentValue()
    $: vars = tableau.getVariableNames()
    $: basis = new Map(tableau.getIndexesOfVarsInBasis().map((i) => [vars[i], true]))
</script>


<div class="table-wrapper">

    <table>
        <thead>
        <tr>

            {#each vars as varName (varName)}
                <th
                        style={basis.has(varName) ? 'color: var(--danger)' : ''}
                >
                    {varName}
                </th>
            {/each}
            <th>

            </th>
        </tr>
        <tr>
            {#each c as value}
                <th>{formatNum(value)}</th>
            {/each}
            <th style="background-color: var(--accent); color: var(--accent-text)">{formatNum(currentVal)}</th>
        </tr>
        </thead>
        <tbody>

        {#each a as row, i}
            <tr>
                {#each row as value}
                    <td>{formatNum(value)}</td>
                {/each}
                <td
                        style="background-color: var(--secondary-10); color: var(--secondary-text)"
                >
                    {formatNum(b[i])}
                </td>
            </tr>
        {/each}
        </tbody>
    </table>
</div>


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
</style>