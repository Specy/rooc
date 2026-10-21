export const fmt = (n, digits = 0, max = digits) =>
    n.toLocaleString('en-US', { minimumFractionDigits: digits, maximumFractionDigits: max });
export const euro = (n) => `€${fmt(n, 2)}`;

// Istanza del problema della dieta: è la stessa dell'esempio "Diet Problem"
// dell'editor (packages/client/src/components/roocDocs/examples.ts), così
// le slide combaciano con la demo. Valori per porzione.
export const DIET = {
    nutrients: [
        { name: 'Protein', min: 50, max: 150, color: 'var(--protein)' },
        { name: 'Carbohydrates', min: 200, max: 300, color: 'var(--carbs)' },
        { name: 'Fat', min: 0, max: 70, color: 'var(--fats)' },
    ],
    foods: [
        { name: 'Chicken', cost: 1.5, values: [30, 0, 5] },
        { name: 'Rice', cost: 0.5, values: [2, 45, 0] },
        { name: 'Avocado', cost: 2.0, values: [2, 15, 20] },
    ],
};
