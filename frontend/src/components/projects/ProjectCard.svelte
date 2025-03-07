<script lang="ts">
    import ButtonLink from '$cmp/inputs/ButtonLink.svelte';
    import Card from '$cmp/layout/Card.svelte';
    import Column from '$cmp/layout/Column.svelte';
    import Row from '$cmp/layout/Row.svelte';
    import type {Project} from '$stores/userProjectsStore.svelte';
    import {createEventDispatcher} from 'svelte';
    import sago from 's-ago';
    import Delete from '~icons/fa/Trash.svelte';
    import Download from '~icons/fa/Download.svelte';
    import Button from '$cmp/inputs/Button.svelte';
    import SyntaxHighlighter from '$cmp/SyntaxHighlighter.svelte';

    interface Props {
        project: Project;
    }

    let {project}: Props = $props();
    const dispatcher = createEventDispatcher<{
        onUpdate: { project: Project };
        onDownload: { project: Project };
        onDelete: { project: Project };
    }>();
    let name = $state(project.name);
    let description = $state(project.description);

    function save() {
        dispatcher('onUpdate', {project: {...project, name, description}});
    }

    function detectRoocOrCplex(code: string) {
        if (code.trim().toLowerCase().endsWith("end")) {
            return 'cplex';
        } else {
            return 'rooc';
        }
    }

</script>

<Card
        border="secondary"
        style="position:relative; min-height: 12rem; max-height: 20rem;"
        radius="0.4rem"
>
    <div class="blurred-underlay">
        <SyntaxHighlighter
                source={project.content} language={detectRoocOrCplex(project.content)} style="font-size: 1rem"/>
    </div>
    <Column style="z-index: 2; flex: 1">
        <Column padding="0.5rem; margin-bottom: 0.4rem;" style="flex:1;">
            <div
                    class="title"
                    contenteditable
                    bind:textContent={name}
                    spellcheck="false"
                    onblur={save}
            ></div>
            <div
                    class="description"
                    contenteditable
                    bind:textContent={description}
                    spellcheck="false"
                    onblur={save}
            ></div>
            <div class="project-dates">
                <span>Last update </span> <span>{sago(new Date(project.updatedAt))}</span>
                <span>Created </span> <span>{sago(new Date(project.createdAt))}</span>
            </div>
        </Column>
        <Row
                background="secondary"
                style="border-radius:0 0 0.4rem 0.4rem;"
                padding="0.3rem"
                justify="end"
                gap="0.5rem"
        >
            <Button on:click={() => dispatcher('onDownload', { project })} hasIcon>
                <Download/>
            </Button>
            <Button on:click={() => dispatcher('onDelete', { project })} hoverColor="danger" hasIcon>
                <Delete/>
            </Button>
            <ButtonLink href="/projects/{project.id}" title="Open" border="secondary">Open</ButtonLink>
        </Row>
    </Column>
</Card>

<style lang="scss">
  .blurred-underlay {
    position: absolute;
    margin: 0 0.8rem;
    padding-top: 0.5rem;
    top: 0;
    left: 0;
    width: calc(100% - 1.6rem);
    height: 100%;
    opacity: 0.2;
    filter: blur(0.05rem);
    overflow: hidden;
  }

  .project-dates {
    display: grid;
    padding: 0 0.5rem;
    margin-top: 0.5rem;
    gap: 0.2rem;
    grid-template-columns: 1fr 1fr;
    font-size: 0.8rem;
    color: var(--hint);
  }

  .description {
    padding: 0.5rem;
    background-color: transparent;
    transition: all 0.2s;
    border-radius: 0.4rem;

    &:hover,
    &:focus {
      background-color: var(--primary-5);
      color: var(--secondary-text);
      filter: brightness(1.2);
    }
  }

  .title {
    font-size: 1.3rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    text-align: center;
    padding: 0.4rem;
    justify-content: center;
    border-radius: 0.4rem;
    background-color: transparent;
    transition: all 0.2s;

    &:hover,
    &:focus {
      background-color: var(--primary-5);
      color: var(--secondary-text);
      filter: brightness(1.2);
    }
  }
</style>
