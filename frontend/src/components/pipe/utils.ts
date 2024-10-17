import Fraction from 'fraction.js'


const cache = {
    4: {}
} as Record<number, Record<number, string>>

export function formatNum(num: number, precision = 4): string {
    if (!cache[precision]) cache[precision] = {}
    if (cache[precision][num] !== undefined) return cache[precision][num]
    const f = new Fraction(num)
    const sim = f.simplify(1 / (10 ** (4 - 1)))
    const [whole, nume, deno] = [sim.toFraction(), sim.n, sim.d]
    //@ts-expect-error bigint comparison
    if (nume > 100n || deno > 100n) {
        const val = String(Number(num.toFixed(precision)))
        cache[precision][num] = val
        return val
    }
    cache[precision][num] = whole
    return whole
}