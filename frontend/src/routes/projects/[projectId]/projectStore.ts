import { type Project } from "$src/stores/projectStore";
import { CompilationError, RoocParser, TransformError } from "@specy/rooc";
import { writable } from "svelte/store";



type ProjectStoreData = {
    source: string,
    compilationError?: string,
    compiled?: string,
    latex?: string
}

export function createCompilerStore(project: Project) {
    const { subscribe, update, set } = writable<ProjectStoreData>({
        source: project.content,
        
    })

    function compile(override?: string) {
        update(s => {
            s.source = override ?? s.source;
            s.compilationError = undefined

            const parser = new RoocParser(s.source);
            const compile = parser.compile();
            if (!compile.ok) {
                s.compilationError = (compile.val as CompilationError).message()
                return s;
            }
            const latex = compile.val.toLatex(); //before transformation because it consumes self
            const transform = compile.val.transform();
            if (!transform.ok) {
                s.compilationError = (transform.val as TransformError).message()
                return s;
            }
            s.compiled = transform.val.stringify();
            s.latex = latex
            return s;
        })
    }
    function typeCheck(override?: string) {
        update(s => {
            try {
                s.source = override ?? s.source;
                s.compilationError = undefined;
                const parser = new RoocParser(s.source);
                const compile = parser.compile();
                if (!compile.ok) {
                    s.compilationError = (compile.val as CompilationError).message()
                    return s;
                }
                const transform = compile.val.typeCheck();
                if (!transform.ok) {
                    s.compilationError = transform.val.message()
                    return s;
                }
            } catch (e) {
                console.error(e);
                s.compilationError = `Internal error: ${e.message}`
            }
            return s;
        })
    }
    return {
        subscribe,
        compile,
        typeCheck,
        set: (data: ProjectStoreData) => {
            set(data)
        }
    }
}

