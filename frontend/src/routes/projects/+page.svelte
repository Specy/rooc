<script lang="ts">
	import ButtonLink from '$cmp/inputs/ButtonLink.svelte';
	import Nav from '$cmp/layout/Nav.svelte';
	import Page from '$cmp/layout/Page.svelte';
	import Row from '$cmp/layout/Row.svelte';
	import ProjectCard from '$cmp/projects/ProjectCard.svelte';
	import { textDownloader } from '$src/lib/utils';
	import { projectStore, type Project } from '$stores/userProjectsStore';
	import { prompter } from '$src/stores/promptStore';
	import { toast } from '$src/stores/toastStore';
	import { onMount } from 'svelte';
	import { scale } from 'svelte/transition';

	onMount(() => {
		if ('launchQueue' in window) {
			// @ts-ignore
			launchQueue.setConsumer(async (launchParams) => {
				for (const file of launchParams.files) {
					try {
						const blob = await file.getFile();
						blob.handle = file;
						const text = await blob.text();
						const project = JSON.parse(text) as Project;
						const p = await projectStore.createNewProject(project.name, project.description);
						await projectStore.updateProject(p.id, project);
					} catch (e) {
						console.error(e);
						toast.error('Failed to import project!');
					}
				}
				toast.logPill(
					// @ts-ignore
					`Imported ${launchQueue.files.length} project${launchQueue.files.length > 1 ? 's' : ''}`
				);
			});
		} else {
			console.error('File Handling API is not supported!');
		}
	});

	async function deleteProject(project: Project) {
		if (!(await prompter.confirm(`Are you sure you want to delete ${project.name}?`))) return;
		try {
			await projectStore.deleteProject(project.id);
			toast.success(`${project.name} deleted`);
		} catch (error) {
			toast.error(error.message);
		}
	}
	function updateProject(project: Project) {
		projectStore.updateProject(project.id, project);
	}
	function onDownload(project: Project) {
		textDownloader(JSON.stringify(project), `${project.name}.rooc`);
	}
</script>

<svelte:head>
	<title>Projects</title>
	<meta name="description" content="Your Rooc projects" />
</svelte:head>

<Nav />
<Page cropped="50rem" padding="2rem" mobilePadding="1rem" gap="3rem">
	<Row justify="between">
		<h1>Projects</h1>
		<ButtonLink href="/projects/new">New Project</ButtonLink>
	</Row>
	<div class="projects-wrapper">
		{#each $projectStore.projects as project, i (project.id)}
			<div
				in:scale|global={{ duration: 200, delay: i * 50 + 150, start: 0.9 }}
				out:scale={{ duration: 300, start: 0.8 }}
			>
				<ProjectCard
					{project}
					on:onDelete={(e) => deleteProject(e.detail.project)}
					on:onUpdate={(e) => updateProject(e.detail.project)}
					on:onDownload={(e) => onDownload(e.detail.project)}
				/>
			</div>
		{/each}
	</div>
	{#if !$projectStore.projects.length}
		<p style="width:100%; text-align:center;">No projects yet, create one!</p>
	{/if}
</Page>

<style lang="scss">
	.projects-wrapper {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
		gap: 1rem;
	}
</style>
