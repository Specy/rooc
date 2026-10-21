import { highlightRooc } from '../rooc.js';

/* Esempi di linearizzazione: a sinistra il frammento ROOC, a destra il
 * frammento corrispondente del LinearModel prodotto da ROOC (output reale
 * di LinearModelPipe, senza i vincoli che non cambiano). */
const EXAMPLES = [
    {
        title: 'Absolute value',
        note: 'one auxiliary variable and two constraints',
        rooc: ['min abs { x - y }'],
        linear: ['min $abs_0', 's.t.', '    $abs_0 + x - y >= 0', '    $abs_0 - x + y >= 0'],
    },
    {
        title: 'Minimum and maximum',
        note: 'if one side is enough, no binary variables',
        rooc: ['max { x, y } <= 10'],
        linear: ['$max_0 <= 10', '$max_0 - y >= 0', '$max_0 - x >= 0'],
    },
    {
        title: 'Logical operators',
        note: 'on boolean variables, they become linear constraints',
        rooc: ['x or y', 'x implies y'],
        linear: ['x + y >= 1', 'x - y <= 0'],
    },
];

const variables = new Set(['x', 'y', 'z']);
const code = (lines) =>
    `<pre class="code card">${lines
        .map((line) => `<span class="line">${highlightRooc(line, new Set(), variables)}</span>`)
        .join('')}</pre>`;

const grid = document.querySelector('#linearizzazione .lin-grid');
grid.insertAdjacentHTML(
    'beforeend',
    EXAMPLES.map(
        (e, i) => `
        <div class="lin-title" data-step="${i + 1}"><h3>${e.title}</h3><p>${e.note}</p></div>
        <div data-step="${i + 1}">${code(e.rooc)}</div>
        <i class="icon flow" data-icon="chevron-right" data-step="${i + 1}"></i>
        <div data-step="${i + 1}">${code(e.linear)}</div>`,
    ).join('') +
        // gli altri costrutti linearizzati, compaiono con l'ultimo esempio
        `
        <div class="lin-title lin-more" data-step="${EXAMPLES.length}" style="--i: 2"><h3>And more…</h3></div>
        <p class="lin-more-list" data-step="${EXAMPLES.length}" style="--i: 2">
            <code>not</code>, <code>iff</code>, <code>xor</code>, logical values used in arithmetic
            expressions, and exact formulations with binary variables
        </p>`,
);
