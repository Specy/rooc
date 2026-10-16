import { DIET_ROOC, DIET_ROOC_CONSTANTS } from '../diet-rooc.js';
import { Deck } from '../deck.js';
import { highlightRooc } from '../rooc.js';
import { domainTex, instanceTex, modelTex } from '../tex.js';

const slide = document.getElementById('linguaggio');

slide.querySelector('.model-tex').textContent = modelTex({ groups: true, withDomain: false });
slide.querySelector('.instance-tex').textContent = instanceTex();
slide.querySelector('.domain-tex').textContent = domainTex();

const code = slide.querySelector('.code');
code.innerHTML = DIET_ROOC
    .map(([group, line]) => `<span class="line grp-${group}">${highlightRooc(line, DIET_ROOC_CONSTANTS) || ' '}</span>`)
    .join('');

/* Frecce: per ogni sezione una parentesi "]" a destra delle formule, una "["
 * a sinistra del codice e una freccia che le collega. Compaiono insieme al
 * codice, al passo 2, una dopo l'altra. La geometria si calcola quando la slide è
 * attiva, perché dipende dall'impaginazione di KaTeX. */
const GROUPS = [1, 2, 3, 4];
const BRACKET = 10;
const INSET = 6; // spazio tra parentesi di sezioni adiacenti
const svg = slide.querySelector('.links');
svg.innerHTML = GROUPS.map(
    (g) => `
    <g class="link" data-step="2" style="--i: ${g + 2}">
        <path class="bracket" />
        <path class="bracket" />
        <path class="curve" pathLength="1" />
        <path class="head" />
    </g>`,
).join('');

// Rettangolo che contiene tutti gli elementi, in coordinate della slide
// (1920x1080). Si misura rispetto alla colonna che li contiene, così la
// traslazione dell'animazione di ingresso non sposta le misure.
function bounds(column, elements) {
    const scale = Deck.scale;
    const base = column.getBoundingClientRect();
    const box = { top: Infinity, bottom: -Infinity, left: Infinity, right: -Infinity };
    for (const el of elements) {
        const r = el.getBoundingClientRect();
        box.top = Math.min(box.top, (r.top - base.top) / scale + column.offsetTop);
        box.bottom = Math.max(box.bottom, (r.bottom - base.top) / scale + column.offsetTop);
        box.left = Math.min(box.left, (r.left - base.left) / scale + column.offsetLeft);
        box.right = Math.max(box.right, (r.right - base.left) / scale + column.offsetLeft);
    }
    return box;
}

function layout() {
    const mathSide = slide.querySelector('.math-side');
    const codeSide = slide.querySelector('.code-side');
    const left = GROUPS.map((g) => bounds(mathSide, mathSide.querySelectorAll(`.grp-${g}`)));
    const right = GROUPS.map((g) => bounds(codeSide, code.querySelectorAll(`.grp-${g}`)));
    const xLeft = Math.max(...left.map((b) => b.right)) + 28;
    const xRight = bounds(codeSide, [code]).left - 24;

    GROUPS.forEach((g, k) => {
        const [bracketL, bracketR, curve, head] = svg.children[k].children;
        const l = { top: left[k].top + INSET, bottom: left[k].bottom - INSET };
        const r = { top: right[k].top + INSET, bottom: right[k].bottom - INSET };
        bracketL.setAttribute('d', `M${xLeft - BRACKET} ${l.top}H${xLeft}V${l.bottom}H${xLeft - BRACKET}`);
        bracketR.setAttribute('d', `M${xRight + BRACKET} ${r.top}H${xRight}V${r.bottom}H${xRight + BRACKET}`);
        const y1 = (l.top + l.bottom) / 2;
        const y2 = (r.top + r.bottom) / 2;
        const x1 = xLeft + 12;
        const x2 = xRight - 22;
        const dx = (x2 - x1) / 2;
        curve.setAttribute('d', `M${x1} ${y1}C${x1 + dx} ${y1} ${x2 - dx} ${y2} ${x2} ${y2}`);
        head.setAttribute('d', `M${x2} ${y2 - 7}L${x2 + 12} ${y2}L${x2} ${y2 + 7}Z`);
    });
}

// calcolata anche subito, così le frecce ci sono pure nelle pagine stampate
// senza essere passati dalla slide
document.fonts.ready.then(layout);
Deck.define('linguaggio', {
    enter() {
        document.fonts.ready.then(layout);
    },
});
