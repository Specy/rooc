import { RoocParser } from '@specy/rooc'
import type monaco from 'monaco-editor'
import type { MonacoType } from '../Monaco'



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

export function createHoverProvider() {
	return {
		provideHover: (model: monaco.editor.ITextModel, position: monaco.Position) => {
			const text = model.getValue()
		}
	}
}

export function createRoocCompletion(monaco: MonacoType) {
	return {
		provideCompletionItems: (model: monaco.editor.ITextModel, position: monaco.Position) => {
		}
	}
}