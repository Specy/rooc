<script lang="ts">
	import Editor from '$cmp/editor/Editor.svelte';
	import Button from '$cmp/inputs/Button.svelte';
	import Page from '$cmp/layout/Page.svelte';
	import { Monaco } from '$src/lib/Monaco';
	import { onMount } from 'svelte';
	import { projectStore, type Project } from '$src/stores/projectStore';
	import { page } from '$app/stores';
	import { createCompilerStore } from './projectStore';
	import Floppy from '~icons/fa/save';
	import Row from '$cmp/layout/Row.svelte';
	import ButtonLink from '$cmp/inputs/ButtonLink.svelte';
	import { toast } from '$src/stores/toastStore';
	let project: Project;
	let store: ReturnType<typeof createCompilerStore>;
	onMount(() => {
		Monaco.load();
		loadProject();
		return () => {
			Monaco.dispose();
		};
	});

	async function loadProject() {
		project = await projectStore.getProject($page.params.projectId);
		if (!project) {
			toast.error('Project not found', 10000);
			return;
		}
		store = createCompilerStore(project);
	}
	async function save() {
		await store?.save();
		toast.logPill('Saved');
	}
	function compile() {
		store?.compile();
	}
</script>

<Page style="height: 100vh">
	<Row justify="between" padding="0.5rem" gap="0.5rem">
		<ButtonLink href="/projects">Projects</ButtonLink>
		<Button on:click={save} hasIcon>
			<Floppy />
		</Button>
	</Row>
	<div class="wrapper">
		{#if store}
			<Editor
				style="flex: 1; height: 100%;"
				language="rooc"
				bind:code={$store.source}
				highlightedLine={-1}
			/>
			<Editor
				style="flex: 1; height: 100%;"
				language="rooc"
				code={$store.compilationError ?? $store.compiled ?? ''}
				config={{
					readOnly: true,
					lineNumbers: 'off'
				}}
				disabled
				highlightedLine={-1}
			/>
		{/if}
	</div>
	<Row justify="end" gap="0.5rem" padding="0.5rem">
		<Button on:click={compile} border="secondary" color="primary">Compile</Button>
	</Row>
</Page>

<style>
	.wrapper {
		display: flex;
		gap: 1rem;
		flex: 1;
		padding: 0 0.5rem;
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
