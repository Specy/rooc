<script lang="ts">
	import Editor from '$cmp/editor/Editor.svelte';
	import Button from '$cmp/inputs/Button.svelte';
	import { Monaco } from '$src/lib/Monaco';
	import { onMount } from 'svelte';
	import { type Project } from '$src/stores/projectStore';
	import Row from '$cmp/layout/Row.svelte';
	import FaCopy from '~icons/fa/copy.svelte';
	import { createCompilerStore } from '$src/routes/projects/[projectId]/projectStore';
	import LatexRenderer from '$cmp/LatexRenderer.svelte';
	import Card from '$cmp/layout/Card.svelte';
	import { toast } from '$src/stores/toastStore';
	export let project: Project;
	let store = createCompilerStore(project);
	onMount(() => {
		Monaco.load();
		return () => {
			Monaco.dispose();
		};
	});

	function compile() {
		store?.compile();
	}
	async function copy() {
		await navigator.clipboard.writeText($store.latex);
		toast.logPill('Copied to clipboard');
	}
	$: $store.source = project.content;
</script>

<div class="wrapper">
	<Editor
		style="flex: 1; height: 100%;"
		language="rooc"
		bind:code={project.content}
		highlightedLine={-1}
	/>
	<div class="result-container">
		<Card style="flex: 1; position:relative">
			<LatexRenderer source={$store.latex} style="flex:1; padding: 0.5rem 1rem; overflow:auto;" />
			<Button
				hasIcon
				style="position:absolute; top: 0.5rem; right: 0.5rem; width: 2.4rem; height: 2.4rem; padding: 0"
				on:click={copy}
			>
				<FaCopy />
			</Button>
		</Card>
		<Editor
			style="flex: 1;"
			language="rooc"
			code={$store.compilationError ?? $store.compiled ?? ''}
			config={{
				readOnly: true,
				lineNumbers: 'off'
			}}
			disabled
			highlightedLine={-1}
		/>
	</div>
</div>
<Row justify="end" gap="0.5rem" padding="0.5rem">
	<Button on:click={compile} border="secondary" color="primary">Compile</Button>
</Row>

<style>
	.result-container{
		display: grid;
		grid-template-rows: 50%;
		gap: 0.5rem;
	}
	.wrapper {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 0.5rem;
		flex: 1;
		padding: 0 0.5rem;
	}
	@media (max-width: 768px) {
		.wrapper {
			grid-template-rows: 50%;
			grid-template-columns: 1fr;
		}
	}
</style>
