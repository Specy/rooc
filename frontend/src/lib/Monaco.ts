import {browser} from '$app/environment';
import {generateTheme} from '$lib/theme/editorTheme';
import type monaco from 'monaco-editor'
import {
    createRoocCompletion,
    createRoocFormatter,
    createRoocHoverProvider,
    createRoocRuntimeDiagnostics,
    makeRoocCompletionToken,
    type RoocCompletionToken,
    RoocLanguage
} from './Rooc/RoocLanguage'
import {getTsGlobal} from "$lib/sandbox/sandboxTsTypes";
import type {SerializedPrimitiveKind} from "@specy/rooc";
import type {NamedParameter, RuntimeFunction} from "@specy/rooc";
import {getFormattedRoocType, roocFunctionToRuntimeFunction} from "$lib/Rooc/RoocUtils";
import type {UserDefinedData} from "$src/routes/projects/[projectId]/projectStore.svelte";

export type MonacoType = typeof monaco

export type RoocFnRef = {
    current: UserDefinedData
    runtimeFunction: RuntimeFunction<NamedParameter[], SerializedPrimitiveKind>[]
    completionTokenSuggestions: RoocCompletionToken[]
    runDiagnosis?: () => void
}


class MonacoLoader {
    private monaco: MonacoType;
    private roocUserDataRef: RoocFnRef = {
        current: {functions: [], constants: {}},
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
                    //@ts-expect-error - Worker works
                    const worker = await import('monaco-editor/esm/vs/language/typescript/ts.worker?worker')
                    return new worker.default()
                }
                //@ts-expect-error - Worker works
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

    setRoocData = (fns: UserDefinedData) => {
        if (this.roocUserDataRef.current === fns) return
        this.roocUserDataRef.current = fns
        this.roocUserDataRef.runtimeFunction = fns.functions.map(roocFunctionToRuntimeFunction)
        this.roocUserDataRef.completionTokenSuggestions = [
            ...this.roocUserDataRef.runtimeFunction.map(makeRoocCompletionToken),
            ...Object.keys(fns.constants).map(name => ({
                label: name,
                kind: this.monaco.languages.CompletionItemKind.Constant,
                insertText: name,
                // @ts-ignore
                detail: getFormattedRoocType(fns.constants[name])
            } as RoocCompletionToken))
        ]
        //refresh all the editors to reload the completion suggestions
        this.roocUserDataRef.runDiagnosis?.()
    }

    registerLanguages = () => {
        this.dispose()
        const {monaco} = this
        if (!monaco) return
        //@ts-expect-error - Language works
        this.toDispose.push(monaco.languages.setMonarchTokensProvider('rooc', RoocLanguage))
        this.toDispose.push(monaco.languages.registerDocumentFormattingEditProvider('rooc', createRoocFormatter()))
        this.toDispose.push(monaco.languages.registerHoverProvider('rooc', createRoocHoverProvider(this.roocUserDataRef)))
        this.toDispose.push(monaco.languages.registerCompletionItemProvider('rooc', createRoocCompletion(this.roocUserDataRef)))
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
            const disposer = createRoocRuntimeDiagnostics(instance, this.monaco.editor, this.roocUserDataRef)
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