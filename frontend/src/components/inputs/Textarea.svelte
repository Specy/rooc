<script lang="ts">
	import type { ColorName } from '$src/stores/themeStore';

	export let color: ColorName = 'secondary';
	export let value: string;
    export let placeholder = '';
</script>

<textarea
    {placeholder}
	style="border: solid 0.1rem var(--{color}); background-color: var(--{color}); color: var(--{color}-text);"
	bind:value
></textarea>

<style lang="scss">
	textarea {
		border-radius: 0.3rem;
		padding: 0.5rem 0.8rem;
	}
</style>
