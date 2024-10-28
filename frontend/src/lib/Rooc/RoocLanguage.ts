import {
    findRoocCompletionTokens,
    findRoocExactToken,
    type PossibleCompletionToken,
    RoocParser,
} from '@specy/rooc'
import {type editor, type IDisposable, languages, MarkerSeverity, Position, Range} from 'monaco-editor'
import {createRoocFunctionSignature, getFormattedRoocType, roocFunctionToRuntimeFunction} from './RoocUtils'
import {roocJsStd} from "$lib/Rooc/roocJsStd";
import type {RoocFnRef} from "$lib/Monaco";
import {createDebouncer} from "$cmp/pipe/utils";


export const RoocLanguage = {
    defaultToken: 'invalid',
    ignoreCase: true,
    tokenPostfix: '.rooc',
    keywords: ["where", "for", "min", "max", "in", "as", "define", "let", 'solve'],
    literals: ["true", "false"],
    operators: ["+", "-", "/", "*", "!", "&", "|", "=", "<=", "=>", "<", ">"],
    symbols: /[=><!~?:&|+\-*\/\^%]+/,
    digits: /\d+(_+\d+)*/,
    tokenizer: {
        root: [
            {include: '@common'},
        ],
        digits_matcher: [
            [/(@digits)[eE]([\-+]?(@digits))?[fFdD]?/, 'number.float'],
            [/(@digits)\.(@digits)([eE][\-+]?(@digits))?[fFdD]?/, 'number.float'],
            [/(@digits)[fFdD]/, 'number.float'],
            [/(@digits)[lL]?/, 'number'],
        ],
        common: [
            [/s\.t\./, 'keyword'],
            [/subject\s+to\b/, 'keyword'],
            //once reached the where block, everything else is declarations
            [/where/, 'keyword', '@where_block'],
            {include: "declarations"},
            [/([a-z$][\w$]*)(?=\(.*\))/, 'function'],
            [/as/, 'keyword', '@as_assertion'],
            [/[a-z$][\w$]*/, {
                "cases": {
                    "@keywords": "keyword",
                    "@literals": "literal",
                    "@default": "identifier"
                }
            }],
            {include: '@whitespace'},
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
            {include: "digits_matcher"},
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
            {include: "digits_matcher"},
            [/[[\]]/, '@brackets'],
            [/[}]/, '@brackets', '@pop'],
        ],
        as_assertion: [
            [/\W+/, ''],
            [/\w*/, 'identifier.type', '@pop'],
        ],
        where_block: [
            [/([a-z$][\w$]*)(?=\s=)/, 'identifier.define'],
            {include: "@common"},
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
            if (!result.isOk()) {
                console.error("Error formatting", result.error)
                return []
            }
            return [{
                range: model.getFullModelRange(),
                text: result.value
            }]
        }
    }
}


const keywords = {
    'min': 'Minimize the objective function',
    'max': 'Maximize the objective function',
    //cannot get this to work
    's.t.': 'Below here, define all the constraints of the problem',
    'subject to': 'Below here, define all the constraints of the problem',
    's': 'Below here, define all the constraints of the problem',
    't': 'Below here, define all the constraints of the problem',
    'where': 'Below here, define all the variables of the problem',
    'for': 'Iterate over one or more ranges to expand the constraint in multiple constraints',
    'in': 'Iterate over a range',
    'as': 'Assert that the domain of variable is of a certain type',
    'define': 'Define the domain of the variables in your model, all variables must be defined',
    'let': 'Define a variable in the model',
    'solve': 'Define the model as a satisfiability problem and not an optimization problem'
}
const domainTypes = {
    'Boolean': 'A boolean value {0,1}',
    'Real': 'A real number',
    'NonNegativeReal': 'A non negative real number',
    'IntegerRange(0, 10)': 'An integer between min and max',
}


function stringifyRuntimeEntry(entry: PossibleCompletionToken) {
    if (entry.type === "RuntimeBlockScopedFunction") {
        return [
            {value: createRoocFunctionSignature(entry)},
            {value: `[BlockScopedFunction] ${entry.name}: ${entry.description}`},
        ]
    } else if (entry.type === "RuntimeBlockFunction") {
        return [
            {value: createRoocFunctionSignature(entry)},
            {value: `[BlockFunction] ${entry.name}: ${entry.description}`},
        ]
    } else if (entry.type === "RuntimeFunction") {
        return [
            {value: `\`\`\`typescript\n${createRoocFunctionSignature(entry)}\n\`\`\``},
            {value: `[Function] ${entry.name}: ${entry.description ?? "Unknown description"}`}
        ]
    }
    return []
}

function mapIfExists<T, U>(value: T | undefined, fn: (v: T) => U): U | undefined {
    if (value) {
        return fn(value)
    }
    return undefined
}

