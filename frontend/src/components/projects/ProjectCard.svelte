<script lang="ts">
	import ButtonLink from '$cmp/inputs/ButtonLink.svelte';
	import Card from '$cmp/layout/Card.svelte';
	import Column from '$cmp/layout/Column.svelte';
	import Row from '$cmp/layout/Row.svelte';
	import type { Project } from '$src/stores/projectStore';
	import { createEventDispatcher } from 'svelte';
	export let project: Project;
	import sago from 's-ago';
	import Delete from '~icons/fa/Trash.svelte';
	import Download from '~icons/fa/Download.svelte';
	import Button from '$cmp/inputs/Button.svelte';
	const dispatcher = createEventDispatcher<{
		onUpdate: { project: Project };
		onDownoad: { project: Project };
		onDelete: { project: Project };
	}>();
	let name = project.name;
	let description = project.description;

	function save() {
		dispatcher('onUpdate', { project: { ...project, name, description } });
	}
</script>

<Card border="secondary" style="position:relative; height: 10rem;" radius="0.4rem">
	<div class="blurred-underlay">
		{project.content}
	</div>
	<Column padding="0.5rem; margin-bottom: 0.4rem;" style="flex:1;">
		<div class="title" contenteditable bind:textContent={name} spellcheck="false" on:blur={save} />
		<div
			class="description"
			contenteditable
			bind:textContent={description}
			spellcheck="false"
			on:blur={save}
		/>
		<div class="project-dates">
			<span>Last update </span> <span>{sago(new Date(project.updatedAt))}</span>
			<span>Created </span> <span>{sago(new Date(project.createdAt))}</span>
		</div>
	</Column>
	<Row
		background="secondary"
		style="border-radius:0 0 0.4rem 0.4rem;"
		padding="0.3rem"
		justify="end"
		gap="0.5rem"
	>
		<Button on:click={() => dispatcher('onDownoad', { project })} hasIcon>
			<Download />
		</Button>
		<Button on:click={() => dispatcher('onDelete', { project })} hoverColor="danger" hasIcon>
			<Delete />
		</Button>
		<ButtonLink href="/projects/{project.id}" title="Open" border="secondary">Open</ButtonLink>
	</Row>
</Card>

<style lang="scss">
	.blurred-underlay {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		opacity: 0.5;
		filter: blur(0.5rem);
		z-index: -1;
	}
	.project-dates {
		display: grid;
		padding: 0 0.5rem;
		margin-top: 0.5rem;
		gap: 0.2rem;
		grid-template-columns: 1fr 1fr;
		font-size: 0.8rem;
		color: var(--hint);
	}
	.description {
		display: flex;
		flex: 1;
		padding: 0.5rem;
		background-color: transparent;
		transition: all 0.2s;
		border-radius: 0.4rem;
		&:hover,
		&:focus {
			background-color: var(--primary-5);
			color: var(--secondary-text);
			filter: brightness(1.2);
		}
	}
	.title {
		font-size: 1.3rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		text-align: center;
		padding: 0.4rem;
		justify-content: center;
		border-radius: 0.4rem;
		background-color: transparent;
		transition: all 0.2s;
		&:hover,
		&:focus {
			background-color: var(--primary-5);
			color: var(--secondary-text);
			filter: brightness(1.2);
		}
	}
</style>
