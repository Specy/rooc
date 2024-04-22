<script lang="ts">
	import ThemeProvider from '$cmp/theme/ThemeProvider.svelte';
	import { currentTheme, themeStorage } from '$stores/themeStore';
	import ErrorLogger from '$cmp/ErrorLogger.svelte';
	import PageTransition from '$cmp/PageTransition.svelte';
	import { page } from '$app/stores';
	import '../global.css';
	import { onMount } from 'svelte';
	import NoiseOverlay from '$cmp/layout/NoiseOverlay.svelte';
	import PromptProvider from '$cmp/PromptProvider.svelte';
	import { registerServiceWorker } from '$src/lib/register-sw';
	onMount(() => {
		registerServiceWorker();
		themeStorage.load();
	});
</script>

<ThemeProvider
	theme={currentTheme}
	style="color: var(--primary-text); flex: 1; background-color: var(--background);"
>
	<ErrorLogger>
		<PromptProvider>
			<PageTransition refresh={$page.url.pathname} />
			<slot />
		</PromptProvider>
	</ErrorLogger>
	<NoiseOverlay opacity={0.08} />
</ThemeProvider>
