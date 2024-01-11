<script lang="ts">
	import SyntaxHighlighter from '$cmp/SyntaxHighlighter.svelte';
	import Card from '$cmp/layout/Card.svelte';
	import Column from '$cmp/layout/Column.svelte';
	import { createRoocFunctionSignature } from '$src/lib/Rooc/RoocUtils';
	import {
		ROOC_RUNTIME_FUNCTIONS,
		RUNTIME_BLOCK_SCOPED_FUNCTIONS,
		RUNTIME_BLOCK_FUNCTIONS
	} from '@specy/rooc/dist/runtime';

	let functions = ROOC_RUNTIME_FUNCTIONS.values();
	let blockFunctions = RUNTIME_BLOCK_FUNCTIONS.values();
	let blockScopedFunctions = RUNTIME_BLOCK_SCOPED_FUNCTIONS.values();
</script>

<h2>Functions</h2>
<Column gap="0.5rem">
	<Column style="margin-bottom: 1rem;" gap='0.4rem'>
		They are functions that accept parameters and return a value, you can use them inside blocks,
		assignments or in expressions
        <br />
        <Card padding='0.8rem'>
            <SyntaxHighlighter language="rooc" source={"x * len(A) <= 2"} />
        </Card>
	</Column>
	{#each functions as fun}
		<Card padding="0.8rem" gap="0.5rem">
			<div style="overflow-x: auto">
				<SyntaxHighlighter language="typescript" source={createRoocFunctionSignature(fun)} />
			</div>
			{fun.description}
		</Card>
	{/each}
</Column>
<h2>Block functions</h2>
<Column gap="0.5rem">
	<Column>
		They are blocks which have one or more expressions separated by a comma, they will use those
		expressions to perform a transformation, like the avg (average) block
	</Column>
	{#each blockFunctions as fun}
		<Card padding="0.8rem" gap="0.5rem">
			<div style="overflow-x: auto">
				<SyntaxHighlighter language="typescript" source={createRoocFunctionSignature(fun)} />
			</div>
			{fun.description}
		</Card>
	{/each}
</Column>
<h2>Block scoped functions</h2>
<Column gap="0.5rem">
	<Column gap='0.4rem' style="margin-bottom: 1rem;">
		They are function blocks, it has as parameters one or more iterators over iterable data, they
		will declare a variable (or more uring tuples destructuring) for each iterator and then execute
		the block.
		<br />
		If there are more than one iterators, they will behave as nested iterators, where the first iterator
		is the outermost one
		<br />
		<Card padding="0.8rem">
			<SyntaxHighlighter language="rooc" source={'sum(i in 0..len(A), el in A[i]) { x_i * el }'} />
		</Card>
	</Column>
	{#each blockScopedFunctions as fun}
		<Card padding="0.8rem" gap="0.5rem">
			<div style="overflow-x: auto">
				<SyntaxHighlighter language="typescript" source={createRoocFunctionSignature(fun)} />
			</div>
			{fun.description}
		</Card>
	{/each}
</Column>
