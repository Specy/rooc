// Ogni slide può avere un suo .css e un suo .js in src/slides/. Stanno in un
// modulo a parte perché Vite sposta gli import glob in cima al file: così il
// CSS delle slide arriva dopo base.css e può sovrascriverlo.
import.meta.glob('./slides/*.css', { eager: true });
import.meta.glob('./slides/*.js', { eager: true });
