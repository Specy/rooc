/* Evidenziazione minimale della sintassi di ROOC, pensata per le slide:
 * le costanti dei dati e le variabili hanno gli stessi colori usati nelle
 * formule (classi data e var). Le variabili ausiliarie introdotte dal
 * compilatore ($abs_0, $su_1, ...) hanno la classe aux. */
const KEYWORDS = new Set([
    'min', 'max', 's.t.', 'where', 'define', 'let', 'for', 'in', 'as', 'sum',
    'abs', 'and', 'or', 'not', 'implies', 'iff', 'xor',
]);
const TOKEN = /(\/\/.*$)|(s\.t\.)|(\$[A-Za-z0-9_]+)|([A-Za-z_][A-Za-z0-9_]*)|(\d+(?:\.\d+)?)|(\s+)|([^\sA-Za-z0-9_]+)/g;

const escape = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
const span = (cls, text) => `<span class="${cls}">${escape(text)}</span>`;

export function highlightRooc(line, constants = new Set(), variables = new Set()) {
    return line.replace(TOKEN, (tok, comment, st, aux, ident, number, space, punct) => {
        if (comment) return span('c-comment', tok);
        if (st) return span('c-kw', tok);
        if (aux) return span('aux', tok);
        if (ident) {
            if (KEYWORDS.has(ident)) return span('c-kw', tok);
            if (constants.has(ident)) return span('data', tok);
            if (variables.has(ident) || /^x_/.test(ident)) return span('var', tok);
            if (/^[A-Z]/.test(ident)) return span('c-type', tok);
            if (/^[a-z]+$/.test(ident) && ident.length > 1) return span('c-fn', tok);
            return escape(tok);
        }
        if (number) return span('c-num', tok);
        if (punct) return span('c-punct', tok);
        return tok;
    });
}
