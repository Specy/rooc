<script lang="ts">
	import Button from '$cmp/inputs/Button.svelte';
	import Input from '$cmp/inputs/Input.svelte';
	import Textarea from '$cmp/inputs/Textarea.svelte';
	import Card from '$cmp/layout/Card.svelte';
	import Page from '$cmp/layout/Page.svelte';
	import { projectStore } from '$src/stores/projectStore';
	import { toast } from '$src/stores/toastStore';
	import { goto } from '$app/navigation';
	let name = '';
	let description = '';

	async function create() {
		try {
			const p = await projectStore.createNewProject(name, description);
			toast.success(`Created project ${p.name}`);
			goto(`/projects/${p.id}`);
		} catch (e) {
			toast.error(`Failed to create project: ${e.message}`);
			console.error(e);
		}
	}
</script>

<Page gap="2rem" cropped padding="2rem" mobilePadding="1rem">
	<h1>Create new project</h1>
	<Card gap="0.5rem" padding="1rem" withShadow>
		<h3>Name</h3>
		<Input bind:value={name} placeholder="Name" />
		<h3>Description</h3>
		<Textarea bind:value={description} placeholder="Description" />
		<Button
			style="margin-left:auto; margin-top: 1rem;"
			color="secondary"
			border="tertiary"
			on:click={create}
		>
			Create
		</Button>
	</Card>
</Page>
