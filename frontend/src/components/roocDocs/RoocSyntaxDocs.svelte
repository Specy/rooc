<script lang="ts">
	import SyntaxHighlighter from "$cmp/SyntaxHighlighter.svelte";
import Card from "$cmp/layout/Card.svelte";
import Column from "$cmp/layout/Column.svelte";

const exampleModel = `
min sum(u in nodes(G)) { x_u }
s.t.
    x_v + sum((_, _, u) in neigh_edges(v)) { x_u } >= 1 for v in nodes(G)
    avg(el in A) { el * lengthOfA } <= x_someString * len(B)
where
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
`.trim()
</script>


<h1>
    The language
</h1>
<Column gap='0.4rem'>
    With ROOC you can formalize mathematical models and let the compiler do the rest.
    <br />
    You can define formal expressions using iterators over data, using different functions to transform it.
    <br />
    A general model can be defined as an objective function, a set of constraints and a set of data (optional).
    <br />
    There are different expansion functions that can be used to expand the expressions
    <Card padding='0.8rem 1rem'>
        <SyntaxHighlighter language='rooc' source={exampleModel} />
    </Card>
</Column>