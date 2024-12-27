<script>
    import Column from "$cmp/layout/Column.svelte";
    import Row from "$cmp/layout/Row.svelte";
    import MenuLink from "./MenuLink.svelte";
    import Button from "$cmp/inputs/Button.svelte";
    import lzstring from "lz-string";
    import FaChevron from "~icons/fa6-solid/chevron-left.svelte";
    import {createProject} from "$stores/Project";
    import Nav from "$cmp/layout/Nav.svelte";

    const p = createProject();
    const s = lzstring.compressToEncodedURIComponent(JSON.stringify(p));
    const url = `/projects/share?project=${s}`;
    let menuOpen = false
</script>
<Nav/>
<Row gap="1rem" flex1>
    <button
            class="side-menu-underlay"
            class:side-menu-underlay-open={menuOpen}
            on:click={() => (menuOpen = false)}
    >

    </button>
    <aside class="side-menu col" class:menu-open={menuOpen}>
        <div class="mobile-only side-menu-open-btn">
            <Button
                    hasIcon
                    on:click={() => (menuOpen = !menuOpen)}
                    style="
                    border-top-left-radius: 0;
                    border-bottom-left-radius: 0;
                    padding: 0.6rem;
                    background-color: rgba(var(--accent-rgb), 0.8);
                    backdrop-filter: blur(0.1rem);
                    "
            >
                <FaChevron
                        style={`transition: all 0.3s;transform: rotate(${menuOpen ? '0deg' : '180deg'})`}
                />
            </Button>
        </div>
        <Column flex1>
            <MenuLink
                    href="/docs/rooc"
                    title="Introduction"
                    on:click={() => (menuOpen = false)}
            />
            <MenuLink
                    href="/docs/rooc/what-are-optimization-models"
                    title="What are optimization models"
                    on:click={() => (menuOpen = false)}
            />
            <MenuLink
                    href="/docs/rooc/rooc-syntax"
                    title="Rooc syntax"
                    on:click={() => (menuOpen = false)}
            />
            <MenuLink
                    href="/docs/rooc/rooc-runtime"
                    title="Rooc runtime"
                    on:click={() => (menuOpen = false)}
            />
            <MenuLink
                    href="/docs/rooc/typescript-runtime"
                    title="Typescript runtime"
                    on:click={() => (menuOpen = false)}
            />
             <MenuLink
                    href="/docs/rooc/examples"
                    title="Rooc examples"
                    on:click={() => (menuOpen = false)}
            />
            <Row padding="0.8rem" style="margin-top: auto">
                <a href="{url}" class="tryit">
                    Try it
                </a>
            </Row>
        </Column>

    </aside>
    <div class="mock">

    </div>
    <Column style="padding-top: 1rem; width: 100%; overflow-x: hidden;">
        <slot/>
        <Row padding="1rem" justify="center">
            <a href="{url}" class="tryit-wide">
                Try ROOC
            </a>
        </Row>

    </Column>
</Row>

<style lang="scss">

  .side-menu {
    position: fixed;
    background-color: var(--primary);
    color: var(--secondary-text);
    width: 15rem;
    gap: 1rem;
    top: 3rem;
    border-top: solid 0.2rem var(--secondary);
    height: calc(100vh - 3.2rem);
    z-index: 10;
  }

  .mock {
    width: 15rem;
    min-width: 15rem;
  }

  .tryit-wide {
    border-radius: 0.8rem;
    width: 100%;
    max-width: 60rem;
    background-color: var(--accent);
    color: var(--accent-text);
    padding: 0.5rem 1rem;
    text-align: center;
  }

  .tryit {
    border-radius: 0.8rem;
    width: 100%;
    background-color: var(--accent);
    color: var(--accent-text);
    padding: 0.5rem 1rem;
    text-align: center;
  }

  .mobile-only {
    display: none;
  }

  .side-menu-open-btn {
    margin-right: -2.15rem;
    justify-content: flex-end;
  }

  @media (max-width: 900px) {
    .side-menu {
      position: fixed;
      width: calc(100vw - 4rem);
      left: 0;
      z-index: 5;
      transition: transform 0.3s;
      transform: translateX(-100%);
      background-color: rgba(var(--primary-rgb), 0.9);
    }
    .mobile-only {
      display: flex;
    }
    .menu-open {
      transform: translateX(0);
    }
    .mock {
      display: none;
    }
  }

  .instruction-search {
    background-color: var(--tertiary);
    color: var(--tertiary-text);
    padding: 0.6rem;
    border-radius: 0.4rem;
  }

  .icon {
    height: 2.2rem;
    display: flex;
    align-items: center;
    gap: 1rem;

    img {
      height: 100%;
    }

    &:hover {
      color: var(--accent)
    }
  }

  .side-menu-underlay {
    position: fixed;
    top: 3.2rem;
    left: 0;
    width: 100vw;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    opacity: 0;
    pointer-events: none;
    cursor: pointer;
    z-index: 3;
    transition: opacity 0.3s;
    backdrop-filter: blur(0.2rem);
  }

  .side-menu-underlay-open {
    opacity: 1;
    pointer-events: all;
  }

</style>