<script lang="ts">
	import ThemeProvider from '$cmp/theme/ThemeProvider.svelte';
	import { currentTheme, themeStorage } from '$stores/themeStore';
	import ErrorLogger from '$cmp/ErrorLogger.svelte';
	import PageTransition from '$cmp/PageTransition.svelte';
	import { page } from '$app/stores';
	import '../global.css';
	import { onMount } from 'svelte';
	import { Monaco } from '$lib/Monaco';
	onMount(() => {
		themeStorage.load();
		Monaco.load()
		return () => {
			Monaco.dispose()
		}
	});
</script>

<ThemeProvider
	theme={currentTheme}
	style="color: var(--primary-text); flex: 1; background-color: var(--background);"
>
	<ErrorLogger>
		<PageTransition refresh={$page.url.pathname} />
		<slot />
	</ErrorLogger>
</ThemeProvider>
