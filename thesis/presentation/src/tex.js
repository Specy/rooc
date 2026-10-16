import { DIET } from './data.js';

/* Formule del problema della dieta in LaTeX, condivise tra le slide.
 * I dati sono colorati con la classe "data", le variabili con "var". */
export const data = (v) => `\\htmlClass{data}{${v}}`;
export const muted = (v) => `\\htmlClass{muted}{${v}}`;
export const x = (i) => `\\htmlClass{var}{x_${i}}`;
const vector = (values) => `(${values.join(',\\ ')})`;

const { foods, nutrients } = DIET;
const T = foods.map((f) => f.values);

// Modello con i simboli. Con groups ogni parte riceve la classe grp-N
// (1 obiettivo, 2 vincoli) per poterla ritrovare nella pagina; withDomain
// aggiunge in fondo la riga di non negatività, attenuata.
export function modelTex({ groups = false, withDomain = true } = {}) {
    const g = (n, v) => (groups ? `\\htmlClass{grp-${n}}{${v}}` : v);
    const sum = (coeff) => `\\sum_i ${data(coeff)}\\,${x('i')}`;
    return String.raw`
    \begin{array}{lll}
        ${g(1, '\\text{min}')} & ${g(1, sum('C_i'))} \\[6pt]
        ${g(2, '\\text{s.t.}')} & ${g(2, `${sum('T_{ij}')} \\ge ${data('\\mathit{Nmin}_j')}`)} & ${g(2, '\\forall j')} \\[6pt]
        & ${g(2, `${sum('T_{ij}')} \\le ${data('\\mathit{Nmax}_j')}`)} & ${g(2, '\\forall j')}
        ${withDomain ? String.raw`\\[6pt] & ${muted('x_i \\ge 0')} & ${muted('\\forall i')}` : ''}
    \end{array}`;
}

// Dominio delle variabili da solo
export const domainTex = () => String.raw`${x('i')} \ge 0 \quad \forall i`;

// L'istanza in colonna, allineata sul segno uguale
export function instanceTex() {
    return String.raw`
    \begin{aligned}
        ${data('C')} &= ${vector(foods.map((f) => f.cost.toFixed(1)))} \\[6pt]
        ${data('\\mathit{Nmin}')} &= ${vector(nutrients.map((n) => n.min))} \\[6pt]
        ${data('\\mathit{Nmax}')} &= ${vector(nutrients.map((n) => n.max))} \\[6pt]
        ${data('T')} &= \begin{pmatrix} ${T.map((row) => row.join(' & ')).join(' \\\\ ')} \end{pmatrix}
    \end{aligned}`;
}

// Una riga del modello compilato: le colonne alternano termine e operatore,
// così i termini della stessa variabile restano incolonnati
function row(kw, coeffs, rel = '', rhs = '') {
    const cells = [];
    let first = true;
    coeffs.forEach((c, i) => {
        if (i > 0) cells.push(c !== 0 && !first ? '{}+{}' : '');
        cells.push(c === 0 ? '' : `${c === 1 ? '' : data(c)}${x(i)}`);
        if (c !== 0) first = false;
    });
    return [kw, ...cells, rel && `{}${rel}{}`, rhs && data(rhs)].join(' & ');
}

// Modello compilato: stessa forma del LinearModel prodotto da ROOC
// (appendice della tesi): termini nulli eliminati, prima i vincoli sul
// minimo e poi quelli sul massimo, variabili x_0, x_1, x_2
export function compiledTex() {
    const column = (j) => T.map((values) => values[j]);
    const rows = [
        row('\\text{min}\\quad', foods.map((f) => f.cost)),
        ...nutrients.map((n, j) => row(j === 0 ? '\\text{s.t.}\\quad' : '', column(j), '\\ge', String(n.min))),
        ...nutrients.map((n, j) => row('', column(j), '\\le', String(n.max))),
        // la non negatività, in secondo piano: le virgole stanno nelle colonne dei "+"
        ['', ...foods.flatMap((_, i) => (i > 0 ? [muted(','), muted(`x_${i}`)] : [muted(`x_${i}`)])), muted('\\ge'), muted('0')].join(' & '),
    ];
    return String.raw`
    \begin{array}{l${'rc'.repeat(foods.length - 1)}rcl}
        ${rows.join(' \\\\\n')}
    \end{array}`;
}
