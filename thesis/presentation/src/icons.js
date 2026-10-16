/* Icone di Lucide (lucide.dev, licenza ISC). Nell'HTML si scrive
 * <i class="icon" data-icon="cpu"></i> e all'avvio diventa l'SVG. */
const ICONS = {
    'pencil-line':
        '<path d="M12 20h9"/><path d="M16.376 3.622a1 1 0 0 1 3.002 3.002L7.368 18.635a2 2 0 0 1-.855.506l-2.872.838a.5.5 0 0 1-.62-.62l.838-2.872a2 2 0 0 1 .506-.854z"/>',
    database:
        '<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14a9 3 0 0 0 18 0V5"/><path d="M3 12a9 3 0 0 0 18 0"/>',
    'arrow-right-left': '<path d="m16 3 4 4-4 4"/><path d="M20 7H4"/><path d="m8 21-4-4 4-4"/><path d="M4 17h16"/>',
    cpu: '<rect width="16" height="16" x="4" y="4" rx="2"/><rect width="6" height="6" x="9" y="9" rx="1"/><path d="M15 2v2M15 20v2M2 15h2M2 9h2M20 15h2M20 9h2M9 2v2M9 20v2"/>',
    'circle-check': '<circle cx="12" cy="12" r="10"/><path d="m9 12 2 2 4-4"/>',
    'chevron-right': '<path d="m9 18 6-6-6-6"/>',
    'chevron-left': '<path d="m15 18-6-6 6-6"/>',
    plus: '<path d="M5 12h14M12 5v14"/>',
    equal: '<path d="M5 9h14M5 15h14"/>',
};

export function renderIcons(root) {
    for (const el of root.querySelectorAll('[data-icon]')) {
        const paths = ICONS[el.dataset.icon];
        if (!paths) throw new Error(`icona sconosciuta: ${el.dataset.icon}`);
        // gli altri attributi (classi, data-step, style) passano all'SVG
        const attrs = [...el.attributes]
            .filter((a) => a.name !== 'data-icon')
            .map((a) => ` ${a.name}="${a.value.replace(/"/g, '&quot;')}"`)
            .join('');
        el.outerHTML = `<svg${attrs} viewBox="0 0 24 24" aria-hidden="true">${paths}</svg>`;
    }
}
