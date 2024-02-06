import { CompilationError, RoocParser, type PossibleCompletionToken, findRoocCompletionTokens, findRoocExactToken } from '@specy/rooc'
import { editor, languages } from 'monaco-editor'
import { MarkerSeverity, Position, Range, type IDisposable } from 'monaco-editor'
import { createRoocFunctionSignature, getFormattedRoocType } from './RoocUtils'


export const RoocLanguage = {
	defaultToken: 'invalid',
	ignoreCase: true,
	tokenPostfix: '.rooc',
	keywords: ["where", "for", "min", "max", "in", "s.t.", "as", "define"],
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
			//TODO not sure why i need to do this
			[/s\.t\./, 'keyword'],
			//once reached the where block, everything else is declarations
			[/where/, 'keyword', '@where_block'],
			{ include: "declarations" },
			[/([a-z$][\w$]*)(?=\(.*\))/, 'function'],
			[/as/, 'keyword', '@as_assertion'],
			[/[a-z$][\w$]*/, {
				"cases": {
					"@keywords": "keyword",
					"@literals": "literal",
					"@default": "identifier"
				}
			}],
			{ include: '@whitespace' },
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
		as_assertion: [
			[/\W+/, ''],
			[/\w*/, 'identifier.type', '@pop'],
		],
		where_block: [
			[/([a-z$][\w$]*)(?=\s=)/, 'identifier.define'],
			{ include: "@common" },
		],

		comment: [
			[/[^\/*]+/, 'comment'],
			[/\/\*/, 'comment', '@push'],    // nested comment
			["\\*/", 'comment', '@pop'],
			[/[\/*]/, 'comment']
		],
		whitespace: [
			[/[ \t\r\n]+/, 'white'],
			[/\/\*/, 'comment', '@comment'],
			[/\/\/.*$/, 'comment'],
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
	'as': 'Assert that the domain of variable is of a certain type',
	'define': 'Define the domain of the variables in your model, all variables must be defined'
}
const domainTypes = {
	'Boolean': 'A boolean value {0,1}',
	'Real': 'A real number ',
	'Integer': 'An integer number',
	'PositiveReal': 'A positive real number',
}


function stringifyRuntimeEntry(entry: PossibleCompletionToken) {
	if (entry.type === "RuntimeBlockScopedFunction") {
		return [
			{ value: createRoocFunctionSignature(entry) },
			{ value: `[BlockScopedFunction] ${entry.name}: ${entry.description}` },
		]
	} else if (entry.type === "RuntimeBlockFunction") {
		return [
			{ value: createRoocFunctionSignature(entry) },
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

			if (pos.lineNumber === 1 && pos.column === 1) {
				if (word?.word === "min" || word?.word === "max") {
					return {
						range,
						contents: [
							{ value: `Objective function` }
						]
					}
				}
			}
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
					const type = domainTypes[word.word]
					const keyword = keywords[word.word]
					contents.push({ value: keyword ?? type ?? 'No type found' })
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

const suggestedKeywords = [
	'where', 'for', 'in', 's.t.', 'as', 'define'
].map(k => ({
	label: k,
	kind: languages.CompletionItemKind.Keyword,
	insertText: k,
	insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
	detail: keywords[k]
}))
const suggestedTypes = [
	"Boolean", "Real", "Integer", "PositiveReal"
].map(k => ({
	label: k,
	kind: languages.CompletionItemKind.Class,
	insertText: k,
	insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
	detail: `Type ${k}`
}))

export function createRoocCompletion() {
	return {
		provideCompletionItems: (model: editor.ITextModel, position: Position) => {
			const word = model.getWordUntilPosition(position)
			const elements = findRoocCompletionTokens(word.word).map(makeRoocCompletionToken).filter(e => !!e) as languages.CompletionItem[]
			const keywords = suggestedKeywords.filter(e => e.label.startsWith(word.word)) as languages.CompletionItem[]
			const types = suggestedTypes.filter(e => e.label.startsWith(word.word)) as languages.CompletionItem[]
			return {
				suggestions: [...elements, ...keywords, ...types]
			}
		}
	} satisfies languages.CompletionItemProvider
}