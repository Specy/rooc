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
	import {preloadHighs} from "$lib/appPipes/AppPipes";
	import { toAbsoluteUrl } from '$lib/seo';
	interface Props {
		children?: import('svelte').Snippet;
	}

	let { children }: Props = $props();

	// Emitted once here rather than per page: both values derive from the URL.
	let canonicalUrl = $derived(toAbsoluteUrl($page.url.pathname));
	onMount(() => {
		registerServiceWorker();
		preloadHighs()
		themeStorage.load();
	});
</script>

<svelte:head>
	<link rel="canonical" href={canonicalUrl} />
	<meta property="og:url" content={canonicalUrl} />
	<meta property="og:site_name" content="ROOC" />
	<meta name="twitter:card" content="summary_large_image" />
</svelte:head>

<ThemeProvider
	theme={currentTheme}
	style="color: var(--primary-text); flex: 1; background-color: var(--background);"
>
	<ErrorLogger>
		<PromptProvider>
			<PageTransition refresh={$page.url.pathname} />
			{@render children?.()}
		</PromptProvider>
	</ErrorLogger>
	<NoiseOverlay opacity={0.08} />
</ThemeProvider>
