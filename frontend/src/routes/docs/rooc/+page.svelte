<script lang="ts">
    import Nav from '$cmp/layout/Nav.svelte';
    import Page from '$cmp/layout/Page.svelte';
    import RoocFunctionsDocs from '$cmp/roocDocs/RoocFunctionsDocs.svelte';
    import RoocSyntaxDocs from '$cmp/roocDocs/RoocSyntaxDocs.svelte';
    import lzstring from "lz-string";
    import {createProject} from "$stores/userProjectsStore.svelte";

    const code = lzstring.compressToEncodedURIComponent(JSON.stringify(createProject()));
    const url = `/projects/share?project=${code}`;
</script>

<svelte:head>
    <title>
        Rooc documentation
    </title>
    <meta name="description" content="The documentation of ROOC, with syntax, function definitions and examples"/>
</svelte:head>

<Nav/>
<Page cropped padding='1rem' mobilePadding='1rem' gap="1rem">
    <RoocSyntaxDocs/>
    <RoocFunctionsDocs/>
    <a href="{url}" class="tryit">
        Try ROOC
    </a>
</Page>


<style>
    .tryit {
        border-radius: 0.8rem;
        width: 100%;
        background-color: var(--accent);
        color: var(--accent-text);
        padding: 0.5rem 1rem;
        text-align: center;
    }
</style>