<script lang="ts">
    import Page from '$cmp/layout/Page.svelte';
    import RoocFunctionsDocs from '$cmp/roocDocs/RoocFunctionsDocs.svelte';

</script>

<svelte:head>
    <title>
        Rooc documentation
    </title>
    <meta name="description" content="The documentation of ROOC, with syntax, function definitions and examples"/>
</svelte:head>

<Page cropped padding='1rem' mobilePadding='1rem' gap="1rem">
    <RoocFunctionsDocs/>

</Page>


<style>
    @import "../common.scss";

</style>