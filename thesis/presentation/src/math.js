import katex from 'katex';

/* Ogni elemento con classe "tex" contiene LaTeX e viene sostituito dalla
 * formula. \htmlClass{nome}{...} dà una classe a una parte della formula,
 * così la si può colorare o evidenziare da CSS. */
export function renderMath(root) {
    for (const el of root.querySelectorAll('.tex')) {
        katex.render(el.textContent, el, {
            throwOnError: false,
            strict: false,
            trust: (ctx) => ctx.command === '\\htmlClass',
        });
    }
}
