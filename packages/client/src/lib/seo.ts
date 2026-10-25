export const SITE_URL = 'https://rooc.specy.app';

/** The same Person specy.app defines; the shared `@id` makes it one entity across all the sites. */
const AUTHOR = {
	'@type': 'Person',
	'@id': 'https://specy.app/#person',
	name: 'Enrico Menichelli',
	alternateName: 'Specy',
	url: 'https://specy.app',
	sameAs: ['https://github.com/Specy']
} as const;

export function toAbsoluteUrl(pathname: string) {
	return `${SITE_URL}${pathname.startsWith('/') ? '' : '/'}${pathname}`;
}

/** `</script>` inside a JSON string would close the surrounding tag; escaping the three
 *  characters that can do that keeps it valid JSON but inert as markup. */
export function serializeJsonLd(value: unknown) {
	return JSON.stringify(value)
		.replace(/</g, '\\u003c')
		.replace(/>/g, '\\u003e')
		.replace(/&/g, '\\u0026');
}

export function softwareApplicationLd() {
	return {
		'@context': 'https://schema.org',
		'@type': 'SoftwareApplication',
		name: 'ROOC',
		description:
			'A modeling language and web platform to write and solve mixed integer linear optimization models, with in-browser solvers and a visible solving pipeline.',
		url: SITE_URL,
		applicationCategory: 'DeveloperApplication',
		operatingSystem: 'Any (web browser)',
		offers: { '@type': 'Offer', price: 0, priceCurrency: 'USD' },
		featureList: [
			'Optimization modeling language with a web editor',
			'Built-in HiGHS and microlp solvers running in the browser',
			'Step by step solving pipeline with intermediate results',
			'Export models as CPLEX LP'
		],
		author: AUTHOR,
		sameAs: ['https://github.com/Specy/rooc', 'https://crates.io/crates/rooc']
	};
}

export const DEFAULT_OG_IMAGE = toAbsoluteUrl('/images/rooc-editor-wide.webp');
export const DOCS_ROOT = '/docs/rooc';

const PUBLISHER = {
	'@type': 'Organization',
	name: 'ROOC',
	url: SITE_URL
} as const;

export function techArticleLd({
	headline,
	description,
	url
}: {
	headline: string;
	description: string;
	url: string;
}) {
	return {
		'@context': 'https://schema.org',
		'@type': 'TechArticle',
		headline,
		description,
		url,
		mainEntityOfPage: url,
		inLanguage: 'en',
		author: AUTHOR,
		publisher: PUBLISHER
	};
}

export function breadcrumbLd(items: { name: string; path: string }[]) {
	return {
		'@context': 'https://schema.org',
		'@type': 'BreadcrumbList',
		itemListElement: items.map((item, index) => ({
			'@type': 'ListItem',
			position: index + 1,
			name: item.name,
			item: toAbsoluteUrl(item.path)
		}))
	};
}

/** Home > Docs > current page. The docs index is the second crumb, so it is not repeated. */
export function docsBreadcrumbs(pathname: string, headline: string) {
	const items = [
		{ name: 'ROOC', path: '/' },
		{ name: 'ROOC documentation', path: DOCS_ROOT }
	];
	if (pathname.replace(/\/$/, '') !== DOCS_ROOT) {
		items.push({ name: headline, path: pathname });
	}
	return items;
}
