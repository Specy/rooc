<script lang="ts">
    import Page from '$cmp/layout/Page.svelte';
    import SyntaxHighlighter from '$cmp/SyntaxHighlighter.svelte';
    import Card from '$cmp/layout/Card.svelte';
    import Column from '$cmp/layout/Column.svelte';
    import Separator from '$cmp/misc/Separator.svelte';

    const vertexCover = `
min sum(v in nodes(G)) { x_v }
s.t.
    //at least one endpoint of each edge must be selected
    x_u or x_v for (u, v) in edges(G)
where
    let G = Graph { 
        A -> [ B, C ], 
        B -> [ A ], 
        C -> [ A ] 
    }
define
    x_v as Boolean for v in nodes(G)
`.trim();

    const allBlock = `
solve
s.t.
    all {
        a or b,
        b implies c,
        not (a xor c)
    }
define
    a, b, c as Boolean
`.trim();

    const maxSat = `
max (a or b) + (not a or c) + (b or not c) + (not b)
s.t.
    a or b or c
define
    a, b, c as Boolean
`.trim();

    const mixed = `
max x + y
s.t.
    //production is only possible in an open facility
    x <= 10 * open_1
    y <= 10 * open_2
    //exactly one of the two facilities can be open
    open_1 xor open_2
    x + y <= 12
define
    x, y as NonNegativeReal
    open_1, open_2 as Boolean
`.trim();
</script>

<svelte:head>
    <title>
        Rooc docs - Logic models
    </title>
    <meta name="description"
          content="Write boolean logic directly in ROOC models: logic operators, logic constraints, logic blocks, and how to mix logic with linear constraints."/>
</svelte:head>

<Page cropped padding="1rem" mobilePadding="1rem" gap="1rem">
    <Column gap="0.8rem">
        <h1>
            Logic models
        </h1>
        <p>
            ROOC lets you write boolean logic directly in a model. Formulas over binary variables are compiled to
            linear constraints automatically, so you can state properties like "at least one of these", "if this then
            that" or "exactly one of the two" without encoding them by hand. A model can be entirely made of logic, or
            mix logic with ordinary linear constraints.
        </p>
        <Separator/>
        <h2 id="logic_operators">
            The operators
        </h2>
        <p>
            Every operator also has a symbolic alias, use whichever you prefer:
        </p>
        <ul>
            <li><code>and</code> <code>&&</code>: true when both operands are true</li>
            <li><code>or</code> <code>||</code>: true when at least one operand is true</li>
            <li><code>not</code> <code>!</code>: negation</li>
            <li><code>xor</code>: true when exactly one operand is true</li>
            <li><code>implies</code> <code>{'->'}</code>: false only when the left side is true and the right side is false
            </li>
            <li><code>iff</code> <code>{'<->'}</code>: true when both sides have the same value</li>
        </ul>
        <p>
            The literals <code>true</code> and <code>false</code> are also available. Operands must be binary: Boolean
            variables, or expressions whose value is 0 or 1. Use parentheses to group formulas.
        </p>
        <Separator/>
        <h2 id="logic_constraints">
            Logic constraints
        </h2>
        <p>
            A constraint that consists of a logic formula alone is an assertion: the formula must be true in every
            solution. There is no need to write a comparison, although the explicit form
            <code>... = true</code> is accepted and means the same thing. The <code>for</code> iteration works exactly
            like in ordinary constraints. This is a full model for the vertex cover problem:
        </p>
        <Card padding="0.8rem 1rem" style="overflow-x: auto;">
            <SyntaxHighlighter language="rooc" source={vertexCover}/>
        </Card>
        <p>
            Assertions compile to compact rows whenever possible: each <code>x_u or x_v</code> above becomes the single
            linear constraint <code>x_u + x_v >= 1</code>, without any auxiliary variable.
        </p>
        <Separator/>
        <h2 id="logic_blocks">
            Logic blocks
        </h2>
        <p>
            Like <code>sum</code> and <code>avg</code>, logic has its own expansion blocks: <code>all {'{ }'}</code>
            is the conjunction of a list of formulas, <code>any {'{ }'}</code> is the disjunction, and
            <code>xor {'{ }'}</code> chains exclusive disjunctions. They also exist in the scoped form, for example
            <code>{'any(v in nodes(G)) { x_v }'}</code> is the disjunction over all nodes. Combined with assertions, a
            set of logic requirements reads almost like a specification:
        </p>
        <Card padding="0.8rem 1rem" style="overflow-x: auto;">
            <SyntaxHighlighter language="rooc" source={allBlock}/>
        </Card>
        <p>
            Here <code>solve</code> is used instead of an objective: the solver only needs to find any assignment that
            satisfies the formulas, like a small SAT problem.
        </p>
        <Separator/>
        <h2 id="logic_values">
            Logic as values
        </h2>
        <p>
            A logic formula can also be used inside arithmetic, where it counts as 1 when true and 0 when false. This
            makes counting satisfied conditions natural. The following model maximizes the number of satisfied
            clauses, a MAX-SAT instance:
        </p>
        <Card padding="0.8rem 1rem" style="overflow-x: auto;">
            <SyntaxHighlighter language="rooc" source={maxSat}/>
        </Card>
        <p>
            When a formula is used as a value, the compiler introduces one auxiliary binary variable for it, tied to
            the operands by linear constraints, so the compiled model is still an ordinary mixed integer linear model.
        </p>
        <Separator/>
        <h2 id="mixed_models">
            Mixing logic and linear constraints
        </h2>
        <p>
            Logic constraints and linear constraints can share the same variables. A common pattern is to use Boolean
            variables both in formulas, to express decisions, and in linear constraints, to gate quantities. Booleans
            used in arithmetic count as 0 or 1 just like formulas do:
        </p>
        <Card padding="0.8rem 1rem" style="overflow-x: auto;">
            <SyntaxHighlighter language="rooc" source={mixed}/>
        </Card>
        <p>
            Here <code>x {'<='} 10 * open_1</code> forces production to zero when the facility is closed, while
            <code>open_1 xor open_2</code> is a purely logical requirement on the same decision variables. The best
            solution opens one facility and produces 10 units there.
        </p>
        <p>
            For complete models using logic, including a dominating set problem written both ways, see the
            <a href="/docs/rooc/examples">examples page</a>.
        </p>
    </Column>
</Page>

<style>
    @import "../common.scss";

    p {
        max-width: 70ch;
        line-height: 1.5rem;
        font-size: 1.1rem;
    }

    ul {
        margin-left: 1rem;
        padding-left: 2rem;
        line-height: 1.6rem;
    }
</style>
