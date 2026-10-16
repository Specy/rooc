import LZString from 'lz-string';
import { Deck } from '../deck.js';
import { DIET_ROOC } from '../diet-rooc.js';

// Stesso formato del pulsante "share" dell'editor: il progetto in JSON,
// compresso con lz-string nell'URL. Il resto dei campi lo completa l'editor.
const project = {
    id: 'share',
    name: 'Problema della dieta',
    description: 'Il modello usato nella presentazione',
    content: DIET_ROOC.map(([, line]) => line).join('\n'),
};
const url = `https://rooc.specy.app/projects/share?project=${LZString.compressToEncodedURIComponent(JSON.stringify(project))}`;

const slide = document.getElementById('demo');
const iframe = slide.querySelector('iframe');

// l'editor si carica solo quando ci si avvicina alla demo, non all'apertura.
// Senza connessione si mostrano gli screenshot al suo posto.
function load() {
    if (!navigator.onLine) slide.classList.add('offline');
    else if (!iframe.src) iframe.src = url;
}

/* L'editor, appena caricato, si prende il focus della tastiera e le frecce
 * non cambierebbero più slide. Finché non lo si attiva cliccandoci sopra (il
 * pulsante trasparente che lo copre) il focus gli viene tolto appena lo
 * prende. Quando il focus torna alle slide, per esempio cliccando fuori
 * dall'iframe, l'editor si blocca di nuovo. */
const activate = slide.querySelector('.demo-activate');
let active = false;

function lock() {
    active = false;
    iframe.inert = true;
    activate.hidden = false;
}

activate.addEventListener('click', () => {
    active = true;
    iframe.inert = false;
    activate.hidden = true;
    iframe.focus();
});

window.addEventListener('blur', () => {
    setTimeout(() => {
        if (!active && document.activeElement === iframe) iframe.blur();
    });
});
window.addEventListener('focus', () => {
    if (active) lock();
});

/* La slide non ha passi. Solo nella versione offline le frecce scorrono gli
 * screenshot, uno per punto della scaletta */
const shots = [...slide.querySelectorAll('.demo-offline img')];
const slideNumber = [...slide.parentElement.children].filter((el) => el.matches('.slide')).indexOf(slide) + 1;
const offline = () => slide.classList.contains('offline');

// O passa a mano agli screenshot, per esempio se il sito è lento
window.addEventListener('keydown', (e) => {
    if (e.key.toLowerCase() === 'o' && slide.classList.contains('active') && !e.ctrlKey && !e.metaKey) {
        slide.classList.toggle('offline');
        Deck.go(slideNumber, 1);
    }
});

Deck.define('scelte', { enter: load });
// in stampa l'editor non c'è: una pagina per ogni screenshot
Deck.define('demo', {
    get steps() {
        return offline() ? shots.length : 0;
    },
    enter: load,
    leave: lock,
    printSteps: shots.length,
    print(page) {
        page.classList.add('offline');
    },
    step(slide, n) {
        slide.querySelectorAll('.demo-offline img').forEach((img, i) => img.classList.toggle('current', i === Math.max(1, n) - 1));
    },
});
