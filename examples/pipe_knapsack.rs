use indexmap::IndexMap;
use rooc::pipe::{
    BinarySolverPipe, LinearModelPipe, ModelPipe, PipeContext, PipeRunner, PipeableData,
    PreModelPipe,
};

fn main() {
    let source = "
max sum((value, i) in enumerate(values)) { value * x_i }
s.t.
    sum((weight, i) in enumerate(weights)) { weight * x_i } <= capacity
where
    let weights = [10, 60, 30, 40, 30, 20, 20, 2]
    let values = [1, 10, 15, 40, 60, 90, 100, 15]
    let capacity = 102
define
    x_i as Boolean for i in 0..len(weights)";

    let pipe_runner = PipeRunner::new(vec![
        Box::new(rooc::pipe::CompilerPipe::new()),
        Box::new(PreModelPipe::new()),
        Box::new(ModelPipe::new()),
        Box::new(LinearModelPipe::new()),
        Box::new(BinarySolverPipe::new()),
    ]);
    let result = pipe_runner
        .run(
            PipeableData::String(source.to_string()),
            &PipeContext::new(vec![], &IndexMap::new()),
        )
        .unwrap();
    let last = result
        .into_iter()
        .last()
        .unwrap()
        .to_binary_solution()
        .unwrap();

    println!("{}", last)
}
