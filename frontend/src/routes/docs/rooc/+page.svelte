<script lang="ts">
    import Page from '$cmp/layout/Page.svelte';

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
        If you are looking for the typescript library documentation, look <a href="/docs/lib"
                                                                             style="color: var(--accent-10); text-decoration: underline">Here</a>
    </p>
    <h1>
        Getting started
    </h1>
    <p>
        To learn how ROOC works, follow those documentation pages, they will guide you through the syntax and how to create
        your own models.
    </p>
    <ul>
        <li>
            <a href="/docs/rooc/rooc-syntax">ROOC syntax</a>
        </li>
        <li>
            <a href="/docs/rooc/rooc-runtime">ROOC runtime</a>
        </li>
    </ul>

</Page>


<style>
    @import "./common.scss";
    ul{
        margin-left: 1rem;
        font-size: 1.2rem;
    }
</style>