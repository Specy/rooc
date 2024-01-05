<script lang="ts">
	import { createDerivedTheme } from "$lib/theme/svelteTheme"
	export let style = ""
	export let theme: any //i have no idea how to type this
	let themeCss = createDerivedTheme(theme, [5, 10, 15])


</script>


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