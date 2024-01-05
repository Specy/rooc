<script lang="ts">
	import Editor from '$cmp/editor/Editor.svelte';
	import Button from '$cmp/inputs/Button.svelte';
	import Page from '$cmp/layout/Page.svelte';
	import { RoocParser } from '@specy/rooc';
	import type { CompilationError, TransformError } from '@specy/rooc/';
	let source = '';
	let compiled = '';

	function compile() {
		const parser = new RoocParser(source);
		const compile = parser.compile();
		if (!compile.ok) {
			return (compiled = (compile.val as CompilationError).message());
		}
		const transform = compile.val.transform();
		if (!transform.ok) {
			console.log(transform.val);
			return (compiled = (transform.val as TransformError).message());
		}
		compiled = transform.val.stringify();
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
