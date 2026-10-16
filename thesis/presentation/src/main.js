import 'katex/dist/katex.min.css';
import './styles/base.css';

import { Deck } from './deck.js';
import { renderIcons } from './icons.js';
import { renderMath } from './math.js';
import './slides.js';

const stage = document.getElementById('stage');
renderIcons(stage);
renderIcons(document.getElementById('nav'));
renderMath(stage);
Deck.start();

// comodo dalla console del browser: Deck.go(3)
window.Deck = Deck;
