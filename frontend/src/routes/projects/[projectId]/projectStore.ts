import {type Project, type ProjectPipe} from "$stores/userProjectsStore";
import {type RoocData, RoocParser, RoocRunnablePipe} from "@specy/rooc";
import {get, writable} from "svelte/store";


type ProjectStoreData = {
    source: string,
    pipes: ProjectPipe[],
}
type RoocResult = {
    ok: boolean,
    latex?: string,
    val: RoocData[]
} | {
    ok: false
    latex?: string
    context: RoocData[]
    error: string
}

export function createCompilerStore(project: Project) {
    const {subscribe: sourceSubscribe, set: sourceSet} = writable<ProjectStoreData>({
        source: project.content,
        pipes: project.pipes,
    })
    const {
        subscribe: resultSubscribe,
        set: resultSet
    } = writable<RoocResult | undefined>(undefined)


    function run(override?: string) {
        const s = get({subscribe: sourceSubscribe})
        s.source = override ?? s.source;
        resultSet(undefined)
        try {
            const pipe = new RoocRunnablePipe(s.pipes.map(p => p.pipe))
            const res = pipe.run(s.source)
            const latex = new RoocParser(s.source)
                .compile()
                .map(x => x.toLatex())
                .unwrapOr("")
            if (res.ok) {
                resultSet({
                    ok: true,
                    latex: latex || undefined,
                    val: res.val,
                })
            } else {
                const error = res.val as { context: RoocData[], error: string }
                resultSet({
                    ok: false,
                    latex: latex || undefined,
                    context: error?.context ?? [],
                    error: error?.error ?? "",
                })
            }
        } catch (e) {
            resultSet({
                ok: false,
                error: e.toString(),
                context: []
            })
        }
        sourceSet(s)
    }

    function reset() {
        resultSet(undefined)
    }

    return {
        rooc: {
            reset,
            run,
        },
        source: {
            subscribe: sourceSubscribe,
            set: (data: ProjectStoreData) => {
                sourceSet(data)
            }
        },
        result: {
            subscribe: resultSubscribe,

        }
    }
}

