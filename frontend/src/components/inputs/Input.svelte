<script lang="ts">
	import type { ColorName } from '$src/stores/themeStore';
	import { onMount } from 'svelte';

	export let value: string;
	export let placeholder = '';
	export let color: ColorName = 'secondary';
	export let el:HTMLInputElement = null
	export let style = ''
	export let focus = false
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
