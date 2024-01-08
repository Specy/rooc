import { browser } from '$app/environment';
import { generateTheme } from '$lib/theme/editorTheme';
//@ts-expect-error - Monaco doesn't have typescript definitions
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker'
import type monaco from 'monaco-editor'
import {RoocLanguage, createRoocFormatter, createRoocHoverProvider} from './Rooc/RoocLanguage'
export type MonacoType = typeof monaco

class MonacoLoader {
	private monaco: MonacoType;
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
		monaco.languages.register({ id: 'rooc' })
		this.monaco = monaco
		this.registerLanguages()
		self.MonacoEnvironment = {
			getWorker: function () {
				return new editorWorker()
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
	registerLanguages = () => {
		this.dispose()
		const { monaco } = this
		if(!monaco) return
		//@ts-expect-error - Language works
		this.toDispose.push(monaco.languages.setMonarchTokensProvider('rooc', RoocLanguage))
		this.toDispose.push(monaco.languages.registerDocumentFormattingEditProvider('rooc', createRoocFormatter()))
		this.toDispose.push(monaco.languages.registerHoverProvider('rooc', createRoocHoverProvider()))
        /*
		this.toDispose.push(monaco.languages.registerCompletionItemProvider('m68k', createM68KCompletition(monaco)))
        */
	}
	async get() {
		if (this.monaco) return this.monaco
		await this.load()
		return this.monaco
	}
}

export const Monaco = new MonacoLoader()