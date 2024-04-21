<script lang="ts">
    import {pipeDataDescriptions, PipeDataType, pipeDescriptions, Pipes} from "@specy/rooc";
    import {createEventDispatcher} from "svelte";
    import Card from "$cmp/layout/Card.svelte";
    import Column from "$cmp/layout/Column.svelte";
    import Row from "$cmp/layout/Row.svelte";
    import PlugOut from "$cmp/icons/PlugOut.svelte";
    import PlugIn from "$cmp/icons/PlugIn.svelte";
    import Button from "$cmp/inputs/Button.svelte";
    import Delete from '~icons/fa/Trash.svelte';
    import FaPlus from "~icons/fa/plus";

    export let pipe: Pipes
    export let previousType: PipeDataType

    const dispatcher = createEventDispatcher<{
        delete: void,
        move: "up" | "down",
        'insert-before'
    }>()

    function getNameOfPipeData(ofPipeData: PipeDataType) {
        return pipeDataDescriptions[ofPipeData].name
    }
</script>

<div class="pipe-input">
    <div class="pipe-line-in" style={pipeDescriptions[pipe].input !== previousType ? "background-color: var(--danger)" : ""}>

    </div>
    <div class="pipe-line-out">

    </div>
    <div class="pipe-add-before">
        <Button
                hasIcon
                style="width: 1.6rem; padding: 0; height: 1.6rem"
                color="accent"
                on:click={() => dispatcher('insert-before')}
        >
            <FaPlus/>
        </Button>
    </div>
    <Column gap="0.1rem">
        <div
                class="pipe"
                class:wrong-pipe={pipeDescriptions[pipe].input !== previousType}>
            <Row align="center" gap="1rem">
                <PlugIn style="font-size: 1.2rem; margin-top: -0.3rem;"/>
                <div class="pipe-type">
                    {getNameOfPipeData(pipeDescriptions[pipe].input)}
                </div>
            </Row>
        </div>
        <Card radius="0.4rem" style="z-index: 2; border: solid 0.1rem var(--border-color)">
            <Row
                    style="border-bottom: solid 0.1rem var(--secondary-15);"
            >
                <select
                        bind:value={pipe}
                        class="pipe-select"
                >
                    {#each Object.values(pipeDescriptions) as p}
                        <option value={p.type}>{p.name}</option>
                    {/each}
                </select>
                <Button
                        on:click={() => dispatcher("delete")} color="secondary"
                        style="border-radius: 0; border-top-right-radius: 0.4rem"
                >
                    <Delete/>
                </Button>
            </Row>

            <div class="pipe-description">
                {pipeDescriptions[pipe].description}
            </div>
        </Card>
        <Row align="center" gap="1rem">
            <PlugOut style="font-size: 1.2rem; margin-bottom: -0.3rem; color: var(--border-color)"/>
            <div class="pipe-type">
                {getNameOfPipeData(pipeDescriptions[pipe].output)}
            </div>
        </Row>
    </Column>
</div>


<style>
    .pipe-select {
        background-color: var(--secondary);
        border-top-left-radius: 0.4rem;
        color: var(--primary-text);
        font-size: 1rem;
        padding: 0.5rem 0.7rem;
        flex: 1;
        transition: background-color 0.3s;
        cursor: pointer;
    }

    .pipe-select:hover {
        background-color: var(--secondary-5);
    }

    .pipe-select option {
        background-color: var(--secondary);
        color: var(--primary-text);
        padding: 0.5rem 1rem;
    }

    .pipe-description {
        padding: 0.5rem 1rem;
        font-size: 0.9rem;
    }

    .pipe-type {
        font-size: 0.8rem;
        opacity: 0.8;
    }

      .pipe {
        color: var(--border-color);
    }
      .pipe .pipe-type {
        color: var(--secondary-text);
    }
    .wrong-pipe {
        --border-color: var(--danger);
        color: var(--danger);
        font-weight: bold;
    }
    .wrong-pipe .pipe-type {
        color: var(--danger);
    }



    .pipe-input {
        --border-color: var(--secondary-15);
        position: relative;
    }

    .pipe-add-before {
        position: absolute;
        left: 50%;
        top: -0.8rem;
        transform: translateX(-50%) scale(0.9);
        opacity: 0;
        transition: opacity 0.3s;
        font-size: 0.6rem;

    }

    .pipe-input:hover .pipe-add-before {
        opacity: 1;
        transform: translateX(-50%) scale(1);
    }

    .pipe-line-in {
        position: absolute;
        left: 0.5rem;
        top: 0.3rem;
        width: 0.2rem;
        height: calc(50%);
        background-color: var(--border-color);
    }
    .pipe-line-out{
        position: absolute;
        left: 0.5rem;
        bottom: 0.3rem;
        width: 0.2rem;
        height: calc(50%);
        background-color: var(--border-color);
    }
</style>