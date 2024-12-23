import {Pipes} from "@specy/rooc"

export type PipePreset = {
    name: string,
    pipes: Pipes[]
}

function makePipePreset(name: string, pipes: Pipes[]): PipePreset {
    return {name, pipes}
}


export const pipePresets = [
    makePipePreset("Auto solver", [
        Pipes.CompilerPipe,
        Pipes.PreModelPipe,
        Pipes.ModelPipe,
        Pipes.LinearModelPipe,
        Pipes.AutoSolverPipe
    ]),
    makePipePreset("MILP solver", [
        Pipes.CompilerPipe,
        Pipes.PreModelPipe,
        Pipes.ModelPipe,
        Pipes.LinearModelPipe,
        Pipes.MILPSolverPipe
    ]),
    makePipePreset("Real solver", [
        Pipes.CompilerPipe,
        Pipes.PreModelPipe,
        Pipes.ModelPipe,
        Pipes.LinearModelPipe,
        Pipes.RealPipe
    ]),
    makePipePreset("Binary solver", [
        Pipes.CompilerPipe,
        Pipes.PreModelPipe,
        Pipes.ModelPipe,
        Pipes.LinearModelPipe,
        Pipes.BinarySolverPipe,
    ]),
    makePipePreset("Integer & binary solver", [
        Pipes.CompilerPipe,
        Pipes.PreModelPipe,
        Pipes.ModelPipe,
        Pipes.LinearModelPipe,
        Pipes.IntegerBinarySolverPipe
    ]),
    makePipePreset("Simplex solver Step by Step", [
        Pipes.CompilerPipe,
        Pipes.PreModelPipe,
        Pipes.ModelPipe,
        Pipes.LinearModelPipe,
        Pipes.StandardLinearModelPipe,
        Pipes.TableauPipe,
        Pipes.StepByStepSimplexPipe
    ]),
    makePipePreset("To standard form", [
        Pipes.CompilerPipe,
        Pipes.PreModelPipe,
        Pipes.ModelPipe,
        Pipes.LinearModelPipe,
        Pipes.StandardLinearModelPipe,
    ]),

] as const
export const defaultPipe = pipePresets[0].pipes

export function findPreset(pipes: Pipes[]): PipePreset | null {
    return pipePresets.find(p => {
        if (p.pipes.length !== pipes.length) return null
        return p.pipes.every((pipe, i) => {
            return pipe === pipes[i]
        })
    }) ?? null
}