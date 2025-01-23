import {type JsPipableData, type LpAssignment, type MILPValue, OptimizationType, type Pipes} from "@specy/rooc";
import type {Highs} from 'highs'
import highsLoader from "highs";

export enum InternalPipe {
    HiGHS = 1000
}

export type AppPipe = Pipes | InternalPipe


export type ProjectPipe = {
    pipe: Pipes
    open: boolean
}


let highsPromise: Promise<Highs> | null = null
export const AppPipesMap = {
    [InternalPipe.HiGHS]: {
        loader: async () => {
            highsPromise ??= highsLoader({
                locateFile(file: string): string {
                    return '/wasm/' + file
                }
            })
            highs = await highsPromise
        },
        fn: highsSolver
    }
}


let highs: Highs | null = null

function stringifyCoeffs(coeffs: number[], names: string[]): string {
    let constraint = coeffs.map((c, i) => {
        if (c === 0) return undefined
        return `${c >= 0 ? '+' : ''} ${c} ${names[i]}`
    }).filter(Boolean).join(' ')
    if (constraint.startsWith('+')) return constraint.slice(1).trim()
    return constraint.trim()
}


const comparisonMap = {
    "LessOrEqual": '<=',
    "GreaterOrEqual": '>=',
    "Equal": '=',
    "Less": '<',
    "Greater": '>'

} satisfies Record<string, string>

function stringifyCoeff(c: number): string {
    if (c === -Infinity) return '-infinity'
    if (c === Infinity) return 'infinity'
    return c.toString()
}

export function highsSolver(data: JsPipableData): JsPipableData {
    if (data.type !== 'LinearModel') throw new Error('Invalid data type, expected LinearModel')
    if (!highs) throw new Error('HiGHS not loaded')
    const model = data.value
    const domain = model.domain
    const constraints = model.constraints.map((c, i) => {
        return `c${i}: ${stringifyCoeffs(c.coefficients, model.variables)} ${comparisonMap[c.constraint_type.type]} ${c.rhs}`
    })
    const bounds = model.variables.map(v => {
        const domain = model.domain[v].as_type
        if (domain.type === "IntegerRange" || domain.type === "NonNegativeReal" || domain.type === "Real") return `${stringifyCoeff(domain.value[0])} <= ${v} <= ${stringifyCoeff(domain.value[1])}`
        return undefined
    }).filter(Boolean)
    const binaryVars = model.variables.filter(v => model.domain[v].as_type.type === "Boolean")
    const integerVars = model.variables.filter(v => model.domain[v].as_type.type === "IntegerRange")
    const cplexLp = `${model.optimization_type.type === "Max" ? 'Maximize' : 'Minimize'} 
obj:
    ${stringifyCoeffs(model.objective, model.variables)} + ${model.objective_offset}
Subject To
    ${constraints.join('\n    ')}
${bounds.length > 0 ? 'Bounds\n    ' : ''}${bounds.join('\n    ')}
${integerVars.length > 0 ? 'General\n    ' : ''}${integerVars.join(' ')}
${binaryVars.length > 0 ? 'Binary\n     ' : ''}${binaryVars.join(' ')}
End`
    const solution = highs.solve(cplexLp)
    const value = solution.ObjectiveValue
    const vars = Object.entries(solution.Columns).map(([name, value]) => {
        if (domain[name].as_type.type === "Boolean") return {name, value: {type: "Bool", value: castToBool(value.Primal)}}
        if (domain[name].as_type.type === "IntegerRange") return {name, value: {type: "Int", value: castToInt(value.Primal)}}
        return {name, value: {type: "Real", value: value.Primal}}
    }) as LpAssignment<MILPValue>[]
    return {
        type: "MILPSolution",
        value: {
            assignment: vars,
            value: value
        }
    }
}

function castToBool(value: number): boolean {
    const delta = 1e-6
    return value > 0 + delta
}
function castToInt(value: number): number {
    return Math.round(value)
}

export function preloadHighs() {
    AppPipesMap[InternalPipe.HiGHS].loader()
}