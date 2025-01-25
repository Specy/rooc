use indexmap::IndexMap;
#[allow(unused)]
use rooc::pipe::{
    CompilerPipe, LinearModelPipe, ModelPipe, PipeContext, PipeRunner, PipeableData, PreModelPipe,
    RealSolver, StandardLinearModelPipe,
};

#[allow(unused)]
fn main() {
    let source = r#"
min 1
subject to
    c_i: x_i >= 0 for i in 0..10
define
    x_i as NonNegativeReal(0, 100) for i in 0..10
    "#
    .to_string();
    let pipe_runner = PipeRunner::new(vec![
        Box::new(CompilerPipe::new()),
        Box::new(PreModelPipe::new()),
        Box::new(ModelPipe::new()),
        Box::new(LinearModelPipe::new()),
        Box::new(RealSolver::new()),
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
