<script lang="ts">
	import Editor from '$cmp/editor/Editor.svelte';
	import Button from '$cmp/inputs/Button.svelte';
	import Page from '$cmp/layout/Page.svelte';
	import { Monaco } from '$src/lib/Monaco';
	import { onMount } from 'svelte';
	import { projectStore, type Project } from '$src/stores/projectStore';
	import { page } from '$app/stores';
	import { createCompilerStore } from './projectStore';
	import Floppy from '~icons/fa/floppy-o';
	import Book from '~icons/fa/book';
	import Row from '$cmp/layout/Row.svelte';
	import ButtonLink from '$cmp/inputs/ButtonLink.svelte';
	import { toast } from '$src/stores/toastStore';
	import FloatingContainer from '$cmp/misc/FloatingContainer.svelte';
	import RoocDocs from '$cmp/roocDocs/RoocFunctionsDocs.svelte';
	import Column from '$cmp/layout/Column.svelte';
	import RoocSyntaxDocs from '$cmp/roocDocs/RoocSyntaxDocs.svelte';
	let showDocs = false;
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

<svelte:head>
	<title>{project?.name ?? 'Project'} - Rooc</title>
	<meta name="description" content="Edit your Rooc project" />
</svelte:head>

<Page style="height: 100vh">
	<Row justify="between" padding="0.5rem" gap="0.5rem">
		<ButtonLink href="/projects">Projects</ButtonLink>
		<Row gap="0.5rem">
			<Button hasIcon on:click={() => (showDocs = !showDocs)}>
				<Book />
			</Button>
			<Button on:click={save} hasIcon>
				<Floppy />
			</Button>
		</Row>
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
	<FloatingContainer bind:visible={showDocs} title="Documentation">
		<Column
			style="width: 45rem; max-width: calc(100vw - 1rem); max-height: 80vh; overflow-y: auto;"
			padding="0.8rem"
			gap="0.5rem"
		>
			<RoocSyntaxDocs />

			<RoocDocs />
		</Column>
	</FloatingContainer>
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
