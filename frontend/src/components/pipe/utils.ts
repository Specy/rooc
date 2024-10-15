import Fraction from 'fraction.js'

export function formatNum(num: number, precision = 4): string {
    const f = new Fraction(num)
    const sim = f.simplify(1/(10**(4 - 1)))
    const [whole, nume, deno] = [sim.toFraction(), sim.n, sim.d]
    //@ts-expect-error bigint comparison
    if(nume > 100n || deno > 100n){
        return String(Number(num.toFixed(precision)))
    }
    return whole
}