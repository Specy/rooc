const operators = ['+', '-', '*', '/']
const comparisons = ['<=', '>=', '=', '<', '>']
export const CplexLPLanguage = {
    keywords: [
        'maximize',
        'minimize',
        'subject to',
        'bounds',
        'general',
        'end',
        'binary',
        'semi-continuous',
        'semi-integer',
        'integer',
        'free',
        'positive',
        'negative',
    ],
    operators: [...operators, ...comparisons],
    tokenizer: {
        root: [
            [/(maximize|minimize|subject to|bounds|general|end|binary|semi-continuous|semi-integer|integer|free|positive|negative)/, {token: 'keyword'}], // Keywords (case-insensitive)
            [/[a-zA-Z_][\w]*:/, {token: 'identifier.type'}], // Name of constraints/objective
            [/[a-zA-Z_][\w]*/, {token: 'identifier'}], // Variable names
            [/[0-9]+(\.[0-9]+)?/, {token: 'number'}], // Numbers
            //operators
            [/[+\-*\/]/, {token: 'operator'}],
            //comparison operators
            [/(<=|>=|=|<|>)/, {token: 'comparison'}],
            [/\/\/.*$/, 'comment'],

        ]
    },
    ignoreCase: true,
}