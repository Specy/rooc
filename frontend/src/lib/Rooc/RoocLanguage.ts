import { CompilationError, RoocParser, type PossibleCompletionToken, findRoocCompletionTokens, findRoocExactToken } from '@specy/rooc'
import { editor, languages } from 'monaco-editor'
import { MarkerSeverity, Position, Range, type IDisposable } from 'monaco-editor'
import type { SerializedPrimitiveKind } from '@specy/rooc/dist/pkg/rooc'
import { createRoocFunctionSignature, getFormattedRoocType } from './RoocUtils'


export const RoocLanguage = {
	defaultToken: 'invalid',
	ignoreCase: true,
	tokenPostfix: '.rooc',
	keywords: ["where", "for", "min", "max", "in", "s.t."],
	literals: ["true", "false"],
	operators: ["+", "-", "/", "*", "!", "&", "|", "=", "<=", "=>"],
	symbols: /[=><!~?:&|+\-*\/\^%]+/,
	digits: /\d+(_+\d+)*/,
	tokenizer: {
		root: [
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
			{ include: "declarations" },
			[/([a-z$][\w$]*)(?=\s=)/, 'identifier.define'],
			[/([a-z$][\w$]*)(?=\(.*\))/, 'function'],
			[/[a-z$][\w$]*/, {
				"cases": {
					"@keywords": "keyword",
					"@literals": "literal",
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
			[/"/, 'string', '@pop']
		],
		declarations: [
			[/Graph/, 'identifier.class', '@graph_declaration'],
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



const keywords = {
	'min': 'Minimize the objective function',
	'max': 'Maximize the objective function',
	's.t.': 'Below here, define all the constraints of the problem',
	'where': 'Below here, define all the variables of the problem',
	'for': 'Iterate over one or more ranges to expand the constraint in multiple constraints',
	'in': 'Iterate over a range',
}

function stringifyRuntimeEntry(entry: PossibleCompletionToken) {
	if (entry.type === "RuntimeBlockScopedFunction") {
		return [
			{ value: createRoocFunctionSignature(entry) },
			{ value: `[BlockScopedFunction] ${entry.name}: ${entry.description}` },
		]
	} else if (entry.type === "RuntimeBlockFunction") {
		return [
			{ value: createRoocFunctionSignature(entry)},
			{ value: `[BlockFunction] ${entry.name}: ${entry.description}` },
		]
	} else if (entry.type === "RuntimeFunction") {
		return [
			{ value: `\`\`\`typescript\n${createRoocFunctionSignature(entry)}\n\`\`\`` },
			{ value: `[Function] ${entry.name}: ${entry.description ?? "Unknown description"}` }
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
			const preciseOffset = model.getOffsetAt(position)
			const exactMatch = findRoocExactToken(word?.word ?? '')
			const range = new Range(pos.lineNumber, pos.column, pos.lineNumber, pos.column + (word?.word.length ?? 0))

			const contents = []
			if (exactMatch) {
				const match = stringifyRuntimeEntry(exactMatch)
				contents.push(...match)
			}
			const parser = new RoocParser(text)
			const parsed = parser.compile()
			if (parsed.ok) {
				const items = parsed.val.createTypeMap()
				const item = items.get?.(preciseOffset) ?? items.get?.(offset)
				if (item) {
					contents.push({ value: `\`\`\`typescript\n${word?.word ?? "Unknown"}: ${getFormattedRoocType(item.value)}\n\`\`\`` })
				} else if (word?.word) {
					contents.push({ value: keywords[word.word] ?? 'No type found' })
				}
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
			detail: `[Function] ${entry.name}(${entry.parameters.map(v => `${v.name}: ${getFormattedRoocType(v.value)}`).join(", ")})`
		}
	}
	return undefined
}


export function createRoocCompletion() {
	return {
		provideCompletionItems: (model: editor.ITextModel, position: Position) => {
			const word = model.getWordUntilPosition(position)
			const elements = findRoocCompletionTokens(word.word).map(makeRoocCompletionToken).filter(e => !!e) as languages.CompletionItem[]
			return {
				suggestions: elements
			}
		}
	} satisfies languages.CompletionItemProvider
}