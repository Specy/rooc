import { Deck } from '../deck.js';

// Passo 4: nessun nuovo elemento, si evidenziano i dati dentro il modello
Deck.define('modello', {
    steps: 4,
    step(slide, n) {
        slide.classList.toggle('show-data', n >= 4);
    },
});
