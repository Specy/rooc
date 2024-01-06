<script lang="ts">
	import Editor from '$cmp/editor/Editor.svelte';
	import Button from '$cmp/inputs/Button.svelte';
	import Page from '$cmp/layout/Page.svelte';
	import { RoocParser } from '@specy/rooc';
	import type { CompilationError, TransformError } from '@specy/rooc';
	import { Monaco } from '$src/lib/Monaco';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	let source = ``;
	let compiled = '';
	let lastCompilation = '';

	onMount(() => {
		Monaco.load();
		const saved = localStorage.getItem('rooc_source');
		source =
			saved ||
			`
		min sum(u in nodes(G)) { x_u }
s.t. 
    x_v + sum((_, _, u) in neigh_edges(v)) { x_u } >= 1    for v in nodes(G)
where
    G = Graph {
		A -> [B, C, D],
		B -> [A, C],
		C -> [A, B, D],
		D -> [A]
    }
		`.trim();
		return () => {
			Monaco.dispose();
		};
	});

	function compile() {
		lastCompilation = '';
		const parser = new RoocParser(source);
		const compile = parser.compile();
		if (!compile.ok) {
			return (compiled = (compile.val as CompilationError).message());
		}
		const transform = compile.val.transform();
		if (!transform.ok) {
			return (compiled = (transform.val as TransformError).message());
		}
		compiled = transform.val.stringify();
		lastCompilation = compiled;
	}
	function typeCheck() {
		if (browser) {
			localStorage.setItem('rooc_source', source);
		}
		try {
			const parser = new RoocParser(source);
			const compile = parser.compile();
			if (!compile.ok) {
				return (compiled = (compile.val as CompilationError).message());
			}
			const transform = compile.val.typeCheck();
			if (!transform.ok) {
				return (compiled = transform.val.message());
			}
			compiled = lastCompilation;
		} catch (e) {
			console.error(e);
			compiled = `Error:\n	${e}`;
		}
	}
	$: if (source) {
		typeCheck();
	}
</script>

<Page padding="1rem" gap="1rem" style="height: 100vh">
	<div class="wrapper">
		<Editor
			style="flex: 1; height: 100%;"
			language="rooc"
			bind:code={source}
			highlightedLine={-1}
		/>
		<Editor
			style="flex: 1; height: 100%;"
			language="rooc"
			bind:code={compiled}
			config={{
				readOnly: true,
				lineNumbers: 'off'
			}}
			disabled
			highlightedLine={-1}
		/>
	</div>
	<div style="display: flex; justify-content: flex-end">
		<Button on:click={compile}>Compile</Button>
	</div>
</Page>

<style>
	.wrapper {
		display: flex;
		gap: 1rem;
		flex: 1;
	}
	@media (max-width: 768px) {
		.wrapper {
			flex-direction: column;
		}
	}
	textarea {
		background-color: var(--secondary);
		color: var(--secondary-text);
		resize: none;
		font-size: 1.1rem;
		padding: 1rem;
		border-radius: 0.4rem;
		flex: 1;
	}
</style>
