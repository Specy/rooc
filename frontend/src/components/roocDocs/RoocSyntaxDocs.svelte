<script lang="ts">
    import SyntaxHighlighter from '$cmp/SyntaxHighlighter.svelte';
    import Card from '$cmp/layout/Card.svelte';
    import Column from '$cmp/layout/Column.svelte';
    import Separator from "$cmp/misc/Separator.svelte";

    const exampleModel = `
min sum(i in A) { x_i }
s.t.
    /*
        here you can put your constraints
    */
    avg(i in A) { x_i } <= 2
    x_i >= minimum for i in A
where
    // here your constants.
    let A = [1, 2, 3]
    let minimum = 1
define
    // and here the domain of the variables
    x_i as Real for i in A
`.trim();
</script>

<Column gap="0.8rem">
    <h1>
        Table of contents
    </h1>
    <ul class="toc">
        <li><a href="#rooc_objective_function">Objective function</a></li>
        <li><a href="#rooc_constraints">Constraints</a></li>
        <li><a href="#rooc_variable">Variables and compound variables</a></li>
        <li><a href="#rooc_data">Data</a></li>
        <li>
            Blocks
            <ul class="toc">
                <li><a href="#rooc_expansion_blocks">Expansion blocks</a></li>
                <li><a href="#rooc_scoped_blocks">Scoped expansion blocks</a></li>
            </ul>
        </li>
        <li><a href="#rooc_domains">Domains</a></li>
        <li><a href="#rooc_functions_and_tuples">Functions and tuples</a></li>
        <li><a href="#rooc_others">Other things</a></li>
    </ul>
    <Separator/>
    <h1 id="rooc_objective_function">
        Objective function
    </h1>
    <p>

        The objective function can be one of either "min" or "max", after the keyword, you can define whichever
        expression
        you wish to optimize.
        <br/>
        Once the model is compiled, it will be ran through the solver and find the optimal
        solution
        which fits the constraints.
        <br/>
        Some solvers allow you to also work on finding a satisfiable solution, which is a solution that fits the
        constraints, not caring about the objective function. in that case, instead of writing the min/max keyword and
        objective function,
        you can use the "solve" keyword.
    </p>

    <Card padding="0.8rem 1rem" style="overflow-x: auto;">
        <SyntaxHighlighter language="rooc" source={`min x + y / 2`} />
    </Card>
    <Separator/>
    <h1 id="rooc_constraints">
        Constraints
    </h1>
    <p>
        The formal model can follow a list of constraints, you can use one of {`<=, >=, =, <, >`} comparisons.
        Some solvers like the simplex, do not allow for strict inequalities {`<, >`}.
        <br/>
        The special keyword "for" can be used at the end of a constraint to create a constraint for each element that
        you iterate over.
        <br/>
        You can also give a name to constraints, so that you can more easily recognise them in the output. 
        To do that, just write the name of the constraint followed by a colon ":". (eg. "myConstraint: x + y = 2"),
        you can also treat the constraint name as a compound variable and use the iteration syntax.
    </p>
    <Card padding="0.8rem 1rem" style="overflow-x: auto;">
        <SyntaxHighlighter language="rooc" source={`something: y >= x + 2\nconst_i: x * i <= i for i in 0..5`}/>
    </Card>
    <Separator/>
    <h1 id="rooc_variable">
        Variables and compound variables
    </h1>
    <p>The language employs two execution environments: a formal model and a compiled model. Within the formal model,
        you can define three types of variables:</p>

    <ul>
        <li><b>Compound variables:</b> These variables have names determined <em>after</em> compilation.</li>
        <li><b>Constant variables:</b> These variables have their values replaced during compilation (e.g., substituting
            a symbolic name with a concrete number).
        </li>
        <li><b>Standard variables:</b> These variables retain their names and values after compilation.</li>
    </ul>

    <h3>Compound Variables</h3>

    <p>A compound variable is identified by an underscore (<code>_</code>) in its name. The portion of the name
        preceding the first underscore serves as a prefix. The remaining parts (split by underscores) are treated as an
        expression that must evaluate to a string, number, or node.</p>

    <p>For example, in the compound variable <code>x_i</code>, <code>x</code> is the prefix, and <code>i</code> is a
        variable whose value will be used during compilation to construct the final variable name. If <code>i</code> has
        the value <code>3</code>, the compiled variable name would be <code>x_3</code>.</p>

    <p>Compound variables are particularly useful when iterating over a list and dynamically generating variable names
        based on each element's value. See the example below for further clarification.</p>

    <p>You can use curly braces <code>{"{}"}</code> to enclose more complex expressions within the compound variable
        name.
        If the expression is a simple number or a single variable name, the curly braces can be omitted. For instance,
        <code>{"data_{i + 1}"}</code> allows for more complex index calculations, while <code>item_1</code> or <code>value_count</code>
        are simpler examples.</p>

    <h3>Escaping Compound Variable Names</h3>

    <p>If you need to use a variable name that <em>looks</em> like a compound variable but should be treated literally,
        you can escape the name using a backslash (<code>\</code>). For example, <code>\x_hello</code> will be
        interpreted as the literal variable name <code>x_hello</code>, preventing the evaluation of <code>hello</code>.
    </p>

    <Card padding="0.8rem 1rem" style="overflow-x: auto;">
        <SyntaxHighlighter language="rooc" source={`min 1\ns.t.\n    x_i + \\x_hello <= 1\n    x_{i + 1}_i <= 2\nwhere\n    let i = 3`}/>
    </Card>
    will be compiled to
    <Card padding="0.8rem 1rem" style="overflow-x: auto;">
        <SyntaxHighlighter language="rooc" source={`min 1\ns.t.\n    x_3 + x_hello <= 1\n    x_4_3 <= 2`} />
    </Card>
    <Separator/>
    <h1 id="rooc_data">
        Data
    </h1>
    <p>Following the constraint definitions, you can define data within the <code>where</code> section. This data is
        then available for use throughout your model.</p>

    <p>The ROOC language supports various data types, including arrays, matrices, graphs, strings, numbers, and boolean
        values. Furthermore, you can use expressions, function calls, and other computational constructs within the
        <code>where</code> section.</p>

    <p>To define a named constant, use the <code>let</code> keyword followed by the constant's name and its value. For
        example:</p>

    <Card padding="0.8rem 1rem" style="overflow-x: auto;">
        <SyntaxHighlighter language="rooc"
                           source={`let A = [1, 2, 3]\nlet B = [\n    [1, 2, 3],\n    [4, 5, 6]\n]\nlet G = Graph {\n    A -> [ C, B:2 ],\n    B -> [ A, C:-3 ],\n    C\n}\nlet lengthOfA = len(A)\nlet someString = "hello"\nlet someBool = true`}/>
    </Card>
    <Separator/>
    <h1 id="rooc_expansion_blocks">
        Expansion blocks
    </h1>
    <p>
        Expansion blocks are a special type of expression macro, used to preprocess other expressions. A common example
        is the
        <code>avg</code> block, which takes a comma-separated list of expressions and expands it calculate the
        arithmetic
        average of those expressions.
    </p>


    <Card padding="0.8rem 1rem"  style="overflow-x: auto;">
        <SyntaxHighlighter language="rooc" source={`avg{ x, y, z }`} />
    </Card>
    <p>
        This will be compiled to:
    </p>
    <Card padding="0.8rem 1rem"  style="overflow-x: auto;">
        <SyntaxHighlighter language="rooc" source={`(x + y + z) / 3`}/>
    </Card>
    <p>

        There are different kinds of expansion blocks, you can find them in the documentation.
    </p>

    <Separator/>
    <h1 id="rooc_scoped_blocks">
        Scoped expansion blocks
    </h1>
    <p>
        There are also special kinds of expansion blocks, which have a <em>scope</em> attached to it.
        <br/>
        In the normal expansion blocks, you need to manually specify the different expressions, separating them with a
        comma.
        The Scoped expansion blocks, you specify a template (which is the expression inside of the block), and an
        iteration scope (eg. iterating over a list).
        <br/>
        Together with compound variables and scoped expansion blocks, you can do things like creating a summation over a
        list or range.
        <br/>
        As an example, here is creating a summation of <el>x_u</el>, where u is a number from 0 to 3 (3 excluded)
    </p>

    <Card padding="0.8rem 1rem"  style="overflow-x: auto;">
        <SyntaxHighlighter language="rooc" source={`sum(u in 0..3) { x_u }`}/>
    </Card>
    <p>
        will be compiled to:
    </p>

    <Card padding="0.8rem 1rem" style="overflow-x: auto;">
        <SyntaxHighlighter language="rooc" source={`x_0 + x_1 + x_2`}/>
    </Card>
    <p>
        there are different scoped expansion blocks that can be used to expand the expressions, you can find them in the
        documentation.
    </p>

    <Separator/>
    <h1 id="rooc_domains">
        Domains
    </h1>
    <p>
        After the data you can define in which domain each variable will be part of, those variables are the ones that
        will remain after the compilation is finished. The domain knowledge will then be used by solvers.
        <br/>
        <br/>
        Every variable that will end up in the compiled model must be defined, you can use the "for" iteration like in
        the constraints to define compound variables.
        <br/>
        The domains are "Real", "NonNegativeReal", "Boolean" and "IntegerRange".
        <br/>
        You can define a minimum and maximum value for each domain except for the "Boolean" domain.
        They are required for the "IntegerRange" domain, and optional for Real (which defaults to -inf and inf) and
        NonNegativeReal (which defaults to 0 and inf).
    </p>

    <Card padding="0.8rem 1rem" style="overflow-x: auto;">
        <SyntaxHighlighter language="rooc" source={`y, x_u as IntegerRange(0,20) for u in 0..5`}/>
    </Card>
    <h1 id="rooc_functions_and_tuples">
        Functions and tuples
    </h1>
    <p>

        The ROOC langage has a set of builtin functions that can be used to manipulate data.
        <br/>
        Those functions can be run anywhere in the model or data section.
        <br/>
        <br/>
        The language also has support for tuples and tuples destructuring, you can destructure a tuple or array by
        writing the name of the variables inside a parenthesis "(a,b,c)". Some builtin values are destructurable, like
        arrays, tuples and graph edges.
    </p>

    <Card padding="0.8rem 1rem" style="overflow-x: auto;">
        <SyntaxHighlighter language="rooc" source={`sum((el, idx) in enumerate([10, 20,30])) { x_idx * el}`}/>
    </Card>
    <p>
        will be compiled to
    </p>
    <Card padding="0.8rem 1rem" style="overflow-x: auto;">
        <SyntaxHighlighter language="rooc" source={`x_0 * 10 + x_1 * 20 + x_2 * 30`} />
    </Card>
    <Separator/>
    <h1 id="rooc_others">
        Other things
    </h1>
    <p>

        You can write comments in the model by using the "//" or "/* */" syntax, a model is structured (in this order)
        by
        the objective function, constraints, data and domains.
    </p>

    <Card padding="0.8rem 1rem" style="overflow-x: auto;">
        <SyntaxHighlighter language="rooc" source={exampleModel} />
    </Card>
</Column>

<style>
    .toc {
        padding-left: 2rem;
    }

    p {
        max-width: 70ch;
        line-height: 1.5rem;
        font-size: 1.1rem;
    }

    ul{
        margin-left: 1rem;
    }
</style>

