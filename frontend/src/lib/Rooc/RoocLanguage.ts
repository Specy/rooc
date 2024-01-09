import { CompilationError, RoocParser, type PossibleCompletionToken, findCompletion } from '@specy/rooc'
import { editor, languages } from 'monaco-editor'
import { MarkerSeverity, Position, Range, type IDisposable } from 'monaco-editor'
import type { SerializedPrimitiveKind } from '@specy/rooc/dist/pkg/rooc'
import { findExact } from '@specy/rooc/'


export const RoocLanguage = {
	defaultToken: 'invalid',
	ignoreCase: true,
	tokenPostfix: '.rooc',
	keywords: ["for", "min", "max", "true", "false", "in", "s.t."],
	operators: ["+", "-", "/", "*", "!", "&", "|", "=", "<=", "=>"],
	symbols: /[=><!~?:&|+\-*\/\^%]+/,
	escapes: /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,
	digits: /\d+(_+\d+)*/,
	tokenizer: {
		root: [
			[/where/, 'keyword', '@where_declaration'],
			{ include: '@common' },
		],
		digits_matcher: [
			[/(@digits)[eE]([\-+]?(@digits))?[fFdD]?/, 'number.float'],
			[/(@digits)\.(@digits)([eE][\-+]?(@digits))?[fFdD]?/, 'number.float'],
			[/(@digits)[fFdD]/, 'number.float'],
			[/(@digits)[lL]?/, 'number'],
		],
		common: [
			[/[ \t\r\n]+/, ''],
			//TODO not sure why i need to do this
			[/s\.t\./, 'keyword'],
			[/[a-z$][\w$]*/, {
				"cases": {
					"@keywords": "keyword",
					"@default": "identifier"
				}
			}],
			[/_/, "identifier.ignore"],
			// regular expressions
			// delimiters
			[/[{}]/, "expansion.brackets"],
			[/[()\[\]]/, '@brackets'],
			[/[<>](?!@symbols)/, '@brackets'],
			[/@symbols/, {
				cases: {
					'@operators': 'delimiter',
					'@default': ''
				}
			}],
			// numbers
			{ include: "digits_matcher" },
			// delimiter: after number because of .\d floats
			[/[;,.]/, 'delimiter'],
			// strings:
			[/"([^"\\]|\\.)*$/, 'string.invalid'],  // non-teminated string
			[/"/, 'string', '@string_double'],
		],
		string_double: [
			[/[^\\"]+/, 'string'],
			[/@escapes/, 'string.escape'],
			[/\\./, 'string.escape.invalid'],
			[/"/, 'string', '@pop']
		],
		where_declaration: [
			[/Graph/, 'identifier.class', '@graph_declaration'],
			{ include: '@common' }
		],
		graph_declaration: [
			[/[{]/, '@brackets'],
			[/[:,]/, 'delimiter'],
			[/->/, 'delimiter'],
			[/[a-z$][\w$]*/, 'identifier'],
			{ include: "digits_matcher" },
			[/[[\]]/, '@brackets'],
			[/[}]/, '@brackets', '@pop'],
		],
	}
}



export function createRoocFormatter() {
	return {
		provideDocumentFormattingEdits: (model: editor.ITextModel) => {
			const text = model.getValue()
			const parser = new RoocParser(text)
			const result = parser.format()
			if (!result.ok) {
				console.error("Error formatting", result.err)
				return []
			}
			return [{
				range: model.getFullModelRange(),
				text: result.val
			}]
		}
	}
}

function getFormattedType(type: SerializedPrimitiveKind) {
	if (type.type === 'Tuple') {
		return `(${type.value.map(getFormattedType).join(', ')})`
	} else if (type.type === "Iterable") {
		return `${getFormattedType(type.value)}[]`
	} else {
		return type.type
	}
}

const keywords = {
	'min': 'Minimize the objective function',
	'max': 'Maximize the objective function',
	's.t.': 'Below here, define all the constraints of the problem',
	'where': 'Below here, define all the constants of the problem',
	'for': 'Iterate over one or more ranges to expand the constraint in multiple constraints',
	'in': 'Iterate over a range',

}

function stringifyRuntimeEntry(entry: PossibleCompletionToken) {
	if (entry.type === "RuntimeBlockScopedFunction") {
		return [
			{ value: `${entry.name}(...) { }` },
			{ value: `[BlockScopedFunction] ${entry.name}: ${entry.description}` },
		]
	} else if (entry.type === "RuntimeBlockFunction") {
		return [
			{ value: `${entry.name}{ }` },
			{ value: `[BlockFunction] ${entry.name}: ${entry.description}` },
		]
	} else if (entry.type === "RuntimeFunction") {
		return [
			{ value: `\`\`\`typescript\n${entry.name}(${entry.parameters.map(v => `${v.name}: ${getFormattedType(v.value)}`).join(", ")})\n\`\`\`` },
		]
	}
	return []
}

export function createRoocHoverProvider() {
	return {
		provideHover: (model: editor.ITextModel, position: Position) => {
			const text = model.getValue()
			const word = model.getWordAtPosition(position)
			const pos = new Position(position.lineNumber, word?.startColumn ?? position.column)
			const offset = model.getOffsetAt(pos)
			const exactMatch = findExact(word?.word ?? '')
			const range = new Range(pos.lineNumber, pos.column, pos.lineNumber, pos.column + (word?.word.length ?? 0))

			const contents = []
			if (exactMatch) {
				const match = stringifyRuntimeEntry(exactMatch)
				contents.push(...match)
			}
			const parser = new RoocParser(text)
			const parsed = parser.compile()
			if (!parsed.ok) return
			const items = parsed.val.createTypeMap()
			const item = items.get?.(offset)
			if (item) {
				contents.push({ value: `\`\`\`typescript\n${word?.word ?? "Unknown"}: ${getFormattedType(item.value)}\n\`\`\`` })
			} else {
				contents.push({ value: keywords[word?.word ?? ''] ?? 'No type found' })
			}
			return {
				range,
				contents
			}
		}
	}
}


export function createRoocRuntimeDiagnostics(model: editor.ITextModel) {
	const disposable: IDisposable[] = []
	disposable.push(model.onDidChangeContent(() => {
		const text = model.getValue()
		const parser = new RoocParser(text)
		const parsed = parser.compile()
		const markers = [] as editor.IMarkerData[]
		if (!parsed.ok) {
			const err = parsed.val as CompilationError
			const span = err.getSpan()
			const start = model.getPositionAt(span.start)
			const end = model.getPositionAt(span.start + span.len)
			const message = err.message()
			markers.push({
				startColumn: start.column,
				endColumn: end.column,
				startLineNumber: start.lineNumber,
				endLineNumber: end.lineNumber,
				message,
				severity: MarkerSeverity.Error
			})
		} else {
			const typeCheck = parsed.val.typeCheck()
			if (!typeCheck.ok) {
				const err = typeCheck.val
				const span = err.getOriginSpan()
				const start = model.getPositionAt(span.start)
				const end = model.getPositionAt(span.start + span.len)
				const message = err.stringifyBaseError()
				markers.push({
					startColumn: start.column,
					endColumn: end.column,
					startLineNumber: start.lineNumber,
					endLineNumber: end.lineNumber,
					message,
					severity: MarkerSeverity.Error
				})
			}
		}
		editor.setModelMarkers(model, 'rooc', markers)
	}))
	return {
		dispose() {
			disposable.forEach(d => d.dispose())
		}
	}
}

function makeRoocCompletionToken(entry: PossibleCompletionToken) {
	if (entry.type === "RuntimeBlockScopedFunction") {
		return {
			label: entry.name,
			kind: languages.CompletionItemKind.Function,
			insertText: `${entry.name}() { }`,
			insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
			detail: `[BlockScopedFunction] ${entry.name}: ${entry.description}`,
		}
	} else if (entry.type === "RuntimeBlockFunction") {
		return {
			label: entry.name,
			kind: languages.CompletionItemKind.Function,
			insertText: `${entry.name} { }`,
			insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
			detail: `[BlockFunction] ${entry.name}: ${entry.description}`,
		}
	} else if (entry.type === "RuntimeFunction") {
		const pars = ",".repeat(entry.parameters.length - 1)
		return {
			label: entry.name,
			kind: languages.CompletionItemKind.Function,
			insertText: `${entry.name}(${pars})`,
			insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
			detail: `[Function] ${entry.name}(${entry.parameters.map(v => `${v.name}: ${getFormattedType(v.value)}`).join(", ")})`
		}
	}
	return undefined
}


export function createRoocCompletion() {
	return {
		provideCompletionItems: (model: editor.ITextModel, position: Position) => {
			const word = model.getWordUntilPosition(position)
			const elements = findCompletion(word.word).map(makeRoocCompletionToken).filter(e => !!e)
			return {
				suggestions: elements
			}
		}
	}
}