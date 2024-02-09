<script lang="ts">
    import Github from '~icons/fa/github.svelte';
    import Burger from '~icons/fa/bars.svelte';
    import Close from '~icons/fa/times.svelte';
    import Book from '~icons/fa/book';
    import Donate from '~icons/fa/dollar';
    import Link from '~icons/fa/external-link.svelte';
    import Row from "$cmp/layout/Row.svelte";
    import Button from "$cmp/inputs/Button.svelte";

    let open = false;
</script>

<div class="nav">
    <a href="/" title="Go to the home page" style="line-height: 0;">
        <img src="/logo.png" class="icon" alt="Rooc logo"/>
    </a>
    <div class="desktop-menu">
        <a href="/projects" title="Go to your projects"> Projects </a>
        <a href="/docs/rooc" title="Go to the docs"> Docs </a>
        <Row style="margin-left: auto;" gap="1rem">
            <a
                    href="https://specy.app/donate"
                    title="Donate to the developer"
                    target="_blank"
            >
                Donate
            </a>
            <a
                    href="https://specy.app"
                    title="Go to my apps"
                    target="_blank"
            >
                Other apps
            </a>
            <a
                    href="https://github.com/Specy/rooc"
                    title="Go to the GitHub repository"
                    target="_blank"
                    class="link-icon"

            >
                <Github/>
                GitHub
            </a>
        </Row>
    </div>
    <div class="mobile-controls">
        <Button hasIcon on:click={() => open = !open} color="primary">
            {#if open}
                <Close/>
            {:else}
                <Burger/>
            {/if}
        </Button>

    </div>
</div>
<div class="mobile-menu" class:mobile-menu-open={open}>
    <a href="/projects" title="Go to your projects"> Projects </a>
    <a href="/docs/rooc" title="Go to the docs" class="link-icon"> <Book /> Docs </a>
    <a
            href="https://specy.app/donate"

            title="Donate to the developer"
            target="_blank"
            class="link-icon"
    >
       <Donate /> Donate
    </a>
    <a
            href="https://specy.app"
            title="Go to my apps"
            target="_blank"
            class="link-icon"
    >
       <Link /> Other apps
    </a>
    <a
            href="https://github.com/Specy/rooc"
            title="Go to the GitHub repository"
            target="_blank"
            class="link-icon"
    >
        <Github/>
        GitHub
    </a>
</div>

<style lang="scss">
  $nav-height: 3rem;
  .nav {
    --nav-height: 3rem;
    display: flex;
    height: $nav-height;
    gap: 1rem;
    z-index: 5;
    background-color: var(--primary);
    color: var(--primary-text);
    padding: 0.5rem;
    border-bottom-left-radius: 0.4rem;
    border-bottom-right-radius: 0.4rem;
    align-items: center;
  }

  .link-icon {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .icon {
    border-radius: 0.4rem;
    width: calc($nav-height - 1rem);
    height: calc($nav-height - 1rem);
  }

  .mobile-menu {
    display: flex;
    opacity: 0;
    pointer-events: none;
    flex-direction: column;
    gap: 0.8rem;
    z-index: 4;
    position: absolute;
    top: $nav-height;
    right: 0;
    width: 100%;
    background-color: rgba(var(--primary-rgb), 0.9);
    backdrop-filter: blur(0.3rem);
    border-top: solid 0.1rem var(--secondary);
    padding: 1rem;
    border-bottom-left-radius: 0.4rem;
    border-bottom-right-radius: 0.4rem;
    transition: opacity 0.3s, transform 0.3s;
    transform: translateY(-1rem);
  }

  .mobile-menu-open {
    pointer-events: all;
    transform: translateY(0);
    opacity: 1;
  }

  .desktop-menu {
    display: flex;
    gap: 1rem;
    flex: 1;
  }

  .mobile-controls {
    display: none;
    margin-left: auto;
  }

  @media (max-width: 768px) {
    .desktop-menu {
      display: none;
    }
    .mobile-controls {
      display: flex;
    }
  }


</style>
