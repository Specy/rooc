import {type AppPipe, InternalPipe} from "$lib/appPipes/AppPipes";
import {pipeDataDescriptions, PipeDataType, type PipeDescription, pipeDescriptions, type Pipes} from "@specy/rooc/runtime";

const internalPipeDescriptions = {
    [InternalPipe.HiGHS]: {
        type: InternalPipe.HiGHS as unknown as Pipes,
        name: "HiGHS solver",
        description: "A high performance MILP solver",
        input: PipeDataType.LinearModel,
        output: PipeDataType.MILPSolution
    }
} satisfies Record<InternalPipe, PipeDescription>

export const PIPE_DESCRIPTIONS = {
    ...pipeDescriptions,
    ...internalPipeDescriptions
}


export function getDescriptionOfPipe(pipe: AppPipe): PipeDescription {
    if (pipe < 1000) {
        return pipeDescriptions[pipe]
    } else {
        return internalPipeDescriptions[pipe]
    }
}

export function getDataOfPipe(pipe: PipeDataType) {
    return pipeDataDescriptions[pipe]
}