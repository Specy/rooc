import { Background } from './background.js';

/* ------------------------------------------------------------ deck --
 * Navigazione:  → ↓ Spazio PagGiù Invio  avanti
 *               ← ↑ PagSu Backspace       indietro
 *               Home                      prima slide
 *               F schermo intero,  B o .  schermo nero
 * In basso a destra ci sono anche i pulsanti avanti e indietro.
 * La posizione resta nell'URL (#slide.step), quindi ricaricando la
 * pagina si torna allo stesso punto.
 *
 * Ogni slide può registrare un comportamento con
 *   Deck.define('id-slide', {
 *       steps,                 // numero di passi (default: max data-step)
 *       enter(slide),          // la slide diventa attiva
 *       step(slide, n, dir),   // passo n raggiunto, dir = 1 avanti, -1 indietro
 *       leave(slide),          // la slide viene lasciata
 *       printSteps,            // in stampa, una pagina per ognuno dei passi 1..printSteps
 *       print(page),           // prepara la copia della slide da stampare
 *   })
 * step() riceve sempre la slide su cui agire: in stampa è una sua copia.
 * Per le slide interattive: Deck.scale è il fattore di scala dello stage,
 * utile per convertire le coordinate del mouse.
 *
 * Stampa: ogni slide diventa una pagina 1920x1080 allo stato finale, con il
 * suo sfondo. Le slide in cui i passi sostituiscono il contenuto (printSteps)
 * diventano una pagina per passo. */
export const Deck = (() => {
    const W = 1920;
    const H = 1080;
    const stage = document.getElementById('stage');
    const progress = document.getElementById('progress');
    const counter = document.getElementById('counter');
    const stepDots = document.getElementById('steps');
    const printRoot = document.getElementById('print');
    const slides = [...stage.querySelectorAll(':scope > .slide')];
    const hooks = {};
    let index = -1;
    let step = 0;
    let scale = 1;

    const clamp = (v, lo, hi) => Math.max(lo, Math.min(hi, v));
    const hook = (slide) => hooks[slide.id] ?? {};

    function stepsOf(slide) {
        if (hook(slide).steps != null) return hook(slide).steps;
        let max = 0;
        for (const el of slide.querySelectorAll('[data-step]')) max = Math.max(max, Number(el.dataset.step) || 0);
        return max;
    }

    // il primo passo è già visibile quando si entra nella slide, così non
    // resta mai il solo titolo
    const firstStep = (slide) => Math.min(1, stepsOf(slide));

    function showSteps(slide, s) {
        for (const el of slide.querySelectorAll('[data-step]')) {
            el.classList.toggle('step-visible', s >= Number(el.dataset.step));
        }
    }

    function go(i, s = 0, dir = 1) {
        i = clamp(i, 0, slides.length - 1);
        const slide = slides[i];
        if (i !== index) {
            const previous = slides[index];
            if (previous) {
                previous.classList.remove('active');
                hook(previous).leave?.(previous);
            }
            void slide.offsetWidth; // riavvia le animazioni di ingresso
            slide.classList.add('active');
            index = i;
            Background.show(i + 1);
            hook(slide).enter?.(slide);
        }
        step = clamp(s, firstStep(slide), stepsOf(slide));
        showSteps(slide, step);
        hook(slide).step?.(slide, step, dir);

        progress.style.width = `${(index / Math.max(1, slides.length - 1)) * 100}%`;
        counter.textContent = `${index + 1} / ${slides.length}`;
        // un pallino per ogni pressione prima della slide successiva: pieni
        // quelli già fatti, vuoti quelli che mancano
        stepDots.innerHTML = Array.from(
            { length: stepsOf(slide) - firstStep(slide) },
            (_, k) => `<i class="${k < step - firstStep(slide) ? 'done' : ''}"></i>`,
        ).join('');
        counter.style.visibility = index === 0 ? 'hidden' : 'visible';
        history.replaceState(null, '', `#${index + 1}${step ? '.' + step : ''}`);
    }

    function next() {
        if (step < stepsOf(slides[index])) go(index, step + 1, 1);
        else if (index < slides.length - 1) go(index + 1, 0, 1);
    }

    function prev() {
        if (step > firstStep(slides[index])) go(index, step - 1, -1);
        else if (index > 0) go(index - 1, stepsOf(slides[index - 1]), -1);
    }

    function fit() {
        scale = Math.min(window.innerWidth / W, window.innerHeight / H);
        stage.style.setProperty('--scale', scale);
    }

    function onKey(e) {
        const t = e.target;
        if (t.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName)) return;
        if (e.ctrlKey || e.metaKey || e.altKey) return;
        const actions = {
            ArrowRight: next, ArrowDown: next, ' ': next, PageDown: next, Enter: next,
            ArrowLeft: prev, ArrowUp: prev, PageUp: prev, Backspace: prev,
            Home: () => go(0),
            f: () => document.fullscreenElement
                ? document.exitFullscreen()
                : document.documentElement.requestFullscreen(),
            b: () => document.body.classList.toggle('blackout'),
            '.': () => document.body.classList.toggle('blackout'),
        };
        const action = actions[e.key.length === 1 ? e.key.toLowerCase() : e.key];
        if (!action) return;
        e.preventDefault();
        action();
    }

    function fromHash() {
        const [s, st] = location.hash.slice(1).split('.').map(Number);
        return [(s || 1) - 1, st || 0];
    }

    // Le pagine da stampare sono copie delle slide, così la presentazione
    // resta com'era. Le copie non hanno id e non caricano iframe.
    function preparePrint() {
        printRoot.replaceChildren();
        slides.forEach((slide, i) => {
            const h = hook(slide);
            const background = `url(${Background.image(i + 1)})`;
            const steps = h.printSteps
                ? Array.from({ length: h.printSteps }, (_, k) => k + 1)
                : [stepsOf(slide)];
            for (const s of steps) {
                const page = slide.cloneNode(true);
                page.removeAttribute('id');
                page.classList.remove('active');
                page.querySelectorAll('iframe').forEach((iframe) => iframe.remove());
                page.style.backgroundImage = background;
                h.print?.(page);
                showSteps(page, s);
                h.step?.(page, s, 1);
                printRoot.append(page);
            }
        });
    }

    let idleTimer;
    function wake() {
        document.body.classList.remove('idle');
        clearTimeout(idleTimer);
        idleTimer = setTimeout(() => document.body.classList.add('idle'), 2000);
    }

    function start() {
        fit();
        window.addEventListener('resize', fit);
        window.addEventListener('keydown', onKey);
        window.addEventListener('mousemove', wake);
        // i pulsanti non prendono il focus, così Spazio e Invio restano alle slide
        for (const [id, action] of [['nav-prev', prev], ['nav-next', next]]) {
            const button = document.getElementById(id);
            button.addEventListener('mousedown', (e) => e.preventDefault());
            button.addEventListener('click', action);
        }
        window.addEventListener('beforeprint', preparePrint);
        window.addEventListener('afterprint', () => printRoot.replaceChildren());
        window.addEventListener('hashchange', () => {
            const [s, st] = fromHash();
            if (s !== index || st !== step) go(s, st);
        });
        const [s, st] = fromHash();
        go(s, st);
        wake();
    }

    return {
        define: (id, behaviour) => (hooks[id] = behaviour),
        start,
        next,
        prev,
        go: (slideNumber, s = 0) => go(slideNumber - 1, s),
        get scale() {
            return scale;
        },
    };
})();
