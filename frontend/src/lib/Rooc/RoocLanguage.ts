/*
problem = {
    SOI ~
    #objective = objective ~ nl+ ~
    ^"s.t." ~ nl+ ~
    #conditions = condition_list ~
    (nl+ ~
    ^"where" ~ nl+ ~
    #where = consts_declaration)? ~
     EOI
}
// required problem body
objective = {
  #objective_type = objective_type ~
  #objective_body = tagged_exp
}
condition_list = { (condition ~ nl+)* ~ condition }
// condition
condition = {
  #lhs = (tagged_exp) ~
  #relation = comparison ~
  #rhs = tagged_exp ~
  (#iteration = for_iteration)?
}
// constants declaration
consts_declaration = { (const_declaration ~ nl+)* ~ const_declaration }
const_declaration  = {
  #name = simple_variable ~
  "=" ~
  #value = primitive
}

// iterations
for_iteration          = _{ ^"for" ~ iteration_declaration_list }
iteration_declaration_list = { (iteration_declaration ~ comma)* ~ iteration_declaration }
iteration_declaration      =  {
  #tuple = (simple_variable | tuple)  ~
  "in" ~
  #iterator = iterator
}
tuple = { "(" ~ (simple_variable | no_par) ~ (comma ~ (simple_variable | no_par))* ~ ")"  }
iterator = { range_iterator | tagged_exp }
//iterators
range_iterator = {
  #from = (tagged_exp) ~
  #range_type = range_type ~
  #to = (tagged_exp)
}

// expressions
test_exp = { SOI ~ exp ~ EOI }
tagged_exp = { exp }
exp         = _{ unary_op? ~ exp_leaf ~ (binary_op ~ unary_op? ~ exp_leaf)* }
exp_leaf    = _{  block_scoped_function | block_function | function | implicit_mul | parenthesis | modulo  | array_access | primitive | variable  }
implicit_mul = { 
    (number | parenthesis | modulo){2,} ~ variable? |
  (number | parenthesis | modulo) ~ variable
}
modulo      =  { "|" ~ exp ~ "|" }
parenthesis =  { "(" ~ exp ~ ")" }
function = {  #function_name = function_name ~ "(" ~ #function_pars = function_pars ~ ")"}
function_pars = { (tagged_exp ~(comma ~ tagged_exp)*)?}
//currently the only non ambiguous implied multiplications
//implied_mul = { primitive ~ variable | primitive ~ parenthesis}
// block functions
block_function = {
    #name = function_name ~
    "{" ~ nl* ~ #body = comma_separated_exp ~ nl* ~"}"
}	
block_scoped_function = {
    #name = function_name ~ 
    "(" ~ nl* ~ #range = iteration_declaration_list ~ nl* ~")" ~ 
    "{" ~ nl* ~ #body = tagged_exp ~ nl* ~ "}"
}
// pointer access var[i][j] or var[0] etc...
array_access        = {
  #name = simple_variable ~
  #accesses = pointer_access_list
}
pointer_access_list = { (pointer_access)+ }
pointer_access      = _{ ^"[" ~ tagged_exp ~ ^"]" }
// constants
primitive = { _primitive }
_primitive = _{ number | array | graph | boolean | string }
graph = { ^"Graph" ~ "{" ~nl* ~ #body = graph_node_list ~ nl* ~ "}" }
graph_node_list = { graph_node? ~ (comma ~ graph_node)* }
graph_node = { #name = simple_variable ~ ( "->" ~ "[" ~ #edges = edges_list ~ "]")?}
edges_list = {  (edge ~ comma)* ~ edge?}
edge = { #node = simple_variable ~ (":" ~ #cost = signed_number)? }
array    =  { "[" ~ nl* ~ ((_primitive ~ comma)* ~ _primitive) ~ nl* ~ "]" }
// utilities
comma_separated_exp = { (tagged_exp ~ comma)* ~ tagged_exp }
comma = _{ "," ~ nl? }
nl = _{NEWLINE}
variable       = _{ !keyword ~ (compound_variable | simple_variable) }
// terminal characters
objective_type = @{ ^"min" | ^"max" }
comparison     = @{ "<=" | ">=" | "=" }
simple_variable   = @{ LETTER+ ~ (NUMBER)* }
compound_variable = @{ simple_variable ~ ("_" ~ LETTER+)+ }
// maybe i should do ("_" ~ LETTER+)+
number    = @{ '0'..'9'+ ~ ("." ~ '0'..'9'+)? }
signed_number = @{ "-"? ~ number}
binary_op = _{ mul | add | sub | div }
mul = { "*" }
add = { "+" }
sub = { "-" }
div = { "/" }

unary_op  = _{ neg }
neg = { "-" }

string = @ { "\"" ~ LETTER* ~ "\""}
boolean = @{ ^"true" | ^"false" }
function_name = @{ LETTER ~ (LETTER | NUMBER | "_")*}
// ignore whitespace in whole grammar
WHITESPACE = _{ " " | "\t" }
range_type = @{ "..=" | ".." }
no_par = @{ "_" }
*/



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
