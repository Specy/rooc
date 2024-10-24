import {type Project} from "$stores/userProjectsStore.svelte";
import {type RoocData, RoocParser, RoocRunnablePipe} from "@specy/rooc";
import {roocJsStd} from "$lib/Rooc/roocJsStd";

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

export function createCompilerStore(project: Project) {
    let compiling = $state(false)
    let result = $state<RoocResult | undefined>(undefined)

    async function run(override?: string) {
        project.content ??= override
        result = undefined
        try {
            compiling = true
            await new Promise(resolve => setTimeout(resolve, 100))
            const pipe = new RoocRunnablePipe(project.pipes.map(p => p.pipe))
            const res = pipe.run(project.content, roocJsStd())
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
        }
    }
}

