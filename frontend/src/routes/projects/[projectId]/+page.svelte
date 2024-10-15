<script lang="ts">
    import Button from '$cmp/inputs/Button.svelte';
    import Page from '$cmp/layout/Page.svelte';
    import {Monaco} from '$src/lib/Monaco';
    import {onMount} from 'svelte';
    import {type Project, projectStore, validateProject} from '$stores/userProjectsStore';
    import {page} from '$app/stores';
    import Row from '$cmp/layout/Row.svelte';
    import ButtonLink from '$cmp/inputs/ButtonLink.svelte';
    import {toast} from '$src/stores/toastStore';
    import FloatingContainer from '$cmp/misc/FloatingContainer.svelte';
    import RoocDocs from '$cmp/roocDocs/RoocFunctionsDocs.svelte';
    import Column from '$cmp/layout/Column.svelte';
    import RoocSyntaxDocs from '$cmp/roocDocs/RoocSyntaxDocs.svelte';
    import ProjectEditor from '$cmp/projects/ProjectEditor.svelte';
    import lzstring from 'lz-string';
    import {prompter} from '$src/stores/promptStore';
    import Github from "~icons/fa-brands/Github.svelte";
    import Floppy from '~icons/fa/floppy-o';
    import Book from '~icons/fa/book';
    import Share from '~icons/fa/share-alt';
    import FaDonate from '~icons/fa6-solid/hand-holding-dollar.svelte';
    let showDocs = false;
    let project: Project | undefined;
    onMount(() => {
        Monaco.load();
        loadProject();
        return () => {
            Monaco.dispose();
        };
    });

    async function loadProject() {
        const id = $page.params.projectId;
        if (id === 'share') {
            const code = $page.url.searchParams.get('project');
            const parsed = JSON.parse(lzstring.decompressFromEncodedURIComponent(code));
            parsed.id = 'share';
            project = validateProject(parsed)
            return
        }
        project = await projectStore.getProject($page.params.projectId);
        if (!project) {
            toast.error('Project not found', 10000);
            return;
        }
    }

    async function save() {
        if (!project) return;
        try {
            if (project.id === 'share') {
                if (!(await prompter.confirm('Do you want to save this shared project in your projets?')))
                    return;
                delete project.id;
                const newProject = await projectStore.createNewProject(project.name, project.description);
                project.id = newProject.id;
                toast.logPill('Project added to your projects');
            }
            await projectStore.updateProject(project.id, project);
        } catch (e) {
            toast.error("Couldn't save project");
            console.error(e);
        }
    }


    function share() {
        if (!project) return;
        const p = {...project};
        p.id = 'share';
        const code = lzstring.compressToEncodedURIComponent(JSON.stringify(p));
        const url = `${window.location.origin}/projects/share?project=${code}`;
        navigator.clipboard.writeText(url);
        toast.logPill('Copied to clipboard');
    }
    function createDebouncer() {
        let timeout: number;
        return (fn: () => void, delay: number) => {
            clearTimeout(timeout);
            timeout = setTimeout(fn, delay);
        };
    }
    const debouncer = createDebouncer();
    $: {
        if(project && project?.id !== 'share'){
            console.log('save')
            debouncer(() => save(), 1000);
        }
    }
</script>

<svelte:head>
    <title>{project?.name ?? 'Project'} - Rooc</title>
    <meta name="description" content="Edit your Rooc project"/>
</svelte:head>

<Page style="min-height: 100vh;">
    <Row justify="between" padding="0.5rem" gap="0.5rem" align="center">
        <ButtonLink href="/projects">Projects</ButtonLink>

        <h3 class="clamp-text">
            {project?.name ?? 'Project'}
        </h3>

        <Row gap="0.5rem" style="margin-left: auto; height: 2.4rem">
            <ButtonLink
                    href="https://github.com/Specy/tokeko"
                    hasIcon
                    style="height: 100%; font-size: 1.2rem"
                    blank
                    title="Github"
            >
                <Github/>
            </ButtonLink>
            <ButtonLink
                    href="https://specy.app/donate"
                    hasIcon
                    style="height: 100%; font-size: 1.2rem"
                    blank
                    title="Donate"
            >
                <FaDonate/>
            </ButtonLink>
            <Button
                    hasIcon
                    on:click={share}
                    title="Share"
            >
                <Share/>
            </Button>
            <Button
                    hasIcon
                    on:click={() => (showDocs = !showDocs)}
                    title="Documentation"
            >
                <Book/>
            </Button>
            {#if project?.id === 'share'}
                <Button on:click={save} hasIcon color="accent">
                    <Floppy/>
                </Button>
            {/if}
        </Row>
    </Row>
    {#if project}
        <ProjectEditor bind:project/>
    {:else}
        <div class="col justify-center align-center flex-1">
            <h1>Loading...</h1>
        </div>
    {/if}
    <FloatingContainer bind:visible={showDocs} title="Documentation">
        <Column
                style="width: 45rem; max-width: calc(100vw - 1rem); max-height: calc(80vh - 1.2rem); overflow-y: auto;"
                padding="0.8rem"
                gap="0.5rem"
        >
            <RoocDocs/>
            <RoocSyntaxDocs/>
        </Column>
    </FloatingContainer>
</Page>


<style>
    .clamp-text {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    :global(html) {
        overflow-y: scroll;
    }
</style>