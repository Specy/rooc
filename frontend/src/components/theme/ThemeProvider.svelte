<script lang="ts">
	import { createDerivedThemeColors } from "$lib/theme/svelteTheme"
	import { TinyColor } from "@ctrl/tinycolor"
	export let style = ""
	export let theme: any //i have no idea how to type this
	let colors = createDerivedThemeColors(theme)

	function toRgb(color: string) {
		return (new TinyColor(color)
			.toRgbString()
			.match(/(\s*\d+\s*),(\s*\d+\s*),(\s*\d+\s*)/) ?? [])[0]
	}
</script>

<div
	style={`
	display: flex; 
	flex-direction: column;

    ${$colors
			.map(({ cssProp, hex, text }) => {
				const rgb = toRgb(hex)
				const rgbText = toRgb(text ?? "#FFFFFF")
				return `
				--${cssProp}: ${hex};
				--${cssProp}-text: ${text};
				--RGB-${cssProp}: ${rgb};
				--RGB-${cssProp}-text: ${rgbText};
				`
			})

			.join("\n")}
    ${style}
`}
>
	<slot />
</div>