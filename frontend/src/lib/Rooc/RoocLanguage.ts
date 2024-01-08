import { RoocParser } from '@specy/rooc'
import type monaco from 'monaco-editor'
import type { MonacoType } from '../Monaco'
import { Position, Range } from 'monaco-editor'
import type { SerializedPrimitiveKind } from '@specy/rooc/dist/pkg/rooc'



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
		provideDocumentFormattingEdits: (model: monaco.editor.ITextModel) => {
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
		console.log(type)
		return `(${type.value.map(getFormattedType).join(', ')})`
	} else if (type.type === "Iterable"){
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

export function createRoocHoverProvider() {
	return {
		provideHover: (model: monaco.editor.ITextModel, position: monaco.Position) => {
			const text = model.getValue()
			const word = model.getWordAtPosition(position)
			const pos = new Position(position.lineNumber, word?.startColumn ?? position.column)
			const offset = model.getOffsetAt(pos)
			const parser = new RoocParser(text)
			const parsed = parser.compile()
			if (!parsed.ok) return
			const items = parsed.val.createTypeMap()
			const item = items.get?.(offset)
			const range = new Range(pos.lineNumber, pos.column, pos.lineNumber, pos.column + (word!.word.length ?? 0))
			if (item) {
				return {
					range,
					contents: [
						{ value: getFormattedType(item.value) }
					]
				}
			} else {
				return {
					range,
					contents: [
						{ value:  keywords[word?.word ?? ''] ?? 'No type found' }
					]
				}
			}
		}
	}
}

export function createRoocCompletion(monaco: MonacoType) {
	return {
		provideCompletionItems: (model: monaco.editor.ITextModel, position: monaco.Position) => {
		}
	}
}