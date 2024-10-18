<script lang="ts">
    import Editor from '$cmp/editor/Editor.svelte';
    import Button from '$cmp/inputs/Button.svelte';
    import {Monaco} from '$src/lib/Monaco';
    import {onMount} from 'svelte';
    import {type Project} from '$stores/userProjectsStore';
    import Row from '$cmp/layout/Row.svelte';
    import {createCompilerStore} from '$src/routes/projects/[projectId]/projectStore';
    import {PipeDataType, pipeDescriptions, Pipes} from "@specy/rooc";
    import PipeInput from "$cmp/pipe/PipeInput.svelte";
    import Card from "$cmp/layout/Card.svelte";
    import PlugOut from "$cmp/icons/PlugOut.svelte";
    import PlugIn from "$cmp/icons/PlugIn.svelte";
    import Column from "$cmp/layout/Column.svelte";
    import {findPreset, pipePresets} from "$lib/pipePresets";
    import {prompter} from '$stores/promptStore'
    import PipeResultRenderer from "$cmp/pipe/PipeResultRenderer.svelte";
    import FaPlus from '~icons/fa/plus.svelte'
    import LatexRenderer from "$cmp/LatexRenderer.svelte";
    import ExpandableContainer from "$cmp/layout/ExpandableContainer.svelte";
    import {toast} from "$stores/toastStore";

    export let project: Project;
    let {rooc, source, result, compiling} = createCompilerStore(project);
    onMount(() => {
        Monaco.load();
        return () => {
            Monaco.dispose();
        };
    });

    function run() {
        rooc?.run();
        setTimeout(() => {
            const element = document.getElementById('jump-to');
            if (element) {
                element.scrollIntoView({behavior: 'smooth'});
            }
        }, 100);
    }

    function reset() {
        rooc?.reset();
    }


    $: $source.source = project.content;
    $: $source.pipes = project.pipes;
    /*
            <Card style="flex: 1; position:relative; overflow: hidden; max-height: 42vh; height: 42vh">
            <LatexRenderer source={$store.latex} style="flex:1; overflow: auto; padding: 0.5rem 1rem;" />
            <Button
                hasIcon
                style="position:absolute; top: 0.5rem; right: 0.5rem; width: 2.4rem; height: 2.4rem; padding: 0"
                on:click={copy}
            >
                <FaCopy />
            </Button>
        </Card>
        <Editor
            style="flex: 1;"
            language="rooc"
            code={$store.compilationError ?? $store.compiled ?? ''}
            config={{
                readOnly: true,
                lineNumbers: 'off'
            }}
            disabled
            highlightedLine={-1}
        />


     */

    $: isPresetPipe = findPreset(project.pipes.map(p => p.pipe))
</script>

