import {browser} from '$app/environment';
import {generateTheme} from '$lib/theme/editorTheme';
//@ts-expect-error - Monaco doesn't have typescript definitions
import type monaco from 'monaco-editor'
import {
    createRoocCompletion,
    createRoocFormatter,
    createRoocHoverProvider,
    createRoocRuntimeDiagnostics,
    makeRoocCompletionToken,
    type RoocCompletionToken,
    roocFunctionToRuntimeFunction,
    RoocLanguage
} from './Rooc/RoocLanguage'
import {getTsGlobal} from "$lib/sandbox/sandboxTsTypes";
import type {RoocFunction, SerializedPrimitiveKind} from "@specy/rooc";
import type {NamedParameter, RuntimeFunction} from "@specy/rooc/src/runtime";

export type MonacoType = typeof monaco

export type RoocFnRef = {
    current: RoocFunction[]
    runtimeFunction: RuntimeFunction<NamedParameter[], SerializedPrimitiveKind>[]
    completionTokenSuggestions: RoocCompletionToken[]
    runDiagnosis?: () => void
}


class MonacoLoader {
    private monaco: MonacoType;
    private roocFnsRef: RoocFnRef = {
        current: [],
        runtimeFunction: [],
        completionTokenSuggestions: []
    }
    loading: Promise<MonacoType>;
    toDispose: monaco.IDisposable[] = [];

    constructor() {
        if (browser) this.load()
    }

    dispose = () => {
        this.toDispose.forEach(d => d.dispose())
    }

    async load(): Promise<MonacoType> {
        if (this.loading) return this.loading
        this.loading = import('monaco-editor')
        const monaco: MonacoType = await this.loading
        monaco.editor.defineTheme('custom-theme', generateTheme())
        monaco.languages.register({id: 'rooc'})
        this.monaco = monaco
        for (const [name, lib] of Object.entries(getTsGlobal())) {
            monaco.languages.typescript.typescriptDefaults.addExtraLib(lib, name)

        }
        monaco.languages.typescript.javascriptDefaults.setCompilerOptions({
            "strict": true,
            "noImplicitAny": true,
            "lib": ["esnext"],
            "strictNullChecks": true,
            "strictFunctionTypes": true,
            "strictPropertyInitialization": true,
            "strictBindCallApply": true,
            "noImplicitThis": true,
            "noImplicitReturns": true,
            "alwaysStrict": true,
            "esModuleInterop": true,
            "declaration": true,
        });

        this.registerLanguages()
        self.MonacoEnvironment = {
            getWorker: async function (_, label) {
                if (label === 'typescript' || label === 'javascript') {
                    const worker = await import('monaco-editor/esm/vs/language/typescript/ts.worker?worker')
                    return new worker.default()
                }
                const worker = await import('monaco-editor/esm/vs/editor/editor.worker?worker')
                return new worker.default()
            }
        }
        return monaco
    }

    setTheme = (theme: string) => {
        this.monaco.editor.setTheme(theme)
    }
    setCustomTheme = (theme: monaco.editor.IStandaloneThemeData) => {
        this.monaco.editor.defineTheme('custom-theme', theme)
        this.monaco.editor.setTheme('custom-theme')
    }

    setRoocFns = (fns: RoocFunction[]) => {
        if (this.roocFnsRef.current === fns) return
        this.roocFnsRef.current = fns
        this.roocFnsRef.runtimeFunction = fns.map(roocFunctionToRuntimeFunction)
        this.roocFnsRef.completionTokenSuggestions = this.roocFnsRef.runtimeFunction.map(makeRoocCompletionToken)
        //refresh all the editors to reload the completion suggestions
        this.roocFnsRef.runDiagnosis?.()
    }

    registerLanguages = () => {
        this.dispose()
        const {monaco} = this
        if (!monaco) return
        //@ts-expect-error - Language works
        this.toDispose.push(monaco.languages.setMonarchTokensProvider('rooc', RoocLanguage))
        this.toDispose.push(monaco.languages.registerDocumentFormattingEditProvider('rooc', createRoocFormatter()))
        this.toDispose.push(monaco.languages.registerHoverProvider('rooc', createRoocHoverProvider(this.roocFnsRef)))
        this.toDispose.push(monaco.languages.registerCompletionItemProvider('rooc', createRoocCompletion(this.roocFnsRef)))
    }

    async typescriptToJavascript(code: string) {
        await this.load()
        const model = this.monaco.editor.createModel(code, 'typescript')
        const worker = await this.monaco.languages.typescript.getTypeScriptWorker()
        return worker(model.uri).then((client) => {
            return client.getEmitOutput(model.uri.toString()).then((result) => {
                model.dispose()
                return result.outputFiles[0].text
            })
        })
    }

    registerRuntimePushers = (language: 'rooc', instance: monaco.editor.ITextModel) => {
        if (language === 'rooc') {
            const disposer = createRoocRuntimeDiagnostics(instance, this.roocFnsRef)
            return () => disposer.dispose()
        }
        return () => {
        }
    }

    async get() {
        if (this.monaco) return this.monaco
        await this.load()
        return this.monaco
    }
}

export const Monaco = new MonacoLoader()