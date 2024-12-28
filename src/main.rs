use indexmap::IndexMap;
use rooc::pipe::IntegerBinarySolverPipe;
#[allow(unused)]
use rooc::pipe::{
    CompilerPipe, LinearModelPipe, ModelPipe, PipeContext, PipeRunner, PipeableData, PreModelPipe,
    RealSolver, StandardLinearModelPipe,
};

#[allow(unused)]
fn main() {
    let source = r#"
min (4a + 3b + 2ca + 3s + 5m + 6ch) - 50
subject to
    a + b >= 3
    ca + s >= 2
    m + ch >= 1
    4a + 3b + 2ca + 3s + 5m + 6ch >= 50
    ca >= 0
    ch >= 0
define
    a, b, ca, s, m, ch as IntegerRange(0, 100)
    "#
    .to_string();
    let pipe_runner = PipeRunner::new(vec![
        Box::new(CompilerPipe::new()),
        Box::new(PreModelPipe::new()),
        Box::new(ModelPipe::new()),
        Box::new(LinearModelPipe::new()),
        Box::new(IntegerBinarySolverPipe::new()),
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