<div class="wrapper">
    <Editor
            style="flex: 1; height: 100%;"
            language="rooc"
            bind:code={project.content}
            highlightedLine={-1}
    />
    <div class="pipe-container">
        <div class="pipe-container-inner">

            <Card background="secondary" radius="0.4rem" padding="0.5rem 1rem">
                Source code
            </Card>
            <Row align="center" gap="1rem">
                <PlugOut style="font-size: 1.2rem; margin-bottom: -0.4rem"/>
                <div style="font-weight: bold; font-size: 0.8rem">
                    String
                </div>
            </Row>
            {#each project.pipes as pipe, i}
                <PipeInput
                        bind:pipe
                        on:delete={() => project.pipes  = project.pipes .filter((_, index) => index !== i)}
                        on:insert-before={() => project.pipes  = [...project.pipes .slice(0, i), {pipe: Pipes.CompilerPipe, open: false}, project.pipes [i], ...project.pipes .slice(i + 1)]}
                        previousType={i === 0 ? PipeDataType.String : pipeDescriptions[project.pipes[i - 1].pipe].output}
                />
            {/each}
            <Column gap="0.2rem">
                <Row gap="1rem">
                    <PlugIn style="font-size: 1.2rem; margin-top: -0.2rem"/>
                    <div style="font-weight: bold; font-size: 0.8rem ">
                        Destination
                    </div>
                </Row>

                <button
                        on:click={() => project.pipes = [...project.pipes, {pipe: Pipes.CompilerPipe, open: false}]}
                        class="add-more-btn"
                >
                    <FaPlus/>
                    Add to pipe
                </button>
            </Column>
        </div>
        <Row justify="between" gap="0.5rem" wrap>
            <select
                    class="pipe-preset-select"
                    on:change={async (e) => {
                            if (e.target.value === 'custom') return;

                            if(isPresetPipe || await prompter.confirm('This will overwrite your current pipe. Are you sure?')){
                                project.pipes = (pipePresets
                                    .find(p => p.name === e.target.value)?.pipes ?? [])
                                    .map((p, i, arr) => ({pipe: p, open: i === arr.length - 1}))
                            }else {
                                e.target.value = 'custom';
                            }
                        }}
                    value={isPresetPipe?.name ?? "custom"}
            >
                <option
                        value="custom"
                        disabled
                        selected={!isPresetPipe}
                >
                    Custom
                </option>
                {#each pipePresets as preset}
                    <option value={preset.name}>{preset.name}</option>
                {/each}
            </select>
            <Row gap="0.5rem">
                {#if $result}
                    <Button on:click={reset} border="secondary" color="primary">Reset</Button>
                {/if}
                <Button
                        on:click={run}
                        border="secondary"
                        color="primary"
                        disabled={$compiling}
                >
                    {$compiling ? 'Running...' : 'Run'}
                </Button>
            </Row>
        </Row>

    </div>
</div>
{#if $result}
    <Column padding="0.5rem" gap="0.5rem">
        <h1 id="jump-to">
            {$result.ok ? "Execution successful" : "Execution failed"}
        </h1>
        {#if $result.latex}
            <ExpandableContainer>
                <h2 slot="title">
                    LaTeX
                </h2>
                <Column style="position: relative;" gap="0.5rem">
                    <LatexRenderer
                            source={$result.latex}
                            style="overflow-y: auto; overflow-x: auto; max-height: 50vh"
                    />
                    <Button on:click={() => {
                        navigator.clipboard.writeText($result.latex);
                        toast.logPill('LaTeX source copied to clipboard')
                    }}>
                        Copy
                    </Button>
                </Column>
            </ExpandableContainer>
        {/if}
        {#if $result.ok}
            <Column gap="0.5rem">
                {#each $result.val as step, i}
                    {#if i === 0}
                        <PipeResultRenderer
                                data={{type: PipeDataType.String, data: $source.source}}
                                pipeStep="Source"
                        />
                    {:else }
                        {#if project.pipes[i - 1]}
                            <PipeResultRenderer
                                    pipeStep={project.pipes[i - 1].pipe}
                                    bind:expanded={project.pipes[i - 1].open}
                                    data={step}
                            />
                        {:else}
                            <PipeResultRenderer
                                    pipeStep={"Unknown"}
                                    expanded={false}
                                    data={step}
                            />
                        {/if}

                    {/if}
                {/each}
            </Column>
        {:else}
            <Column gap="0.5rem">
                {#each $result.context as step, i}
                    {#if i === 0}
                        <PipeResultRenderer
                                data={{type: PipeDataType.String, data: $source.source}}
                                pipeStep="Source"
                        />
                    {:else }
                        {#if project.pipes[i - 1]}
                            <PipeResultRenderer
                                    pipeStep={project.pipes[i - 1].pipe}
                                    data={step}
                                    bind:expanded={project.pipes[i - 1].open}
                            />
                        {:else}
                            <PipeResultRenderer
                                    pipeStep={"Unknown"}
                                    expanded={false}
                                    data={step}
                            />
                        {/if}
                    {/if}
                {/each}
            </Column>

            <Card
                    style="background-color: rgba(var(--danger-rgb), 0.2); border: solid 0.2rem rgba(var(--danger-rgb), 0.5);"
                    padding="1rem"
            >
                <pre style="overflow-x: auto">{$result.error}</pre>
            </Card>
        {/if}
    </Column>
{/if}

<style>
    .pipe-container {
        display: flex;
        flex: 1;
        flex-direction: column;
        overflow: hidden;
        gap: 0.5rem;
    }

    .pipe-container-inner {
        background-color: var(--primary);
        padding: 0.5rem;
        border-radius: 0.4rem;
        overflow-y: auto;
        height: 100%;
    }

    .add-more-btn {
        display: flex;
        justify-content: center;
        align-items: center;
        border-radius: 0.4rem;
        cursor: pointer;
        gap: 0.5rem;
        padding: 0.5rem 1rem;
        background-color: var(--primary);
        color: var(--primary-text)
    }

    .wrapper {
        display: grid;
        grid-template-columns: 4fr 2fr;
        gap: 0.5rem;
        height: calc(100vh - 3.5rem - 0.5rem);
        padding: 0 0.5rem;
    }

    @media (max-width: 768px) {
        .wrapper {
            display: flex;
            flex-direction: column;
        }
    }

    .pipe-preset-select {
        padding: 0.5rem 1rem;
        border-radius: 0.4rem;
        border: 1px solid var(--primary);
        background-color: var(--primary);
        color: var(--primary-text);
    }
</style>
