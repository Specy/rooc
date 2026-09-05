import { SITE_URL } from '$lib/seo';

export const prerender = true;

/** Project surfaces hold the reader's own models; they are not content to index. */
const EXCLUDED = new Set(['/projects', '/projects/new']);

/** Discovered rather than hand-listed, so a new docs page is in the sitemap as soon as
 *  it exists. Parameterised routes are skipped: they only ever render local projects. */
function routes() {
	const modules = import.meta.glob('/src/routes/**/+page.svelte');
	return Object.keys(modules)
		.map((file) => file.replace('/src/routes', '').replace('/+page.svelte', '') || '/')
		.filter((route) => !route.includes('['))
		.filter((route) => !EXCLUDED.has(route))
		.sort();
}

/**
 * The generated `@specy/rooc` API reference. TypeDoc writes these straight into static/,
 * so they are plain files rather than routes — real, searchable reference pages ("rooc
 * ModelBuilder") that nothing else would list. Globbed as raw urls so a regenerated
 * reference stays in step without a manual edit.
 */
function generatedDocsRoutes() {
	const pages = import.meta.glob('/static/docs/lib/**/*.html', { query: '?url' });
	return Object.keys(pages).map((file) =>
		// `.../index.html` is served as the directory itself, so listing `/index` would
		// point a crawler at a URL that does not resolve.
		file
			.replace('/static', '')
			.replace(/\/index\.html$/, '')
			.replace(/\.html$/, '')
	);
}

function escapeXml(value: string) {
	return value
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;')
		.replace(/'/g, '&apos;');
}

export const GET = async () => {
	const buildDate = new Date().toISOString().slice(0, 10);
	const entries = [...routes(), ...generatedDocsRoutes()].map(
		(route) => `	<url>
		<loc>${escapeXml(SITE_URL + route)}</loc>
		<lastmod>${buildDate}</lastmod>
	</url>`
	);
	const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${entries.join('\n')}
</urlset>`;
	return new Response(xml, { headers: { 'Content-Type': 'application/xml' } });
};
