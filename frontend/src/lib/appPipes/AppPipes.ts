import type {JsPipableData, LpAssignment, MILPValue, Pipes} from "@specy/rooc";
import type {Highs} from 'highs'
import highsLoader from "highs";
import type {DomainVariable, SerializedLinearModel} from "@specy/rooc/dist/pkg/rooc";

export enum InternalPipe {
    HiGHS = 1000,
    HiGHSCplexLP = 1001,
    ToCplexLP = 1002,
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
    },
    [InternalPipe.ToCplexLP]: {
        loader: async () => {
        },
        fn: toCplexLp
    },
    [InternalPipe.HiGHSCplexLP]: {
        loader: async () => {
        },
        fn: (data) => {
            if (data.type !== 'String') throw new Error('Invalid data type, expected String')
            return solveWithHIGHSUsingCplexLp(data.value, {})
        }
    }
}  satisfies Record<InternalPipe, { loader: () => Promise<void>, fn: (data: JsPipableData) => JsPipableData }>


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
    const domain = data.value.domain
    const cplexLp = linearModelToLp(data.value)
    return solveWithHIGHSUsingCplexLp(cplexLp, domain)
}

function solveWithHIGHSUsingCplexLp(lp: string, domain: Record<string, DomainVariable>): JsPipableData {
    try {
        const solution = highs.solve(lp)
        const value = solution.ObjectiveValue
        const vars = Object.entries(solution.Columns).map(([name, value]) => {
            if (domain[name]?.as_type.type === "Boolean") return {
                name,
                value: {type: "Bool", value: castToBool(value.Primal)}
            }
            if (domain[name]?.as_type.type === "IntegerRange") return {
                name,
                value: {type: "Int", value: castToInt(value.Primal)}
            }
            return {name, value: {type: "Real", value: value.Primal}}
        }) as LpAssignment<MILPValue>[]
        const constraints = Object.fromEntries(solution.Rows.map(value => {
            return [value.Name, value.Primal]
        }))
        return {
            type: "MILPSolution",
            value: {
                assignment: vars,
                constraints,
                value: value
            }
        }
    } catch (e) {
        console.error(e)
        throw new Error(e)
    }

}


function toCplexLp(data: JsPipableData): JsPipableData {
    if (data.type !== 'LinearModel') throw new Error('Invalid data type, expected LinearModel')
    return {
        type: "String",
        value: linearModelToLp(data.value)
    }
}


function linearModelToLp(model: SerializedLinearModel): string {
    let names = {} as Record<string, number>
    const constraints = model.constraints.map(c => {
        const exists = names[c.name] !== undefined
        names[c.name] = (names[c.name] ?? 0) + 1
        const name = exists ? `${c.name}_${names[c.name]}` : c.name
        return `${name}: ${stringifyCoeffs(c.coefficients, model.variables)} ${comparisonMap[c.constraint_type.type]} ${c.rhs}`
    })
    const bounds = model.variables.map(v => {
        const domain = model.domain[v].as_type
        if (domain.type === "IntegerRange" || domain.type === "NonNegativeReal" || domain.type === "Real") return `${stringifyCoeff(domain.value[0])} <= ${v} <= ${stringifyCoeff(domain.value[1])}`
        return undefined
    }).filter(Boolean)
    const binaryVars = model.variables.filter(v => model.domain[v].as_type.type === "Boolean")
    const integerVars = model.variables.filter(v => model.domain[v].as_type.type === "IntegerRange")
    const lpModel = `${model.optimization_type.type === "Max" ? 'Maximize' : 'Minimize'} 
    obj: ${stringifyCoeffs(model.objective, model.variables)} + ${model.objective_offset}
Subject To
    ${constraints.join('\n    ')}
${bounds.length > 0 ? 'Bounds\n    ' : ''}${bounds.join('\n    ')}
${integerVars.length > 0 ? 'General\n    ' : ''}${integerVars.join(' ')}
${binaryVars.length > 0 ? 'Binary\n     ' : ''}${binaryVars.join(' ')}`
    return lpModel.trim() + '\nEnd'
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