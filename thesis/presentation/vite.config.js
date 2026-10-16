import { defineConfig } from 'vite';
import { viteSingleFile } from 'vite-plugin-singlefile';

// Il build produce un unico dist/presentation.html con JS, CSS, font e
// immagini incorporati, da aprire offline con un doppio click.
export default defineConfig({
    base: './',
    build: {
        assetsInlineLimit: Number.MAX_SAFE_INTEGER,
    },
    plugins: [katexWoff2Only(), viteSingleFile(), renameHtml('presentation.html')],
});

// Il CSS di KaTeX dichiara ogni font in woff2, woff e ttf: tutti i browser
// moderni usano woff2, quindi gli altri formati sono solo peso in più.
function katexWoff2Only() {
    return {
        name: 'katex-woff2-only',
        enforce: 'pre',
        transform(code, id) {
            if (!/katex.*\.css$/.test(id)) return;
            return code.replace(/,\s*url\([^)]*?\.(?:woff|ttf)\)\s*format\("(?:woff|truetype)"\)/g, '');
        },
    };
}

function renameHtml(fileName) {
    return {
        name: 'rename-html',
        enforce: 'post',
        generateBundle(_, bundle) {
            if (bundle['index.html']) bundle['index.html'].fileName = fileName;
        },
    };
}
