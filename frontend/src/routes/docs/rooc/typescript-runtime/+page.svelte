<script lang="ts">
    import Nav from '$cmp/layout/Nav.svelte';
    import Page from '$cmp/layout/Page.svelte';
    import RoocFunctionsDocs from '$cmp/roocDocs/RoocFunctionsDocs.svelte';
    import RoocSyntaxDocs from '$cmp/roocDocs/RoocSyntaxDocs.svelte';
    import lzstring from "lz-string";
    import {createProject} from "$stores/Project";

    const code = lzstring.compressToEncodedURIComponent(JSON.stringify(createProject()));
    const url = `/projects/share?project=${code}`;
</script>

<svelte:head>
    <title>
        Rooc documentation
    </title>
    <meta name="description" content="The documentation of ROOC, with syntax, function definitions and examples"/>
</svelte:head>

<Page cropped padding='1rem' mobilePadding='1rem' gap="1rem">
    <h1>
        This docs page is under construction
    </h1>
</Page>


<style>
    @import "../common.scss";

</style>