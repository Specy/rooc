export const SITE_URL = 'https://rooc.specy.app';

const AUTHOR = {
	'@type': 'Person',
	name: 'Specy',
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
