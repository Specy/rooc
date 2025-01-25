<script lang="ts">
    import Page from '$cmp/layout/Page.svelte';
</script>

<svelte:head>
    <title>
        Rooc docs - Solvers
    </title>
    <meta name="description" content="Pick the right solver for your ROOC optimization problem. Learn about different solvers and their capabilities (HiGHS, Auto, MILP, Real, Binary, Integer & binary, Simplex)."/>
</svelte:head>

<Page cropped padding='1rem' mobilePadding='1rem' gap="1rem">
    <h1>
        Solvers
    </h1>
    <p>
        ROOC allows you to use different kinds of solvers depending on the problem you are trying to
        solve. <br/> <br/>
        The problems you write might have different types of variables (Real, Integer, Binary), and not all solvers can
        use all of them. <br/>
        You can quickly select which solver you want to use by choosing the "preset" next to the "run" button, or create
        your own pipeline. <br/>
        Here are the main requirements and differences for each solver:
    </p>
    <ul>
        <li><strong>HiGHS solver</strong>: It uses the <a href="https://highs.dev/">HiGHS</a> solver, the most complete
            and performant solver in ROOC. It allows you to solve any MILP problem, so problems with any kind of
            variables. (Recommended)
        </li>
        <li><strong>Auto solver</strong>: It picks the most appopriate solver depending on the problem you are trying to
            solve, it is the default.
        </li>
        <li><strong>MILP solver</strong>: It uses the <a href="https://github.com/Specy/microlp/">microlp</a>
            solver, a simple solver for MILP problems (like HiGHS), slower than HiGHS.
        </li>
        <li><strong>Real solver</strong>: It uses the <a href="https://clarabel.org/stable/">Clarabel</a> solver, a
            solver
            for models that use only Real or NonNegativeReal variables.
        </li>
        <li><strong>Binary solver</strong>: It uses the <a href="https://github.com/ffminus/copper">Copper</a> solver, a
            constraint
            programming solver that can solve problems with binary variables. All coefficients must be integers (they
            will be
            rounded otherwise).
        </li>
        <li><strong>Integer & binary solver</strong>: It uses the <a href="https://github.com/ffminus/copper">Copper</a>
            solver,
            a constraint programming solver that can solve problems with binary and integer variables. All coefficients
            must be
            integers (they will be rounded otherwise).
        </li>
        <li><strong>Simplex solver Step by Step</strong>: It uses the internal rooc solver to show how to solve a linear
            problem by using the simplex method step by step. It might be useful to understand how the simplex method
            works, but is not
            reccomended to actually solve problems as it might have errors and is slow.
        </li>
    </ul>
    <p>
        Alternatively, if you don't want to use the ROOC syntax to create the model, you can use the <a
            href="https://www.ibm.com/docs/en/icos/22.1.2?topic=cplex-lp-file-format-algebraic-representation">CPLEX
        LP</a> format, which is a standard format to represent linear problems. And then you can choose the HiGHS CPLEX
        solver to solve it by using the HiGHS solver.
    </p>
    <p>
        You can also write the model using the ROOC syntax, to then convert it to the CPLEX LP format, this way you can
        use any other
        solver that supports the CPLEX LP format.
    </p>

</Page>


<style>
    @import "../common.scss";

    a {
        color: var(--accent-10);
    }

    ul {
        font-size: 1.1rem;
    }
    li{
        margin: 0.5rem 0;
    }

    strong{
        color: var(--accent-10);
    }
</style>