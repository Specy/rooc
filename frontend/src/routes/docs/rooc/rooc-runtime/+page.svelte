<script lang="ts">
    import Page from '$cmp/layout/Page.svelte';
    import RoocFunctionsDocs from '$cmp/roocDocs/RoocFunctionsDocs.svelte';
</script>

<svelte:head>
    <title>
        Rooc docs - Runtime
    </title>
    <meta name="description" content="Learn about ROOC's runtime capabilities: Use functions to manipulate data within your models. Discover normal functions, block functions, and powerful scoped block functions for complex operations."/>
</svelte:head>

<Page cropped padding='1rem' mobilePadding='1rem' gap="1rem">
    <h1>
        Rooc runtime
    </h1>
    <p>
        Inside Rooc, you can use the functions defined inside of the runtime to perform operations on the data.
        There are 3 kinds of functions:
    </p>
    <ul>
        <li>
            <a href="#functions">Normal functions</a>: They accept an input and return an output
        </li>
        <li>
            <a href="#block_functions">Block functions</a>: They have a block where you can write an expression, and the
            block function will do some operation over the block
        </li>
        <li>
            <a href="#block_scoped_functions">Block scoped functions</a>: Like block functions, but they have an iteration
            scope.
            Inside of the scope you can define how to iterate over the data, and it will be available in the block for
            you to use.
        </li>
    </ul>

    <RoocFunctionsDocs/>

</Page>


<style>
    @import "../common.scss";
</style>