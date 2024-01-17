<script lang="ts">
	import { createDerivedTheme, textColor, type ThemeStore } from '$lib/theme/svelteTheme';
	import { TinyColor } from '@ctrl/tinycolor';
	export let style = '';
	export let theme: ThemeStore<any> //i have no idea how to type this
	let themeCss = createDerivedTheme(theme, [5, 10, 15]);
	import { Body } from 'svelte-body';
	let accent = theme.getColorStore('accent');
	let background = theme.getColorStore('background');
</script>

<Body
	style={`
		--scroll-accent: ${$accent.color};
		background-color: ${$background.color};
		color: ${new TinyColor($background.color).isDark() ? textColor.light : textColor.dark};
	`}
/>

<div
	style={`
	display: flex; 
	flex-direction: column;
    ${$themeCss}
    ${style}
`}
>
	<slot />
</div>
