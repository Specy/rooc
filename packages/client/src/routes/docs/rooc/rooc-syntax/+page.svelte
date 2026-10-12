<script lang="ts">
    import PageSeo from '$cmp/seo/PageSeo.svelte';
    import Page from '$cmp/layout/Page.svelte';
    import RoocSyntaxDocs from '$cmp/roocDocs/RoocSyntaxDocs.svelte';

</script>

<PageSeo
    title={"ROOC syntax: objectives, constraints and variables"}
    headline={"Syntax"}
    description={"Learn the ROOC syntax. Explore objective functions, constraints, variables, data handling, expansion blocks, domains, and more with examples."}
/>

<Page cropped padding='1rem' mobilePadding='1rem' gap="1rem">
    <h1>
        ROOC syntax
    </h1>
    <RoocSyntaxDocs/>

</Page>


<style>
    @import "../common.scss";

</style>