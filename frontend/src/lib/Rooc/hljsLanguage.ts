const hljskeywords = ["where", "for", "min", "max", "in", "s.t.", "as", "define"]
const hljsLiterals = ["true", "false"]
const hljsOperators = ["+", "-", "/", "*", "!", "&", "|", "=", "<=", "=>"]
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
			className: 'identifierDefine',
			begin: '\\b[a-z\\d_]+(?=\\s*=)',
			end: '=', 
			excludeEnd: true,
		},
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
		},
		{
			className: 'operator',
			begin: '[\\+\\-\\*\\/\\^\\%\\=\\!\\&\\|\\<\\>]',
		}
	]

}
