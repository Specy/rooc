import {type AppPipe, InternalPipe} from "$lib/appPipes/AppPipes";
import {pipeDataDescriptions, PipeDataType, type PipeDescription, pipeDescriptions} from "@specy/rooc/runtime";

const internalPipeDescriptions = {
    [InternalPipe.HiGHS]: {
        type: InternalPipe.HiGHS,
        name: "HiGHS solver",
        description: "A high performance MILP solver, using a linear model",
        input: PipeDataType.LinearModel,
        output: PipeDataType.MILPSolution
    },
    [InternalPipe.HiGHSCplexLP]: {
        type: InternalPipe.HiGHSCplexLP,
        name: "HiGHS solver (Cplex LP)",
        description: "A high performance MILP solver, using the Cplex LP format",
        input: PipeDataType.String,
        output: PipeDataType.MILPSolution
    },
    [InternalPipe.ToCplexLP]: {
        type: InternalPipe.ToCplexLP,
        name: "To Cplex LP",
        description: "Converts a linear model to the Cplex LP format",
        input: PipeDataType.LinearModel,
        output: PipeDataType.String
    }
} satisfies Record<InternalPipe, PipeDescription>

export const PIPE_DESCRIPTIONS = {
    ...pipeDescriptions,
    ...internalPipeDescriptions
}


export function getDescriptionOfPipe(pipe: AppPipe): PipeDescription {
    return PIPE_DESCRIPTIONS[pipe]
}

export function getDataOfPipe(pipe: PipeDataType) {
    return pipeDataDescriptions[pipe]
}