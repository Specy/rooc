import { WasmPipeRunner,Pipes} from '../src/pkg/rooc.js'
import {RoocParser} from '../src/index.ts'


const model = `
//maximize the value of the bag
max sum((value, i) in enumerate(values)) { value * x_i }
s.t.
    //make sure that the selected items do not go over the bag's capacity
    sum((weight, i) in enumerate(weights)) { weight * x_i } <= capacity
where
    let weights = [10, 60, 30, 40, 30, 20, 20, 2]
    let values = [1, 10, 15, 40, 60, 90, 100, 15]
    let capacity = 102
define
    x_i as Boolean for i in 0..len(weights) 
`

const pipes = [
    Pipes.CompilerPipe,
    Pipes.PreModelPipe,
    Pipes.ModelPipe,
    Pipes.LinearModelPipe,
    Pipes.MILPSolverPipe
]
const res = WasmPipeRunner.new_wasm(pipes).wasm_run_from_string(model, [], [])

console.log(res[res.length - 1])

function assert(condition: boolean, message: string): asserts condition {
    if (!condition) throw new Error(message)
}

const logicSource = `
min abs { x }
s.t.
    a or b
    b = true
define
    x as Real(-1, 1)
    a, b as Boolean
`
const parser = new RoocParser(logicSource)
const preModel = parser.compile().unwrap().serialize()
const serializedAbs = preModel.objective.rhs
assert(serializedAbs.type === 'BlockFunction', 'abs must serialize as a registered block function')
assert(serializedAbs.value.value.kind.type === 'Abs', 'abs block kind must be serialized')
const serializedOr = preModel.constraints[0].lhs
assert(serializedOr.type === 'BinaryOperation', 'pre-model or must be a binary operation')
assert(serializedOr.value[0].value.type === 'Or', 'serialized pre-model operator must be tagged')
assert(preModel.constraints[0].is_logic_assertion, 'bare constraint must retain its assertion flag')
assert(!preModel.constraints[1].is_logic_assertion, 'explicit comparison must not become an assertion')

const transformed = parser.compileAndTransform().unwrap().serialize()
assert(transformed.objective.rhs.type === 'Abs', 'transformed abs must use the Exp::Abs tag')
assert(transformed.constraints[0].lhs.type === 'Or', 'transformed logic expression must be tagged')
assert(transformed.constraints[0].is_logic_assertion, 'model must retain the assertion flag')
assert(!transformed.constraints[1].is_logic_assertion, 'model must retain explicit comparisons')
