import { WasmPipeRunner,Pipes} from '../src/pkg/rooc.js'


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