<script lang="ts">
	import type { ColorName } from '$src/stores/themeStore';
	import { onMount } from 'svelte';

	interface Props {
		value: string;
		placeholder?: string;
		color?: ColorName;
		el?: HTMLInputElement;
		style?: string;
		focus?: boolean;
	}

	let {
		value = $bindable(),
		placeholder = '',
		color = 'secondary',
		el = $bindable(null),
		style = '',
		focus = false
	}: Props = $props();
	onMount(() => {
		if(focus) el?.focus()
	})
</script>

<input
	bind:value
	{placeholder}
	type="text"
	bind:this={el}
	style="border: solid 0.1rem var(--{color}); background-color: var(--{color}); color: var(--{color}-text); {style}"
/>

<style lang="scss">
	input {
		padding: 0.5rem 0.8rem;
		border-radius: 0.3rem;
	}
</style>
