<script lang="ts">
	import { navigationStore } from '$stores/navigationStore';
	import { fly } from 'svelte/transition';
	interface Props {
		cropped?: boolean | string;
		style?: string;
		contentStyle?: string;
		padding?: string;
		mobilePadding?: string;
		gap?: string;
		children?: import('svelte').Snippet;
	}

	let {
		cropped = false,
		style = '',
		contentStyle = '',
		padding = '0',
		mobilePadding = '0',
		gap = '0',
		children
	}: Props = $props();
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
		{@render children?.()}
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
	    flex:1;
		padding: var(--padding);
		@media (max-width: 768px) {
			padding: var(--mobile-padding);
		}
	}
</style>
