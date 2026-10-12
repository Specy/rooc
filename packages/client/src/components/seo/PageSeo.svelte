<script lang="ts">
	import { page } from '$app/stores';
	import {
		breadcrumbLd,
		docsBreadcrumbs,
		serializeJsonLd,
		techArticleLd,
		toAbsoluteUrl
	} from '$lib/seo';

	interface Props {
		title: string;
		description: string;
		/** Short name used in structured data and breadcrumbs; defaults to the title. */
		headline?: string;
	}

	let { title, description, headline = title }: Props = $props();

	// Canonical, og:url, og:image and twitter:card come from the root layout; this adds the rest.
	let url = $derived(toAbsoluteUrl($page.url.pathname));
	let article = $derived(serializeJsonLd(techArticleLd({ headline, description, url })));
	let crumbs = $derived(
		serializeJsonLd(breadcrumbLd(docsBreadcrumbs($page.url.pathname, headline)))
	);
</script>

<svelte:head>
	<title>{title}</title>
	<meta name="description" content={description} />
	<meta property="og:title" content={title} />
	<meta property="og:description" content={description} />
	<meta property="og:type" content="article" />
	<meta name="twitter:title" content={title} />
	<meta name="twitter:description" content={description} />
	{@html `<script type="application/ld+json">${article}</script>`}
	{@html `<script type="application/ld+json">${crumbs}</script>`}
</svelte:head>
