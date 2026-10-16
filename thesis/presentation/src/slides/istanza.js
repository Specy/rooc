import { compiledTex, instanceTex, modelTex } from '../tex.js';

// Modello, istanza e modello compilato sono generati dai dati, così restano
// coerenti con le altre slide
const slide = document.getElementById('istanza');
slide.querySelector('.model-tex').textContent = modelTex();
slide.querySelector('.instance-tex').textContent = instanceTex();
slide.querySelector('.compiled-tex').textContent = compiledTex();
