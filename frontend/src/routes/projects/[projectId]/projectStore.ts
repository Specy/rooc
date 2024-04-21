import {type Project} from "$stores/userProjectsStore";
import {type Pipes, type RoocData, RoocParser, RoocRunnablePipe} from "@specy/rooc";
import {writable} from "svelte/store";


type ProjectStoreData = {
    source: string,
    pipes: Pipes[],
    result?: {
        ok: boolean,
        latex?: string,
        val: RoocData[]
        pipes: Pipes[]
    } | {
        ok: false
        latex?: string
        context: RoocData[]
        error: string
        pipes: Pipes[]
    }
}

export function createCompilerStore(project: Project) {
    const {subscribe, update, set} = writable<ProjectStoreData>({
        source: project.content,
        pipes: project.pipes,
    })

    function run(override?: string) {
        update(s => {
            s.source = override ?? s.source;
            s.result = undefined;
            const pipe = new RoocRunnablePipe(s.pipes)
            const res = pipe.run(s.source)
            const latex = new RoocParser(s.source)
                .compile()
                .map(x => x.toLatex())
                .unwrapOr("")
            if (res.ok) {
                s.result = {
                    ok: true,
                    latex: latex || undefined,
                    val: res.val,
                    pipes: [...s.pipes]
                }
            } else {
                const error = res.val as { context: RoocData[], error: string }
                s.result = {
                    ok: false,
                    latex: latex || undefined,
                    context: error?.context ?? [],
                    error: error?.error ?? "",
                    pipes: [...s.pipes]
                }
            }
            return s;
        })
    }

    function reset() {
        update(s => {
            s.result = undefined;
            return s;
        })

    }

    return {
        subscribe,
        run,
        reset,
        set: (data: ProjectStoreData) => {
            set(data)
        }
    }
}

