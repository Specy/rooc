<script lang="ts">
	import type { ColorName } from '$src/stores/themeStore';
	import { justifyMap, type Justify } from './layoutUtils';

	export let justify: Justify | undefined = undefined
	export let align: Justify | undefined = undefined;
	export let gap: string | undefined = undefined;
	export let padding: string | undefined = undefined;
	export let background: ColorName | undefined = undefined;
    export let style: string | undefined = undefined;
</script>

<div
	class="flex col"
	style="
    {justify ? `justify-content: ${justifyMap[justify]};` : ''}
    {align ? `align-items: ${justifyMap[align]};` : ''}
    {gap ? `gap: ${gap};` : ''}
    {padding ? `padding: ${padding};` : ''}
    {background ? `background-color: var(--${background}); color: var(--${background}-text);` : ''}
    {style}
    "
>
	<slot />
</div>
