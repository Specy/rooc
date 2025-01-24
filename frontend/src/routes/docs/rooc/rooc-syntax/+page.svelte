<script lang="ts">
    import Page from '$cmp/layout/Page.svelte';
    import RoocSyntaxDocs from '$cmp/roocDocs/RoocSyntaxDocs.svelte';

</script>

<svelte:head>
    <title>
        Rooc docs - Syntax
    </title>
    <meta name="description" content="Learn the ROOC syntax. Explore objective functions, constraints, variables, data handling, expansion blocks, domains, and more with examples."/>
</svelte:head>

<Page cropped padding='1rem' mobilePadding='1rem' gap="1rem">
    <RoocSyntaxDocs/>

</Page>


<style>
    @import "../common.scss";

</style>