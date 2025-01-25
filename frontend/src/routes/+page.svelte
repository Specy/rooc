<script lang="ts">
    import Nav from '$cmp/layout/Nav.svelte';
    import Page from '$cmp/layout/Page.svelte';
    import {onMount} from "svelte";
    import ButtonLink from "$cmp/inputs/ButtonLink.svelte";
    import Icon from "$cmp/layout/Icon.svelte";
    import Button from "$cmp/inputs/Button.svelte";
    import Download from '~icons/fa/Download.svelte';
    import Donate from '~icons/fa/Heart.svelte';
    import Github from "~icons/fa-brands/Github.svelte";
    import Column from "$cmp/layout/Column.svelte";
    import Row from "$cmp/layout/Row.svelte";
    import {projectStore} from "$stores/userProjectsStore.svelte";
    import Book from '~icons/fa/book';
    let installEvent: Event | null = $state(null)
    onMount(() => {
        window.addEventListener('beforeinstallprompt', (e) => {
            e.preventDefault()
            console.log('beforeinstallprompt', e)
            installEvent = e
        })
        projectStore.init()
    })
    let hasProjects = $derived(projectStore.projects.length || !projectStore.initialized)
</script>

<svelte:head>
    <title>Rooc</title>
    <meta name="description" content="A language and platform for easy optimization models."/>
</svelte:head>

<Nav/>
<Page padding="0" gap="1rem">
    <div class="content row">
        <div class="preview-image"></div>
        <div class="presentation">
            <div class="presentation-content">
                <Column gap="1rem">
                    <h1 class="welcome-title textShadow">
                        ROOC Optimization
                    </h1>
                    <div style="text-shadow: 2px 2px 10px black;">
                        A language and platform for easy optimization models.
                    </div>
                    <ul style="max-width: 30rem;">
                        <li>Easy to use optimization modeling language with web editor and solvers</li>
                        <li>
                          Piping system to create your own solving pipeline, with visible intermediate steps to learn how
                          optimization works and debug your models
                        </li>
                        <li>
                          Use the built-in solvers like <a href="https://highs.dev/">HiGHS</a> and <a href="https://github.com/Specy/microlp/">microlp</a> to 
                          find the optimal solution in-browser, or export your model as a Cplex LP model
                        </li>
                    </ul>
                    <div class="buttons">
                        <ButtonLink
                                color="accent"
                                href={hasProjects ? '/projects' : '/projects/new'}
                                style={'box-shadow: 0 3px 10px rgb(0 0 0 / 0.2); padding: 0.5rem 0.6rem; gap: 0.6rem;'}
                                title="Open the editor"
                        >
                            {hasProjects
                            ? "Go to your projects"
                            : "Create your first project"}
                        </ButtonLink>
                        <ButtonLink
                                href={'/docs/rooc'}
                                style={'box-shadow: 0 3px 10px rgb(0 0 0 / 0.2); padding: 0.5rem 0.6rem; gap: 0.6rem;'}
                                title="Docs"
                        >
                          <Book /> Docs
                        </ButtonLink>
                        <Row gap="0.6rem">
                            <ButtonLink
                                    style={'box-shadow: 0 3px 10px rgb(0 0 0 / 0.2); font-size: 1.2rem'}
                                    color="secondary"
                                    href="https://github.com/Specy/rooc"
                                    title="Open the project on github"
                            >
                                <Github/>
                            </ButtonLink>
                            <ButtonLink
                                    style={'box-shadow: 0 3px 10px rgb(0 0 0 / 0.2); font-size: 1.2rem'}
                                    color="secondary"
                                    href="https://specy.app/donate"
                                    title="Donate to the project"
                            >
                                <Donate/>
                            </ButtonLink>
                        </Row>

                    </div>

                    {#if installEvent}
                        <Button
                                style="gap: 0.5rem;"
                                color="secondary"
                                on:click={async () => {
                                  try {
                                    // @ts-ignore
                                    await installEvent.prompt()
                                  } catch (e) {
                                    console.error(e)
                                  }
                                  installEvent = null
                                }}
                        >
                            <Icon>
                                <Download/>
                            </Icon>
                            Install WebApp
                        </Button>
                    {:else}
                        <div style="height: 2.6rem">

                        </div>
                    {/if}
                </Column>
            </div>
        </div>
    </div>
</Page>

<style lang="scss">
  .buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 0.6rem;
  }

  .content {
    display: flex;
    flex: 1;
    overflow: hidden;
    position: relative;
    border-radius: 0.45rem;
  }

  .presentation-content {
    padding: 0 10vw;
  }

  .welcome-title {
    font-size: 3rem;
    color: var(--primary-text);
  }

  .presentation {
    display: flex;
    flex: 1;
    justify-content: flex-end;
    align-items: center;
    background-color: rgba(var(--RGB-primary), 0.9);
    z-index: 2;
  }

  a{
    color: var(--accent-10);
  }
  ul {
    margin-left: 1rem;

  }

  li {
    margin: 0.5rem 0;
    text-shadow: 2px 2px 10px black;
  }

  .preview-image {
    display: flex;
    width: 100%;
    height: 100%;
    top: 0;
    left: 0;
    position: absolute;
    background-image: url('/images/rooc-editor-wide.webp');
    mask: linear-gradient(90deg, rgba(0, 0, 0, 1) 0%, rgba(0, 0, 0, 0.1) 70%);
    filter: blur(1px);
    background-repeat: no-repeat;
    background-position: center;
    background-size: cover;
  }


  .textShadow {
    text-shadow: 2px 2px 12px rgb(36 36 36);
  }

  @media screen and (max-width: 650px) {
    .content {
      width: 100%;
    }
    .welcome-title {
      font-size: 2.4rem;
    }
    .presentation-content {
      padding: 0 1rem;
    }
    .preview-image{
      mask: linear-gradient(0deg, rgba(0, 0, 0, 1) 0%, rgba(0, 0, 0, 0.1) 85%);
    }
  }

</style>
