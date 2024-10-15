<script lang="ts">
    import type {OptimalTableau} from "@specy/rooc";
    import {formatNum} from "$cmp/pipe/utils";

    export let tableau: OptimalTableau
    $: baseTableau = tableau.getTableau()

    $: vars = baseTableau.getVariableNames()
    $: values = tableau.getVariablesValues()

    $: biggestRhs = Math.max(tableau.getOptimalValue(), ...baseTableau.getBVector())
</script>


<div class="table-wrapper">

    <table>
        <thead>
        <tr>
            {#each vars as varName (varName)}
                <th
                        class:slack-surplus={varName.startsWith("$")}
                >
                    {varName}
                </th>
            {/each}
            <th class="slack-surplus">
                <div style="opacity: 0;">
                    {formatNum(biggestRhs)}

                </div>
            </th>
        </tr>

        </thead>
        <tbody>
        <tr>
            {#each values as value, i}
                <th
                        class:slack-surplus={vars[i].startsWith("$")}
                >{formatNum(value)}</th>
            {/each}
            <th class="slack-surplus">

            </th>

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

