<script lang="ts">
	import Editor from '$cmp/editor/Editor.svelte';
	import Button from '$cmp/inputs/Button.svelte';
	import Page from '$cmp/layout/Page.svelte';
	import { Monaco } from '$src/lib/Monaco';
	import { onMount } from 'svelte';
	import { projectStore, type Project, createProject } from '$src/stores/projectStore';
	import { page } from '$app/stores';
	import Floppy from '~icons/fa/floppy-o';
	import Book from '~icons/fa/book';
	import Share from '~icons/fa/share-alt';
	import Row from '$cmp/layout/Row.svelte';
	import ButtonLink from '$cmp/inputs/ButtonLink.svelte';
	import { toast } from '$src/stores/toastStore';
	import FloatingContainer from '$cmp/misc/FloatingContainer.svelte';
	import RoocDocs from '$cmp/roocDocs/RoocFunctionsDocs.svelte';
	import Column from '$cmp/layout/Column.svelte';
	import RoocSyntaxDocs from '$cmp/roocDocs/RoocSyntaxDocs.svelte';
	import ProjectEditor from '$cmp/projects/ProjectEditor.svelte';
	import lzstring from 'lz-string';
	import { prompter } from '$src/stores/promptStore';
	let showDocs = false;
	let project: Project | undefined;
	onMount(() => {
		Monaco.load();
		loadProject();
		return () => {
			Monaco.dispose();
		};
	});

	async function loadProject() {
		const id = $page.params.projectId;
		if (id === 'share') {
			const code = $page.url.searchParams.get('project');
			const parsed = JSON.parse(lzstring.decompressFromEncodedURIComponent(code));
			parsed.id = 'share';
			project = parsed as Project;
			return
		}
		project = await projectStore.getProject($page.params.projectId);
		if (!project) {
			toast.error('Project not found', 10000);
			return;
		}
	}
	async function save() {
		if (!project) return;
		try {
			if (project.id === 'share') {
				if (!(await prompter.confirm('Do you want to save this shared project in your projets?')))
					return;
				delete project.id;
				const newProject = await projectStore.createNewProject(project.name, project.description);
				project.id = newProject.id;
			}
			await projectStore.updateProject(project.id, project);
			toast.logPill('Saved');
		} catch (e) {
			toast.error("Couldn't save project");
			console.error(e);
		}
	}
	function share(){
		if (!project) return;
		const p = {...project};
		p.id = 'share';
		const code = lzstring.compressToEncodedURIComponent(JSON.stringify(p));
		const url = `${window.location.origin}/projects/share?project=${code}`;
		navigator.clipboard.writeText(url);
		toast.logPill('Copied to clipboard');
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
			<Button hasIcon on:click={share}>
				<Share />
			</Button>
			<Button hasIcon on:click={() => (showDocs = !showDocs)}>
				<Book />
			</Button>
			<Button on:click={save} hasIcon>
				<Floppy />
			</Button>
		</Row>
	</Row>
	{#if project}
		<ProjectEditor bind:project />
	{:else}
		<div class="col justify-center align-center flex-1">
			<h1>Loading...</h1>
		</div>
	{/if}
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
</Page>

