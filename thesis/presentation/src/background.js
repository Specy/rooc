/* ---------------------------------------------------------- sfondo --
 * Porting di specy-app/src/components/Background.svelte: una griglia di
 * game of life fatta evolvere per qualche generazione, disegnata a bassa
 * risoluzione, sfocata e sfumata verso il basso. Rispetto al sito la griglia
 * è più grossa (blob più grandi) e si richiude sui bordi, così i blob
 * possono stare anche ai lati. Il seed dipende dalla slide, quindi ogni
 * slide ha sempre lo stesso sfondo. */
export const Background = (() => {
    const root = document.getElementById('background');
    const BLUR = 6;
    const BASE = 30;
    const BIAS = 0.25;
    const GENERATIONS = 20;
    const hasFilter = 'filter' in CanvasRenderingContext2D.prototype;
    const multiplier = hasFilter ? 5 : 2;
    let current = null;
    let currentSeed = 0;

    function mulberry32(seed) {
        return () => {
            seed |= 0;
            seed = (seed + 0x6d2b79f5) | 0;
            let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
            t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
            return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
        };
    }

    function size(ratio) {
        const height = Math.ceil(BASE / ratio / 4) * 4;
        return { width: BASE, height };
    }

    function generation(m, width, height) {
        const next = Array.from({ length: height }, () => new Uint8Array(width));
        for (let i = 0; i < height; i++) {
            const up = m[(i + height - 1) % height];
            const row = m[i];
            const down = m[(i + 1) % height];
            for (let j = 0; j < width; j++) {
                const l = (j + width - 1) % width;
                const r = (j + 1) % width;
                const n = up[l] + up[j] + up[r] + row[l] + row[r] + down[l] + down[j] + down[r];
                next[i][j] = row[j] ? (n === 2 || n === 3 ? 1 : 0) : n === 3 ? 1 : 0;
            }
        }
        return next;
    }

    function render(seed, ratio = window.innerWidth / window.innerHeight) {
        const { width, height } = size(ratio);
        const random = mulberry32(seed * 9973 + 17);
        let m = Array.from({ length: height }, () =>
            Uint8Array.from({ length: width }, () => Math.round(random() - BIAS)),
        );
        for (let g = 0; g < GENERATIONS; g++) m = generation(m, width, height);

        const accent = getComputedStyle(document.documentElement).getPropertyValue('--accent').trim();
        const [r, g, b] = [1, 3, 5].map((k) => parseInt(accent.slice(k, k + 2), 16));
        const data = new Uint8ClampedArray(width * height * 4);
        for (let i = 0, k = 0; i < height; i++) {
            for (let j = 0; j < width; j++, k += 4) {
                if (!m[i][j]) continue;
                data[k] = r;
                data[k + 1] = g;
                data[k + 2] = b;
                data[k + 3] = 255;
            }
        }

        const off = document.createElement('canvas');
        off.width = width;
        off.height = height;
        off.getContext('2d').putImageData(new ImageData(data, width, height), 0, 0);

        const canvas = document.createElement('canvas');
        canvas.width = width * multiplier;
        canvas.height = height * multiplier;
        if (!hasFilter) canvas.classList.add('no-filter');
        const ctx = canvas.getContext('2d');
        ctx.filter = `blur(${BLUR}px)`;
        ctx.drawImage(off, 0, 0, width, height, 0, 0, canvas.width, canvas.height);

        const gradient = ctx.createLinearGradient(0, 0, 0, canvas.height);
        gradient.addColorStop(0, 'rgba(0, 0, 0, 0.7)');
        gradient.addColorStop(0.7, 'rgba(0, 0, 0, 0.5)');
        gradient.addColorStop(1, 'rgba(0, 0, 0, 0.3)');
        ctx.globalCompositeOperation = 'destination-in';
        ctx.fillStyle = gradient;
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        return canvas;
    }

    function show(seed) {
        currentSeed = seed;
        const canvas = render(seed);
        const previous = current;
        current = canvas;
        root.appendChild(canvas);
        const timing = { duration: 1000, easing: 'cubic-bezier(.2,.7,.46,1.01)', fill: 'forwards' };
        canvas.animate([{ opacity: 0 }, { opacity: 1 }], timing);
        if (previous) {
            previous.animate([{ opacity: 1 }, { opacity: 0 }], timing).finished.then(() => previous.remove());
        }
    }

    // Lo sfondo di una slide come immagine 16:9, per la stampa: già composto
    // sul colore di fondo, con la stessa opacità e lo stesso margine fuori
    // pagina dello sfondo a schermo
    function image(seed) {
        const W = 960;
        const H = 540;
        const canvas = document.createElement('canvas');
        canvas.width = W;
        canvas.height = H;
        const ctx = canvas.getContext('2d');
        ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--background').trim();
        ctx.fillRect(0, 0, W, H);
        ctx.globalAlpha = 0.5;
        ctx.imageSmoothingQuality = 'high';
        ctx.drawImage(render(seed, W / H), -0.15 * W, -0.15 * H, 1.3 * W, 1.3 * H);
        return canvas.toDataURL('image/jpeg', 0.9);
    }

    let resizeTimer;
    window.addEventListener('resize', () => {
        clearTimeout(resizeTimer);
        resizeTimer = setTimeout(() => show(currentSeed), 200);
    });

    return { show, image };
})();
