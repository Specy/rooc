<script lang="ts">
	import type { ColorName } from '$src/stores/themeStore';
	import Button from './Button.svelte';
	export let disabled = false;
	export let style = '';
	export let hasIcon = false;
	export let color: ColorName | undefined = undefined;
	export let border: ColorName | undefined = undefined;
	export let active = false;
	export let title = '';
	export let href: string;
</script>

<a
	{href}
	{title}
	on:click={(e) => {
		if (disabled) {
			e.preventDefault();
		}
	}}
>
	<Button {disabled} {style} {hasIcon} {color} {active} {title} {border}>
		<slot />
	</Button>
</a>