export function createRoocHoverProvider(ref: RoocFnRef) {
    return {
        provideHover: (model: editor.ITextModel, position: Position) => {
            const text = model.getValue()
            const word = model.getWordAtPosition(position)
            const pos = new Position(position.lineNumber, word?.startColumn ?? position.column)
            const offset = model.getOffsetAt(pos)
            const preciseOffset = model.getOffsetAt(position)
            const exactMatch = findRoocExactToken(word?.word ?? '')
                ?? suggestedRoocStd.find(e => e.name === word?.word)
                ?? ref.runtimeFunction.find(e => e.name === word?.word)
            const range = new Range(pos.lineNumber, pos.column, pos.lineNumber, pos.column + (word?.word.length ?? 0))

            if (pos.lineNumber === 1 && pos.column === 1) {
                if (word?.word === "min" || word?.word === "max") {
                    return {
                        range,
                        contents: [
                            {value: `Objective function`}
                        ]
                    }
                }
                if (word?.word === 'solve') {
                    return {
                        range,
                        contents: [
                            {value: `Solve the model as a satisfiability problem`}
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
            if (parsed.isOk()) {
                const items = parsed.value.createTypeMap([
                    ...roocJsStd(),
                    ...ref.current
                ])
                const item = items.get?.(preciseOffset) ?? items.get?.(offset)
                if (item) {
                    contents.push({value: `\`\`\`typescript\n${word?.word ?? "Unknown"}: ${getFormattedRoocType(item.value)}\n\`\`\``})
                } else if (word?.word && word.word.startsWith('IntegerRange')) {
                    contents.push({value: `An integer between min and max`})
                }
            }
            if (word?.word) {
                const type = domainTypes[word.word]
                const keyword = keywords[word.word]
                if (word.word === 'subject') {
                    contents.push({value: keywords['subject to']})
                }
                if (keyword ?? type) {
                    contents.push({value: keyword ?? type})
                } else if (contents.length === 0) {
                    contents.push({value: `No type found`})
                }
            }

            return {
                range,
                contents
            }
        }
    }
}


export function createRoocRuntimeDiagnostics(model: editor.ITextModel, edit: typeof editor, ref: RoocFnRef) {
    const disposable: IDisposable[] = []
    let disposed = false
    const debounce = createDebouncer()

    function callback() {
        if (disposed) return
        const text = model.getValue()
        const parser = new RoocParser(text)
        const parsed = parser.compile()
        const markers = [] as editor.IMarkerData[]
        if (!parsed.isOk()) {
            const err = parsed.error
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
            const typeCheck = parsed.value.typeCheck([
                ...roocJsStd(),
                ...ref.current
            ])
            if (!typeCheck.isOk()) {
                const err = typeCheck.error
                try {
                    if(err.instance?.constructor?.name === "RuntimeError"){
                        throw String(err.instance)
                    }
                    const span = err.getOriginSpan()
                    const start = model.getPositionAt(span?.start)
                    const end = model.getPositionAt(span?.start + span.len)
                    const message = err.stringifyBaseError()
                    markers.push({
                        startColumn: start.column,
                        endColumn: end.column,
                        startLineNumber: start.lineNumber,
                        endLineNumber: end.lineNumber,
                        message,
                        severity: MarkerSeverity.Error
                    })
                } catch (e) {
                    markers.push({
                        startColumn: 0,
                        endColumn: 0,
                        startLineNumber: 0,
                        endLineNumber: 0,
                        message: e,
                        severity: MarkerSeverity.Error
                    })
                }
            } else {
            }
        }
        edit.setModelMarkers(model, 'rooc', markers)
    }

    disposable.push(model.onDidChangeContent(() => {
        debounce(callback, 250)
    }))
    ref.runDiagnosis = callback
    return {
        dispose() {
            disposed = true
            ref.runDiagnosis = undefined
            disposable.forEach(d => d.dispose())
        }
    }
}

export function makeRoocCompletionToken(entry: PossibleCompletionToken) {
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

export type RoocCompletionToken = ReturnType<typeof makeRoocCompletionToken>

const suggestedKeywords = [
    'where', 'for', 'in', 's.t.', 'as', 'define', 'let', 'solve'
].map(k => ({
    label: k,
    kind: languages.CompletionItemKind.Keyword,
    insertText: k,
    insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
    detail: keywords[k]
}))
const suggestedTypes = [
    "Boolean", "Real", "NonNegativeReal", "IntegerRange(0, 10)"
].map(k => ({
    label: k,
    kind: languages.CompletionItemKind.Class,
    insertText: k,
    insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
    detail: `Type ${k}`
}))



const suggestedRoocStd = roocJsStd().map(roocFunctionToRuntimeFunction)
const suggestedStdCompletionToken = suggestedRoocStd.map(makeRoocCompletionToken)

export function createRoocCompletion(ref: RoocFnRef) {
    return {
        provideCompletionItems: (model: editor.ITextModel, position: Position) => {
            const word = model.getWordUntilPosition(position)
            const elements = findRoocCompletionTokens(word.word).map(makeRoocCompletionToken).filter(e => !!e) as languages.CompletionItem[]
            const keywords = suggestedKeywords.filter(e => e.label.startsWith(word.word)) as languages.CompletionItem[]
            const types = suggestedTypes.filter(e => e.label.startsWith(word.word)) as languages.CompletionItem[]
            const parsed = new RoocParser(model.getValue()).compile()
            const suggestions = [...elements, ...suggestedStdCompletionToken, ...ref.completionTokenSuggestions, ...keywords, ...types,] as languages.CompletionItem[]
            if (parsed.isOk()) {
                const identifiers = [...parsed.value.createTypeMap().values()].filter(e => e.identifier)
                const unique = [...new Set(identifiers.map(e => e.identifier))]
                suggestions.push(...unique.map(e => ({
                    label: e,
                    kind: languages.CompletionItemKind.Variable,
                    insertText: e,
                    insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
                    detail: e
                }) as languages.CompletionItem))
            }
            return {
                suggestions: structuredClone(suggestions)
            }
        }
    } satisfies languages.CompletionItemProvider
}