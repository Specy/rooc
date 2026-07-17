<script lang="ts">
	import type { ColorName } from '$src/stores/themeStore';
	import { justifyMap, type Justify } from './layoutUtils';

	interface Props {
		justify?: Justify | undefined;
		align?: Justify | undefined;
		gap?: string | undefined;
		padding?: string | undefined;
		background?: ColorName | undefined;
		style?: string | undefined;
		flex1?: boolean | undefined
		children?: import('svelte').Snippet;
	}

	let {
		justify = undefined,
		align = undefined,
		gap = undefined,
		padding = undefined,
		background = undefined,
		style = undefined,
		flex1 = false,
		children
	}: Props = $props();
</script>

<div
	class="flex col"
	style="
    {justify ? `justify-content: ${justifyMap[justify]};` : ''}
    {align ? `align-items: ${justifyMap[align]};` : ''}
    {gap ? `gap: ${gap};` : ''}
    {padding ? `padding: ${padding};` : ''}
    {background ? `background-color: var(--${background}); color: var(--${background}-text);` : ''}
    {flex1 ? 'flex: 1;' : ''}
    {style}
    "
>
	{@render children?.()}
</div>
