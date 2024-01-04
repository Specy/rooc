<script lang="ts">
	import Editor from '$cmp/editor/Editor.svelte';
	import Button from '$cmp/inputs/Button.svelte';
	import Page from '$cmp/layout/Page.svelte';
	import { RoocParser } from '@specy/rooc';
	import type { CompilationError } from '@specy/rooc/';
	let source = '';
	let compiled = '';

	function compile() {
		const parser = new RoocParser(source);
		const transform = parser.compile();
		if (transform.ok) {
			compiled = transform.val.stringify();
		} else {
			compiled = (transform.val as CompilationError).message();
		}
	}
</script>

<Page padding="1rem" gap="1rem">
	<div class="row wrapper">
		<Editor
			style="flex: 1; height: 100%;"
			language="rooc"
			bind:code={source}
			highlightedLine={-1}
		/>
		<textarea value={compiled} />
	</div>
	<Button on:click={compile}>compile</Button>
</Page>

<style>
	.wrapper {
		gap: 1rem;
		flex: 1;
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
