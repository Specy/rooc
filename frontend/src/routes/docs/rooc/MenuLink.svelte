<script lang="ts">
    import FaCircle from "~icons/fa6-solid/circle.svelte"
    import Icon from "$cmp/layout/Icon.svelte";
    import {page} from '$app/stores';

    export let href: string
    export let title: string

    $: current = $page.url.pathname === href;

</script>

<a {href} class="menu-link row" on:click class:current={current}>
    <Icon size={0.4} style="color: var(--accent);">
        <FaCircle/>
    </Icon>
    <h2 class="menu-title">{title}</h2>
</a>

<style lang="scss">
  .menu-title {
    font-size: 1rem;
    font-weight: normal;
  }

  .menu-link {
    gap: 0.9rem;
    padding: 0.7rem;
    align-items: center;
    transition: color 0.2s;
  }

  .menu-link:hover {
    color: var(--accent);
  }

  .current {
    background-color: rgba(var(--accent-rgb), 0.2);
  }
</style>