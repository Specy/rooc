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
 *
 * Directory indexes are listed with their trailing slash: that is the form the server
 * returns 200 for, while the bare form of `/docs/lib` 308-redirects to `/docs/lib/`.
 * A `name.html` sitting next to a `name/index.html` is skipped, because both serve the
 * same path segment and the directory index is the page that carries the reference.
 */
function generatedDocsRoutes() {
	const files = Object.keys(import.meta.glob('/static/docs/lib/**/*.html', { query: '?url' }));
	const directoryIndexes = new Set(
		files
			.filter((file) => file.endsWith('/index.html'))
			.map((file) => file.slice(0, -'index.html'.length))
	);
	const routes = files.flatMap((file) => {
		if (file.endsWith('/index.html')) {
			return [file.replace('/static', '').slice(0, -'index.html'.length)];
		}
		if (directoryIndexes.has(file.replace(/\.html$/, '/'))) {
			return [];
		}
		return [file.replace('/static', '').replace(/\.html$/, '')];
	});
	return [...new Set(routes)];
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
	const entries = [...new Set([...routes(), ...generatedDocsRoutes()])].map(
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
