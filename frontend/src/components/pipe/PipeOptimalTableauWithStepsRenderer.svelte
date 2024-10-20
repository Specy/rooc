<script lang="ts">
    import { run } from 'svelte/legacy';

    import type {OptimalTableauWithSteps} from "@specy/rooc";
    import Column from "$cmp/layout/Column.svelte";
    import Row from "$cmp/layout/Row.svelte";
    import Button from "$cmp/inputs/Button.svelte";
    import PipeOptimalTableauRenderer from "$cmp/pipe/PipeOptimalTableauRenderer.svelte";
    import PipeTableauRenderer from "$cmp/pipe/PipeTableauRenderer.svelte";
    import ChevronRight from '~icons/fa6-solid/chevron-right.svelte';
    import ChevronLeft from '~icons/fa6-solid/chevron-left.svelte';
    import Var from "$cmp/pipe/Var.svelte";

    interface Props {
        data: OptimalTableauWithSteps;
    }

    let { data }: Props = $props();

    let steps = $derived(data.getSteps())
    let result = $derived(data.getResult())
    let current = $state(0)
    $effect(() => {
        if (current > steps.length - 1) {
            current = steps.length - 1
        }
    });
    let currentStep = $derived(steps[current])
    let variables = $derived(currentStep?.getTableau().getVariableNames())
    let pivot = $derived(currentStep?.getPivot())

    function increment() {
        current = Math.min(current + 1, steps.length - 1)
    }

    function decrement() {
        current = Math.max(current - 1, 0)
    }
</script>

<Column gap="1rem">
    <Row justify="between" align="center">
        <Button on:click={decrement} style="padding-left: 0.5rem; gap: 0.5rem" disabled={current === 0}>
            <ChevronLeft/>
            Previous
        </Button>
        <div style="font-weight: bold; font-size: 1.4rem">
            {current + 1} / {steps.length }
        </div>
        <Button on:click={increment} style="padding-right: 0.5rem; gap: 0.5rem" disabled={current >= steps.length - 1}>
            Next
            <ChevronRight/>
        </Button>
    </Row>
    {#if currentStep}
        <PipeTableauRenderer
                tableau={currentStep.getTableau()}
                outIndex={pivot.leaving}
                inIndex={pivot.entering}
        />
        <div>
            Pivoting: variable
            <code style="--bg: var(--danger)"><Var value={variables[pivot.leaving]} /></code>
            exits and
            <code style="--bg: var(--success)"><Var value={variables[pivot.entering]} /></code>
            enters
        </div>
    {/if}

    <div style="font-size: 1.5rem">
        Optimal tableau
    </div>
    <PipeOptimalTableauRenderer tableau={result}/>
</Column>

<style>
    code{
        background-color: var(--bg);
        padding: 0.2rem 0.4rem;
        border-radius: 0.5rem;
        font-weight: bold;
    }
</style>