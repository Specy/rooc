<script lang="ts">
	import type {ColorName} from '$src/stores/themeStore';
	import Button from './Button.svelte';

	interface Props {
		disabled?: boolean;
		style?: string;
		hasIcon?: boolean;
		color?: ColorName | undefined;
		border?: ColorName | undefined;
		active?: boolean;
		title?: string;
		href: string;
		blank?: boolean;
		iconLeft?: boolean;
		children?: import('svelte').Snippet;
	}

	let {
		disabled = false,
		style = '',
		hasIcon = false,
		color = undefined,
		border = undefined,
		active = false,
		title = '',
		href,
		blank = false,
		iconLeft = false,
		children
	}: Props = $props();
</script>

<a
	{href}
	target={blank ? "_blank" : undefined}
	{title}
	onclick={(e) => {
		if (disabled) {
			e.preventDefault();
		}
	}}
>
	<Button {disabled} {style} {hasIcon} {color} {active} {title} {border} {iconLeft}>
		{@render children?.()}
	</Button>
</a>


