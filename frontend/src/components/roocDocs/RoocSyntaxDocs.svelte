<script lang="ts">
    import SyntaxHighlighter from '$cmp/SyntaxHighlighter.svelte';
    import Card from '$cmp/layout/Card.svelte';
    import Column from '$cmp/layout/Column.svelte';
    import {roocExamples} from "$cmp/roocDocs/examples";
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

<h1>The language</h1>
<Column gap="0.8rem">
    With ROOC you can formalize mathematical models and let the compiler create the final model.
    <br/>
    <h1>
        Table of contents
    </h1>
    <ul class="toc">
        <li><a href="#rooc_objective_function">Objective function</a></li>
        <li><a href="#rooc_constraints">Constraints</a></li>
        <li><a href="#rooc_variable">Variables and compound variables</a></li>
        <li>
            Blocks
            <ul class="toc">
                <li><a href="#rooc_expansion_blocks">Expansion blocks</a></li>
                <li><a href="#rooc_scoped_blocks">Scoped expansion blocks</a></li>
            </ul>
        </li>
        <li><a href="#rooc_data">Data</a></li>
        <li><a href="#rooc_domains">Domains</a></li>
        <li><a href="#rooc_functions_and_tuples">Functions and tuples</a></li>
        <li><a href="#rooc_others">Other things</a></li>

        <li><a href="#rooc_examples">Examples</a></li>
    </ul>
    <Separator/>
    <h1 id="rooc_objective_function">
        Objective function
    </h1>
    <p>

        The objective function can be one of either "min" or "max", after the keyword, you can define whichever
        expression
        you wish to optimize.
        <br />
        Once the model is compiled, it will be ran through the solver and find the optimal
        solution
        which fits the constraints.
    </p>

    <Card padding="0.8rem 1rem">
        <SyntaxHighlighter language="rooc" source={`min x + y / 2`} style="overflow-x: auto;"/>
    </Card>
    <Separator/>
    <h1 id="rooc_constraints">
        Constraints
    </h1>
    <p>

        The formal model can follow a list of constraints, you can use one of {`<=, >=, =`} relations.
        <br />
        The special keyword "for" can be used at the end of a constraint to create a constraint for each element that
        you iterate over.
    </p>
    <Card padding="0.8rem 1rem">
        <SyntaxHighlighter language="rooc" source={`y >= x + 2\nx * i <= i for i in 0..5`}
                           style="overflow-x: auto;"/>
    </Card>
    <Separator/>
    <h1 id="rooc_variable">
        Variables and compound variables
    </h1>
    <p>
        The language has "two runtimes", the formal model and the compiled model. In the formal model you can define
        variables whose name is
        determined after the model is compiled (compound variables), variables whose value is replaced during
        compilation
        (for example a number), or normal variables which will be kept after the compilation is finished.
        <br />
        <br />
        A compound variable is any variable with an underscore in it's name, the first part of the variable will be used
        as
        a prefix name, and the rest of the variable (split by the underscore) will be treated as an expression, and
        evaluated as a string or number
        <br />
        You can omit the curly braces if the index is a number or a variable name, for other kind of expressions, they
        are
        needed.
        <br />
        <br />
        Sometimes you might need to manually write a name of a variable which looks like a compound variable, in that
        case
        you can escape the name with a backslash. Example: "\x_hello"
    </p>
    <Card padding="0.8rem 1rem">
        <SyntaxHighlighter language="rooc"
                           source={`min 1\ns.t.\n    x_i + \\x_hello <= 1\n    x_{i + 1}_i <= 2\nwhere\n    let i = 3`}
                           style="overflow-x: auto;"/>
    </Card>
    will be compiled to
    <Card padding="0.8rem 1rem">
        <SyntaxHighlighter language="rooc" source={`min 1\ns.t.\n    x_3 + x_hello <= 1\n    x_4_3 <= 2`}
                           style="overflow-x: auto;"/>
    </Card>
    <Separator/>
    <h1 id="rooc_expansion_blocks">
        Expansion blocks
    </h1>
    <p>
        Expansion blocks are a special kind of "expression" that can be used to preprocess an expression, an example is
        the average block
        which, given a comma separated list of expressions, it will expand it to the average of all the expressions.
    </p>

    <Card padding="0.8rem 1rem">
        <SyntaxHighlighter language="rooc" source={`avg{ x, y, z }`} style="overflow-x: auto;"/>
    </Card>
    <p>
        will be compiled to
    </p>
    <Card padding="0.8rem 1rem">
        <SyntaxHighlighter language="rooc" source={`(x + y + z) / 3`} style="overflow-x: auto;"/>
    </Card>
    <p>

        There are different expansion blocks that can be used to expand the expressions, you can find them in the
        documentation.
    </p>

    <Separator/>
    <h1 id="rooc_scoped_blocks">
        Scoped expansion blocks
    </h1>
    <p>

        Another type of expansion blocks are the ones that have a scope connected to it, inside the scope, the action of
        the
        expansion block
        will be applied to all the iterations that the scope has.
        <br />
        You can put more than one scope and it will behave like if they were nested inside each other.
    </p>

    <Card padding="0.8rem 1rem">
        <SyntaxHighlighter language="rooc" source={`sum(u in 0..3) { x_u }`} style="overflow-x: auto;"/>
    </Card>
    <p>

        will be compiled to
    </p>

    <Card padding="0.8rem 1rem">
        <SyntaxHighlighter language="rooc" source={`x_0 + x_1 + x_2 + x_3`} style="overflow-x: auto;"/>
    </Card>
    <p>

        there are different scoped expansion blocks that can be used to expand the expressions, you can find them in the
        documentation.
    </p>

    <Separator/>
    <h1 id="rooc_data">
        Data
    </h1>
    <p>

        After the constraints, inside the "where" section you can define the data of the model, this data will be used inside the constraints and
        objective functions.
        <br />
        <br />
        The ROOC language supports arrays, matrices, graphs, strings, numbers and boolean values.
        <br />
        To define a variable, you can use the "let" keyword followed by the name of the variable and the value.
    </p>

    <Card padding="0.8rem 1rem">
        <SyntaxHighlighter language="rooc"
                           source={`let A = [1, 2, 3]\nlet B = [\n    [1, 2, 3],\n    [4, 5, 6]\n]\nlet G = Graph {\n    A -> [ C, B:2 ],\n    B -> [ A, C:-3 ],\n    C\n}\nlet lengthOfA = len(A)\nlet someString = "hello"\nlet someBool = true`}
                           style="overflow-x: auto;"/>
    </Card>
    <Separator/>
    <h1 id="rooc_domains">
        Domains
    </h1>
    <p>

        After the data you can define in which domain each variable will be part of, those variables are the ones that
        will
        remain after the compilation is finished.
        <br />
        <br />
        Every variable that will end up in the compiled model must be defined, you can use the "for" iteration like in
        the
        constraints to define compound variables.
        <br />
        The domains are "Real", "PositiveReal", "Integer", "Boolean"
    </p>

    <Card padding="0.8rem 1rem">
        <SyntaxHighlighter language="rooc" source={`y, x_u as Integer for u in 0..5`} style="overflow-x: auto;"/>
    </Card>
    <h1 id="rooc_functions_and_tuples">
        Functions and tuples
    </h1>
    <p>

        The ROOC langage has a set of builtin functions that can be used to manipulate data, in the future, the language
        will support custom user defined functions.
        <br />
        Those functions can be run anywhere in the model or data section.
        <br />
        <br />
        The language also has support for tuples and tuples destructuring, you can destructure a tuple or array by
        writing the name of the variables inside a parenthesis "(a,b,c)". Some builtin values are destructurable, like arrays, tuples and graph edges.
    </p>

    <Card padding="0.8rem 1rem">
        <SyntaxHighlighter language="rooc" source={`sum((el, idx) in enumerate([10, 20,30])) { x_idx * el}`}
                           style="overflow-x: auto;"/>
    </Card>
    <p>
    will be compiled to
    </p>
    <Card padding="0.8rem 1rem">
        <SyntaxHighlighter language="rooc" source={`x_0 * 10 + x_1 * 20 + x_2 * 30`} style="overflow-x: auto;"/>
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

    <Card padding="0.8rem 1rem">
        <SyntaxHighlighter language="rooc" source={exampleModel} style="overflow-x: auto;"/>
    </Card>
    <Separator/>
    <h1 id="rooc_examples">
        Examples
    </h1>
    {#each roocExamples as code}
        <Card padding="0.8rem 1rem">
            <SyntaxHighlighter language="rooc" source={code} style="overflow-x: auto;"/>
        </Card>
    {/each}
</Column>

<style>
    .toc {
        padding-left: 2rem;
    }
    p{
        max-width: 70ch;
        line-height: 1.5rem;
        font-size: 1.1rem;
    }
</style>

