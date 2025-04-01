use indexmap::IndexMap;
#[allow(unused)]
use rooc::pipe::{
    CompilerPipe, LinearModelPipe, ModelPipe, PipeContext, PipeRunner, PipeableData, PreModelPipe,
    RealSolver, StandardLinearModelPipe,
};
use rooc::pipe::{StepByStepSimplexPipe, TableauPipe};

#[allow(unused)]
fn main() {
    let source = r#"

max 2x_1 + x_2 - x_3
subject to
//write the constraints here
5x_1 - 2x_2 + 8x_3 ≤ 15
8x_1+3x_2 -x_3 ≥ 9
x_1+x_2+x_3≤6
where

define
// define the model's variables here
x_1 as Real
x_2 as Real
x_3 as Real
    "#
    .to_string();
    let pipe_runner = PipeRunner::new(vec![
        Box::new(CompilerPipe::new()),
        Box::new(PreModelPipe::new()),
        Box::new(ModelPipe::new()),
        Box::new(LinearModelPipe::new()),
        Box::new(StandardLinearModelPipe::new()),
        Box::new(TableauPipe::new()),
        Box::new(StepByStepSimplexPipe::new())
    ]);

    let (result) = pipe_runner.run(
        PipeableData::String(source),
        &PipeContext::new(vec![], &IndexMap::new()),
    );
    match result {
        Ok(data) => {
            let last = data.last().unwrap();
            let str = data
                .iter()
                .map(|data| format!("//--------{}--------//\n\n{}", data.get_type(), data))
                .collect::<Vec<String>>()
                .join("\n\n");
            println!("{}", last)
        }
        Err((error, context)) => {
            let context = context
                .iter()
                .map(|data| format!("//--------{}--------//\n\n{}", data.get_type(), data))
                .collect::<Vec<String>>()
                .join("\n\n");
            println!("Context:\n{}\n\nError:\n{}", context, error)
        }
    }
}
