import hljs from 'highlight.js/lib/core';

const hljskeywords = ["where", "for", "min", "max", "in", "s.t.", "as", "define", "let", "subject", "to"]
const hljsLiterals = ["true", "false"]
const hljsOperators = ["+", "-", "/", "*", "!", "&", "|", "<=", ">=", "="]
export const highlightJsGrammar = {
    case_insensitive: true,
    keywords: {
        keyword: hljskeywords,
        literal: hljsLiterals,
        built_in: 'Graph',
    },
    contains: [
        {
            'className': 'keyword',
            'begin': 's.t.'
        },
        {
            "begin": 'Graph',
            "end": '}',
            excludeBegin: true,
            contains: [
                {
                    "className": "keyword",
                    "begin": 'Graph',
                },
                {
                    className: "brackets",
                    begin: '[\\{]',
                },
                {
                    className: "brackets",
                    begin: '[\\[\\]]',
                },
                {
                    className: "delimiter",
                    begin: '[\\:\\,]',
                },
                {
                    className: "operator",
                    begin: '->',
                },
                {
                    className: "number",
                    begin: '\\b\\d+(\\.\\d+)?',
                },
                {
                    className: "identifier",
                    begin: '[a-z\\d_]+',
                }

            ]

        },
        //comments
        {
            className: 'comment',
            begin: '//', end: '$',
        },
        {
            className: 'comment',
            begin: '/\\*', end: '\\*/',
        },
        //assignment
        {
            className: 'identifier',
            begin: '[a-z$][\\w$]*',
            keywords: hljskeywords,
        },
        {
            className: "identifierIgnore",
            begin: '_',
        },
        {
            className: "number",
            begin: '\\b\\d+(\\.\\d+)?',
        },
        {
            className: "string",
            begin: '"', end: '"',
        },
        {
            className: "bracketsExpansion",
            begin: '[\\{\\}]',
        },
        {
            className: 'brackets',
            begin: '[\\[\\]()]',
        }, {
            className: 'operator',
            begin: `${hljsOperators.map(i => {
                return `${i.split('').map(j => `\\${j}`).join('')}`
            }).join('|')}`,
        }
    ]

}


export function cplexHighlightJs() {
    return {
        name: 'CPLEX LP',
        case_insensitive: true,
        keywords: {
            keyword: 'minimize maximize subject to st bounds binary general semi-continuous semicontinuous end',
            built_in: 'inf infinity',
        },
        contains: [
            hljs.C_LINE_COMMENT_MODE,
            hljs.C_BLOCK_COMMENT_MODE,
            hljs.NUMBER_MODE,
            {
                className: 'variable',
                begin: /([A-Z0-9]*:)/,
                relevance: 0,
            },
            {
                className: 'operator',
                begin: /<=|>=|=|<|>|\+|-|\*|\/|\^/,
                relevance: 0,
            },
        ]
    };
}