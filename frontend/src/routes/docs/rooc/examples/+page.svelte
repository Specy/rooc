<script lang="ts">
    import Page from '$cmp/layout/Page.svelte';
    import {roocExamples} from "$cmp/roocDocs/examples";
    import SyntaxHighlighter from "$cmp/SyntaxHighlighter.svelte";
    import Card from "$cmp/layout/Card.svelte";
    import Column from "$cmp/layout/Column.svelte";
</script>

<svelte:head>
    <title>
        Rooc docs - Examples
    </title>
    <meta name="description" content="The documentation of ROOC, with syntax, function definitions and examples"/>
</svelte:head>

<Page cropped padding='1rem' mobilePadding='1rem' gap="1rem">
    <h1>
        Examples
    </h1>
    <p>
        Here are some classic examples of optimization problems that can be solved using ROOC.
    </p>

    <Column gap="4rem">

        {#each roocExamples as code, i}
            <Column gap="1rem">
                <h2>
                    {code.name}
                </h2>
                <p>
                    {code.description}
                </p>
                <Card padding="0.8rem 1rem">
                    <SyntaxHighlighter language="rooc" source={code.code} style="overflow-x: auto;"/>
                </Card>
            </Column>
        {/each}
    </Column>

</Page>


<style>
    @import "../common.scss";

</style>