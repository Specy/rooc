import type {Project} from "$stores/userProjectsStore.svelte";
import {
    makeRoocFunction,
    type RoocData,
    type RoocFunction,
    RoocParser,
    RoocRunnablePipe,
    type SerializedPrimitive
} from "@specy/rooc";
import {roocJsStd} from "$lib/Rooc/roocJsStd";
import {runSandboxedCode} from "$lib/sandbox/sandbox";
import {Monaco} from "$lib/Monaco";
import {createDebouncer} from "$cmp/pipe/utils";
import {AppPipesMap} from "$lib/appPipes/AppPipes";

type RoocResult = ({
    ok: boolean,
    latex?: string,
    val: RoocData[]
} | {
    ok: false
    latex?: string
    context: RoocData[]
    error: string
})
export type UserDefinedData = {
    functions: RoocFunction[]
    constants: Record<string, SerializedPrimitive>
}

const debouncer = createDebouncer()

export function createCompilerStore(project: Project) {
    let compiling = $state(false)
    let result = $state<RoocResult | undefined>(undefined)
    let loadingUserFunctions = Promise.resolve<UserDefinedData>({functions: [], constants: {}})
    let userDefinedData = $state.raw<UserDefinedData>({functions: [], constants: {}})

    $effect(() => {
        compileUserFunctions($state.snapshot(project.runtime), $state.snapshot(project.files))
    })

    async function compileUserFunctions(code: string, files: string[]) {
        debouncer(async () => {
            // eslint-disable-next-line no-async-promise-executor
            loadingUserFunctions = new Promise<UserDefinedData>(async (res) => {
                try {
                    const jsCode = await Monaco.typescriptToJavascript(code)
                    const fns = await getRuntimeFns(jsCode, files)
                    res(fns)
                } catch (e) {
                    console.error(e)
                    res({functions: [], constants: {}})
                }

            })
            userDefinedData = await loadingUserFunctions
        }, 1000)

    }

    async function run(override?: string) {
        project.content ??= override
        result = undefined
        //wait for user functions to load in case they are being compiled
        await loadingUserFunctions
        try {
            compiling = true
            await new Promise(resolve => setTimeout(resolve, 100))
            const pipe = new RoocRunnablePipe()
            for(const p of project.pipes){
                if(p.pipe < 1000){
                    pipe.addPipeByName(p.pipe)
                }else{
                    const internalPipe = AppPipesMap[p.pipe]
                    await internalPipe.loader()
                    pipe.addPipe(internalPipe.fn)
                }
            }
            const res = pipe.run(project.content, toConstantEntries(userDefinedData.constants), [
                ...roocJsStd(),
                ...userDefinedData.functions
            ])
            const latex = new RoocParser(project.content)
                .compile()
                .map(x => x.toLatex())
                .unwrapOr("")
            if (res.isOk()) {
                result = {
                    ok: true,
                    latex: latex || undefined,
                    val: res.value,
                }
            } else {
                const error = res.error as { context: RoocData[], error: string }
                result = {
                    ok: false,
                    latex: latex || undefined,
                    context: error?.context ?? [],
                    error: error?.error ?? "",
                }
            }
        } catch (e) {
            console.error(e)
            result = {
                ok: false,
                error: e.toString(),
                context: []
            }
        }
        compiling = false
    }

    function reset() {
        result = undefined
        compiling = false
    }

    return {
        reset,
        run,
        get result() {
            return result
        },
        get compiling() {
            return compiling
        },
        get userDefinedFunctions() {
            return userDefinedData
        }
    }
}

function toConstantEntries(data: Record<string, SerializedPrimitive>) {
    return [...Object.entries(data)]
}

async function getRuntimeFns(code: string, files: string[]) {
    const functions: RoocFunction[] = []
    let constants = {} as Record<string, SerializedPrimitive>
    await runSandboxedCode(code, {
        GET_FILES: () => files,
        constants(c) {
            constants = c ?? {}
        },
        register(d) {
            try {
                functions.push(makeRoocFunction(d))
            } catch (e) { /* empty */
            }
        }
    })
    return {
        functions,
        constants
    }
}
