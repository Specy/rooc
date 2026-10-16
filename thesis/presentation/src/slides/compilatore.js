import { DIET, euro, fmt } from '../data.js';
import { Deck } from '../deck.js';
import { DIET_ROOC, DIET_ROOC_CONSTANTS } from '../diet-rooc.js';
import { highlightRooc } from '../rooc.js';

// Output reale della pipeline di ROOC sul modello della dieta
// (CompilerPipe, ModelPipe, LinearModelPipe, StandardLinearModelPipe, AutoSolverPipe)
const MODEL = `min 1.5 * x_0 + 0.5 * x_1 + 2 * x_2
s.t.
    30 * x_0 + 2 * x_1 + 2 * x_2 >= 50
    0 * x_0 + 45 * x_1 + 15 * x_2 >= 200
    5 * x_0 + 0 * x_1 + 20 * x_2 >= 0
    30 * x_0 + 2 * x_1 + 2 * x_2 <= 150
    0 * x_0 + 45 * x_1 + 15 * x_2 <= 300
    5 * x_0 + 0 * x_1 + 20 * x_2 <= 70
define
    x_0, x_1, x_2 as NonNegativeReal`;

const LINEAR = `min 1.5x_0 + 0.5x_1 + 2x_2
s.t.
    30x_0 + 2x_1 + 2x_2 >= 50
    45x_1 + 15x_2 >= 200
    5x_0 + 20x_2 >= 0
    30x_0 + 2x_1 + 2x_2 <= 150
    45x_1 + 15x_2 <= 300
    5x_0 + 20x_2 <= 70
define
    x_0, x_1, x_2 as NonNegativeReal`;

const STANDARD = `min 1.5x_0 + 0.5x_1 + 2x_2
s.t.
    30x_0 + 2x_1 + 2x_2 - $su_1 = 50
    45x_1 + 15x_2 - $su_2 = 200
    5x_0 + 20x_2 - $su_3 = 0
    30x_0 + 2x_1 + 2x_2 + $sl_1 = 150
    45x_1 + 15x_2 + $sl_2 = 300
    5x_0 + 20x_2 + $sl_3 = 70
define
    x_0, x_1, x_2, $su_1, ..., $sl_3 as NonNegativeReal`;

const SOLUTION = { objective: 4.277778, values: [1.37037, 4.444444, 0] };

const rooc = (text, constants) =>
    text
        .split('\n')
        .map((line) => `<span class="line">${highlightRooc(line, constants) || ' '}</span>`)
        .join('');

// solo l'assegnamento delle variabili; il valore ottimo è a destra, come spesa
const solution = () =>
    SOLUTION.values
        .map((v, i) => `<span class="var">x_${i}</span> = ${v}`)
        .map((line) => `<span class="line">${line}</span>`)
        .join('');

// La soluzione tradotta nei termini del problema iniziale
const answer = DIET.foods
    .map((f, i) => `<li><span>${f.name}</span><b>${fmt(SOLUTION.values[i], 0, 2)} porzioni</b></li>`)
    .join('');

const VIEWS = [
    {
        code: rooc(DIET_ROOC.map(([, line]) => line).join('\n'), DIET_ROOC_CONSTANTS),
        note: '<p>Il punto di partenza: il modello e i dati dell’istanza, scritti in ROOC.</p>',
    },
    {
        code: rooc(MODEL),
        note: `<p>Le costanti vengono sostituite dai loro valori e le sommatorie espanse:
            <code class="var">x_i</code> diventa <code class="var">x_0</code>, <code class="var">x_1</code>,
            <code class="var">x_2</code>.</p>`,
    },
    {
        code: rooc(LINEAR),
        note: `<p>Ogni vincolo diventa una combinazione lineare delle variabili e i termini nulli
            spariscono.</p><p class="hint">È il modello compilato visto prima.</p>`,
    },
    {
        code: rooc(STANDARD),
        note: `<p>Il <code>LinearModel</code> viene tradotto nel formato del solver scelto: il simplesso,
            per esempio, vuole la forma standard, con le variabili di slack.</p>
            <p class="hint">Solver: microlp (predefinito), Clarabel, HiGHS, CBC, SCIP, ...</p>`,
    },
    {
        code: solution(),
        className: 'solution',
        note: `<p>La risposta alla domanda iniziale:</p>
            <ul class="answer">${answer}<li class="total"><span>Spesa</span><b>${euro(SOLUTION.objective)}</b></li></ul>`,
    },
];

const slide = document.getElementById('compilatore');
const views = slide.querySelector('.views');
views.innerHTML = VIEWS.map(
    (v) => `
    <div class="view ${v.className ?? ''}">
        <pre class="code">${v.code}</pre>
        <div class="note">${v.note}</div>
    </div>`,
).join('');

Deck.define('compilatore', {
    printSteps: VIEWS.length,
    step(slide, n) {
        const current = Math.max(1, n) - 1;
        [...slide.querySelector('.views').children].forEach((view, i) => view.classList.toggle('current', i === current));
        [...slide.querySelectorAll('.stage')].forEach((stage, i) => stage.classList.toggle('current', i === current));
    },
});
