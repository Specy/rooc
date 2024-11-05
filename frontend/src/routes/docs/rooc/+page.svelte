<script lang="ts">
    import Page from '$cmp/layout/Page.svelte';
    import {onMount} from "svelte";

    let link = $state(null)
    onMount(() => {
        //hack to disable prerendering error
        link = "/docs/lib"
    })
</script>

<svelte:head>
    <title>
        Rooc documentation
    </title>
    <meta name="description" content="The documentation of ROOC, with syntax, function definitions and examples"/>
</svelte:head>

<Page cropped padding='1rem' mobilePadding='1rem' gap="1rem">
    <h1>
        ROOC documentation
    </h1>
    <p>
        ROOC is a language designed to parse and convert formal optimization models into static formulations. These
        static formulations can be transformed into linear models which can then be solved using optimization
        techniques.
    </p>
    <p>
        If you are looking for the typescript library documentation, look <a href={link}
                                                                             target="_blank"
                                                                             style="color: var(--accent-10); text-decoration: underline">Here</a>
    </p>
    <p>
        If you are looking for the rust library documentation, look <a href="https://docs.rs/rooc/latest/rooc/" target="_blank"
                                                                       style="color: var(--accent-10); text-decoration: underline">Here</a>
    </p>
    <h1>
        Getting started
    </h1>
    <p>
        To learn how ROOC works, follow those documentation pages, they will guide you through the syntax and how to
        create
        your own models.
    </p>
    <ul>
        <li>
            <a href="/docs/rooc/rooc-syntax">Rooc syntax</a>
        </li>
        <li>
            <a href="/docs/rooc/rooc-runtime">Rooc runtime</a>
        </li>
        <li>
            <a href="/docs/rooc/rooc-runtime">Typescript runtime</a>
        </li>
        <li>
            <a href="/docs/rooc/examples">Rooc examples</a>
        </li>
    </ul>
    <h1>
        Contributing
    </h1>
    <p>
        If you want to contribute to the project, you can find the source code on <a
            href="https://github.com/Specy/rooc" style="color: var(--accent-10); text-decoration: underline">Github</a>.
        <br/>
    </p>
</Page>


<style>
    @import "./common.scss";

    a {
        color: var(--accent-10);
    }

    ul {
        font-size: 1.2rem;
    }
</style>