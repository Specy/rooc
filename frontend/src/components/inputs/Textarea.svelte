<script lang="ts">
	import type { ColorName } from '$src/stores/themeStore';

	interface Props {
		color?: ColorName;
		value: string;
		placeholder?: string;
	}

	let { color = 'secondary', value = $bindable(), placeholder = '' }: Props = $props();
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
