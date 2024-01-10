<script lang="ts">
	import type { ColorName } from '$src/stores/themeStore';
	import Column from './Column.svelte';
	import type { Justify } from './layoutUtils';

	export let justify: Justify | undefined = undefined;
	export let align: Justify | undefined = undefined;
	export let gap: string | undefined = undefined;
	export let padding: string | undefined = undefined;
	export let style: string | undefined = undefined;
	export let background: ColorName = 'primary';
	export let border: ColorName | undefined = undefined;
	export let borderStyle: string = "solid 0.1rem"
	export let radius: string = "0.5rem"
	export let withShadow: boolean = false
</script>

<Column
	{justify}
	{align}
	{gap}
	{padding}
	{background}
	style="
	{border ? `border: ${borderStyle} var(--${border})` : ''}; 
	border-radius: {radius};
	{withShadow ? 'box-shadow: 0 0.1rem 0.5rem rgba(var(--shadow-rgb), 0.2);' : ''};
	{style}"
>
	<slot />
</Column>
