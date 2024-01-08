<script lang="ts">
	import { navigationStore } from '$stores/navigationStore';
	import { fly } from 'svelte/transition';
	export let cropped: boolean | string = false;
	export let style = '';
	export let contentStyle = '';
	export let padding = '0';
	export let mobilePadding = '0';
	export let gap = '0';
</script>

<main
	class="content"
	{style}
	in:fly|global={{ x: $navigationStore.direction === 'back' ? 30 : -30, duration: 500 }}
>
	<div
		class="col content-padded"
		style="
	max-width: {cropped
			? `${typeof cropped === 'string' ? cropped : '60rem'}`
			: 'unset'}; width:100%;height: 100%; 
	--padding: {padding}; 
	--mobile-padding: {mobilePadding};
	gap:{gap}; 
	{contentStyle};"
	>
		<slot />
	</div>
</main>

<style lang="scss">
	.content {
		display: flex;
		flex-direction: column;
		align-items: center;
		position: relative;
		flex: 1;
	}
	.content-padded {
		padding: var(--padding);
		@media (max-width: 768px) {
			padding: var(--mobile-padding);
		}
	}
</style>
