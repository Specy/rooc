import { DIET } from './data.js';

// Lo stesso modello in ROOC, riga per riga con la sezione a cui appartiene
// (1 obiettivo, 2 vincoli, 3 dati, 4 variabili). Compilato con ROOC produce
// esattamente il modello compilato della slide "Modello e istanza".
const { foods, nutrients } = DIET;
const list = (values) => `[${values.join(', ')}]`;
export const DIET_ROOC = [
    [1, 'min sum(i in 0..n) { C[i] * x_i }'],
    [2, 's.t.'],
    [2, '    sum(i in 0..n) { T[i][j] * x_i } >= Nmin[j] for j in 0..m'],
    [2, '    sum(i in 0..n) { T[i][j] * x_i } <= Nmax[j] for j in 0..m'],
    [3, 'where'],
    [3, `    let C = ${list(foods.map((f) => f.cost.toFixed(1)))}`],
    [3, `    let Nmin = ${list(nutrients.map((n) => n.min))}`],
    [3, `    let Nmax = ${list(nutrients.map((n) => n.max))}`],
    [3, '    let T = ['],
    ...foods.map((f, i) => [3, `        ${list(f.values)}${i < foods.length - 1 ? ',' : ''}`]),
    [3, '    ]'],
    [3, '    let n = len(C)'],
    [3, '    let m = len(Nmin)'],
    [4, 'define'],
    [4, '    x_i as NonNegativeReal for i in 0..n'],
];
export const DIET_ROOC_CONSTANTS = new Set(['C', 'T', 'Nmin', 'Nmax', 'n', 'm']);
