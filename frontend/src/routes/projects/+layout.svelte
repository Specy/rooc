<script lang="ts">
	import { onMount } from 'svelte';
	import { projectStore } from '$stores/userProjectsStore';
	onMount(() => {
		projectStore.syncProjectsWithStore();
	});
</script>


<slot />