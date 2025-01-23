import type {Pipes} from "@specy/rooc";
import type {PipePreset} from "$lib/pipePresets";

//this is so that i dont import the runtime that uses wasm in the frontend
const defaultPipe = {
    name: "Auto solver",
    pipes: [
        0,
        1,
        2,
        3,
        11
    ]
} satisfies PipePreset


export type Project = {
    version: number,
    id: string,
    name: string,
    description: string,
    createdAt: number,
    updatedAt: number
    content: string
    runtime: string
    runtimeVisible: boolean
    files: string[]
    pipes: ProjectPipe[]
}


const defaultTs = `
/*
const files = GET_FILES()

register({
    name: 'sqrt',
    description: 'Calculate the square root of a number',
    parameters: [['of_num', Primitive.Number]],
    returns: Primitive.Number,
    call: (num) => {
        return {type: "Number", value: Math.sqrt(num.value)}
    }
})

constants({
   PIHalf: { type: "Number", value: Math.PI / 2 }
})
*/
`.trim()



export function createProject(): Project {
    return {
        version: 1,
        id: "",
        runtime: defaultTs,
        runtimeVisible: false,
        name: "Unnamed",
        description: "",
        createdAt: new Date().getTime(),
        updatedAt: new Date().getTime(),
        files: [],
        content:
            `min x
subject to
    /* write the constraints here */
    x >= y
where
    // write the constants here
    let y = 10
define
    // define the model's variables here
    x as NonNegativeReal`
        ,
        pipes: [...defaultPipe.pipes].map((p, i, arr) => ({pipe: p, open: i === (arr.length - 1)}))
    }
}
