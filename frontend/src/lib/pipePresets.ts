import { Pipes } from "@specy/rooc"

type PipePreset = {
    name: string,
    pipes: Pipes[]
}

function makePipePreset(name: string, pipes: Pipes[]): PipePreset {
    return {name, pipes}
}
export const defaultPipe = [
    Pipes.CompilerPipe,
    Pipes.PreModelPipe,
    Pipes.ModelPipe,
    Pipes.LinearModelPipe,
    Pipes.StandardLinearModelPipe,
    Pipes.TableauPipe,
    Pipes.OptimalTableauPipe
]
export const pipePresets = [
    makePipePreset("Simplex", defaultPipe)
]

export function isPreset(pipes: Pipes[]): PipePreset | null {
    return pipePresets.find(p => {
        return p.pipes.every((pipe, i) => {
            return pipe === pipes[i]
        })
    }) ?? null
}