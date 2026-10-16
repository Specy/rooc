import { DIET, euro } from '../data.js';

// Il problema: limiti della dieta e tabella degli ingredienti
const slide = document.getElementById('problema');
const g = (n) => `<b>${n} g</b>`;

slide.querySelector('.diet').innerHTML = DIET.nutrients
    .map(
        (n) => `
        <div class="diet-row">
            <span><span class="dot" style="--c: ${n.color}"></span>${n.name}</span>
            <span class="diet-limit">${n.min > 0 ? `almeno ${g(n.min)}, al massimo ${g(n.max)}` : `al massimo ${g(n.max)}`}</span>
        </div>`,
    )
    .join('');

slide.querySelector('.foods').innerHTML = `
    <tr>
        <th>Ingrediente</th>
        <th>Costo</th>
        ${DIET.nutrients.map((n) => `<th><span class="dot" style="--c: ${n.color}"></span>${n.name}</th>`).join('')}
    </tr>
    ${DIET.foods
        .map(
            (f) => `
        <tr>
            <td>${f.name}</td>
            <td><b>${euro(f.cost)}</b></td>
            ${f.values.map((v) => `<td>${v} g</td>`).join('')}
        </tr>`,
        )
        .join('')}`;
