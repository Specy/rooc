<script lang="ts">
	import SyntaxHighlighter from '$cmp/SyntaxHighlighter.svelte';
	import Card from '$cmp/layout/Card.svelte';
	import Column from '$cmp/layout/Column.svelte';

	const exampleModel = `
min sum(u in nodes(G)) { x_u }
s.t.
    // here you can put your constraints
    x_v + sum((_, _, u) in neigh_edges(v)) { x_u } >= 1 for v in nodes(G)
    avg(el in A) { el * lengthOfA } <= x_someString * len(B)
where
    /*
        here your constants. 
    */
    G = Graph {
        A -> [ C, B:2 ],
        B -> [ A, C:-3 ],
        C
    }
    A = [1.0, 2.0, 3.0]
    lengthOfA = len(A)
    B = [
        [1, 2, 3],
        [4, 5, 6]
    ]
    someString = "hello"
define
    // and here the domain of the variables
    x_u, x_v as Boolean for v in nodes(G), (_,_,u) in neigh_edges(v)
    \\x_hello as Real
`.trim();
const exampleModel2 = `
min 1
s.t.
    x_u + x_{u + 1} + x_2 + x_{len(A)} >= 1
where 
    u = 1
    A = [1, 2, 3]
define
    x_u, x_{u + 1}, x_2, x_{len(A)} as PositiveReal 
`.trim();
</script>

<h1>The language</h1>
<Column gap="0.8rem">
	With ROOC you can formalize mathematical models and let the compiler do the rest.
	<br />
	You can define formal expressions using iterators over data, using different functions to transform
	it.
	<br />
	A general model can be defined as an objective function, a set of constraints,a set of data (optional), and the domains where the variables lay in (Real, PositiveReal, Boolean, Integer etc..).
	<br />
	There are different expansion functions that can be used to expand the expressions
	<Card padding="0.8rem 1rem">
		<SyntaxHighlighter language="rooc" source={exampleModel} style="overflow-x: auto;" />
	</Card>
	<h2>
		Compound variables
	</h2>
	Compound variables are variables whose name is determined at compilation time. They have a name, and a list of indices. 
	Those indices can be anything that can be written out, so a number or something that can be written out as a string, like strings, graph nodes, true/false.
	You can omit the curly braces if the index is a number or a variable name, for expressions they need to be there.

	If you want to manually write a variable which "looks like" a compound variable, but in reality is a normal one, you can 
	escape the name with a backslash. Example: "\x_hello"
	<Card padding="0.8rem 1rem">
		<SyntaxHighlighter language="rooc" source={exampleModel2} style="overflow-x: auto;" />
	</Card>

</Column>

