<script lang="ts">
    import type {SimplexTableau} from "@specy/rooc";
    import {formatNum} from "$cmp/pipe/utils";
    import Var from "$cmp/pipe/Var.svelte";

    interface Props {
        tableau: SimplexTableau;
        outIndex?: number | undefined;
        inIndex?: number | undefined;
    }

    let { tableau, outIndex = undefined, inIndex = undefined }: Props = $props();
    let a = $derived(tableau.getAMatrix())
    let b = $derived(tableau.getBVector())
    let c = $derived(tableau.getCVector())
    let currentVal = $derived(tableau.getCurrentValue())
    let vars = $derived(tableau.getVariableNames())
    let basis = $derived(tableau.getIndexesOfVarsInBasis())
    let basisMap = $derived(new Map(basis.map((i) => [vars[i], true])))
</script>


<div class="table-wrapper">

    <table>
        <thead>
        <tr>
            {#each vars as varName,i (varName)}
                <th
                        class:in-basis={basisMap.has(varName)}
                        class:entering={inIndex === i}
                        class:exiting={outIndex === i}
                >
                    <Var value={varName}/>
                </th>
            {/each}
            <th>

            </th>
            <th>

            </th>
        </tr>
        <tr>
            {#each c as value}
                <th>{formatNum(value)}</th>
            {/each}
            <th style="background-color: var(--accent); color: var(--accent-text)">{formatNum(currentVal)}</th>
            <th>
                Basis
            </th>
        </tr>
        </thead>
        <tbody>

        {#each a as row, i}
            <tr>
                {#each row as value, j}
                    <td
                        class:pivoting-row={inIndex === basis[i] }
                    >{formatNum(value)}</td>
                {/each}
                <td
                        style="background-color: var(--secondary-10); color: var(--secondary-text)"
                >
                    {formatNum(b[i])}
                </td>
                <td>
                    <Var value={vars[basis[i]]} />
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
    padding: 0.3rem 0.5rem;

    text-align: center;
    border: solid 0.1rem var(--secondary-5);
  }

  th {
    font-weight: bold;
  }


  .in-basis {
    color: var(--warn);
  }

  .entering {
    background-color: var(--success);
    color: var(--success-text)
  }


  .exiting {
    background-color: var(--danger);
    color: var(--danger-text)

  }
  .pivoting-row{
    background-color: var(--success);
    color: var(--success-text)
  }

</style>