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
        ROOC is a language designed to help you formalize problems and solve them through optimization models. You only need to focus on formalizing your problem to get started; the ROOC environment is able to handle the rest.
    </p>
    <h1>
        Getting started
    </h1>
    <p>
        If you are looking for the typescript library documentation, look <a href={link}
                                                                             target="_blank"
                                                                             style="color: var(--accent-10); text-decoration: underline">Here</a>
    </p>
    <p>
        If you are looking for the rust library documentation, look <a href="https://docs.rs/rooc/latest/rooc/" target="_blank"
                                                                       style="color: var(--accent-10); text-decoration: underline">Here</a>
    </p>
    <p>
        These documentation pages are structured to help learn how ROOC works, guiding you through the syntax and how to create your own models.
    </p>
    <ul>
        <li>
            <a href="/docs/rooc/what-are-optimization-models">What are optimization models?</a>
        </li>
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

    <h1>Why Would You Use ROOC?</h1>
    <p>
    Picture any scenario where you're faced with making the best of limited resources.
    There any are number of domains where we often face these kind of decisions, but we'll start with a trip for groceries.
    </p>
    <p>
    You have a $50 store coupon and need to buy your groceries for the week. Your shopping list includes:
    </p>
    <ul>
         <li>
         At least 3 fruits
        </li>
        <li>
        At least 2 vegetables
        </li>
        <li>
        At least 1 dairy product
        </li>
    </ul>
    <p>
        Each item has a cost:
    </p>
    <ul>
        <li>
            Apples: $4
        </li>
        <li>
            Bananas: $3
        </li>
        <li>
            Carrots: $2
        </li>
        <li>
            Spinach: $3
        </li>
        <li>
            Milk: $5
        </li>
        <li>
            Cheese: $6
        </li>
    </ul>
    <p>
        Your goal is to select items that meet the requirements and spend at least the full coupon, but you want to minimize spending more than the coupon value.
        <br/> With ROOC you can formalize and solve this problem.
    </p>
    <h1>
        Contributing
    </h1>
    <p>
        If you want to contribute to the project, you can find the source code on <a
            href="https://github.com/Specy/rooc" style="color: var(--accent-10); text-decoration: underline">Github</a>.
        <br/>
    </p>

    <h1>
        Limitations
    </h1>
    <p>
        The maximum length of an expression is about 6000 elements, so big models might give errors when compiling
        if they are too big.
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