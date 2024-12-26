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
        ROOC is a language designed to help you formalize problems, and prepares those problems in linear form to be solved. The ROOC environment
        does this in 3 steps:
    </p>
    <ol>
        <li>
        Compile-and-parse step helps formalize your problem into a pre-model (e.g. a LaTeX expression)
        </li>
        <li>
        Transform step converts the formal problem into models you can revisit later.
        </li>
        <li>
        Runtime step will run your linear model against solvers to find the optimal solution.
        </li>
    </ol>
    <p>
        You only need to focus on formalising your problem in the first step and the ROOC environment is able to handle the rest. You can always
        revisit and revise your models later if needed.
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
    Picture your team dealing with limited resources and an ever-growing workload. It's not long before you're searching for an SaaS like Linear app to help divide up the work and track overlap between responsibilities and problems.
    </p>
    <p>
    We often want to track such overlaps so that we can find two kinds of solutions:
    </p>
    <ol>
        <li>Optimal minimum solution to help run as lean as possible in early stages.</li>
        <li>Optimal maximum solution to reap the most benefits at mature stages.</li>
    </ol>
    <p>
    Instead of paying for external service to crunch these solutions, you can think of ROOC as the toolkit to approach building your own in-house service.
    </p>
    <p>ROOC won't mockup the service for you, but its output gives you the common ground to help identify and agree on problems before committing to the work needed to try achieving an optimal outcome.
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