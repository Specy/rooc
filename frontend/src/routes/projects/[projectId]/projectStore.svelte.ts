import {type Project} from "$stores/userProjectsStore.svelte";
import {makeRoocFunction, type RoocData, type RoocFunction, RoocParser, RoocRunnablePipe} from "@specy/rooc";
import {roocJsStd} from "$lib/Rooc/roocJsStd";
import {runSandboxedCode} from "$lib/sandbox/sandbox";
import {Monaco} from "$lib/Monaco";
import {createDebouncer} from "$cmp/pipe/utils";

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

const debouncer = createDebouncer()

export function createCompilerStore(project: Project) {
    let compiling = $state(false)
    let result = $state<RoocResult | undefined>(undefined)
    let loadingUserFunctions = Promise.resolve([])
    let userDefinedFunctions = $state.raw<RoocFunction[]>([])

    $effect(() => {
        compileUserFunctions(project.runtime)
    })

    async function compileUserFunctions(code: string) {
        debouncer(async () => {
            // eslint-disable-next-line no-async-promise-executor
            loadingUserFunctions = new Promise<RoocFunction[]>(async (res) => {
                try {
                    const jsCode = await Monaco.typescriptToJavascript(code)
                    const fns = await getRuntimeFns(jsCode)
                    res(fns)
                } catch (e) {
                    console.error(e)
                    res([])
                }

            })
            userDefinedFunctions = await loadingUserFunctions
        }, 500)

    }

    async function run(override?: string) {
        project.content ??= override
        result = undefined
        //wait for user functions to load in case they are being compiled
        await loadingUserFunctions
        try {
            compiling = true
            await new Promise(resolve => setTimeout(resolve, 100))
            const pipe = new RoocRunnablePipe(project.pipes.map(p => p.pipe))
            const res = pipe.run(project.content, [
                ...roocJsStd(),
                ...userDefinedFunctions
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
            return userDefinedFunctions
        }
    }
}


async function getRuntimeFns(code: string) {
    const res: RoocFunction[] = []
    await runSandboxedCode(code, {
        register(d) {
            try {
                res.push(makeRoocFunction(d))
            } catch (e) { /* empty */
            }
        }
    })
    return res
}
