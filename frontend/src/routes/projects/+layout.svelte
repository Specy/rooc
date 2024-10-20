<script lang="ts">
	import { onMount } from 'svelte';
	import { projectStore } from '$stores/userProjectsStore.svelte';
	interface Props {
		children?: import('svelte').Snippet;
	}

	let { children }: Props = $props();
	onMount(() => {
		projectStore.syncProjectsWithStore();
	});
</script>


{@render children?.()}