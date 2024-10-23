<script lang="ts">
    import type {OptimalTableau} from "@specy/rooc";
    import {formatNum} from "$cmp/pipe/utils";
    import Var from "$cmp/pipe/Var.svelte";

    interface Props {
        tableau: OptimalTableau;
    }

    let { tableau }: Props = $props();
    let baseTableau = $derived(tableau.getTableau())

    let vars = $derived(baseTableau.getVariableNames())
    let values = $derived(tableau.getVariablesValues())

    let biggestRhs = $derived(Math.max(tableau.getOptimalValue(), ...baseTableau.getBVector()))
</script>


<div class="table-wrapper">

    <table>
        <thead>
        <tr>
            {#each vars as varName (varName)}
                <th
                        class:slack-surplus={varName.startsWith("$")}
                >
                    <Var value={varName}/>
                </th>
            {/each}

        </tr>

        </thead>
        <tbody>
        <tr>
            {#each values as value, i}
                <th
                        class:slack-surplus={vars[i].startsWith("$")}
                >{formatNum(value)}</th>
            {/each}


        </tr>
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

  .slack-surplus {
    background-color: var(--primary);
  }
</style>

