use indexmap::IndexMap;
#[allow(unused)]
use rooc::pipe::{
    CompilerPipe, LinearModelPipe, ModelPipe, PipeContext, PipeRunner, PipeableData, PreModelPipe,
    RealSolver, StandardLinearModelPipe,
};

#[allow(unused)]
fn main() {
    /*
    for some reason this does not generate a valid basis during solution:
            x + 3y + 4z = 1
            2x + y + 3z = 2
    but this does:
            2x + y + 3z = 2
            x + 3y + 4z = 1
     */
    let source = r#"
    max 0
    s.t.
        1 <= x + a + b + c for (a, b, c) in zip(aa,bb,cc)
    where 
        let aa = [1,2,3]
        let bb = [4,5,6,7]
        let cc = [1,2]
    define
        x as Real
        x_i as Real(0, 5) for i in 0..10
    "#
    .to_string();
    let pipe_runner = PipeRunner::new(vec![
        Box::new(CompilerPipe::new()),
        Box::new(PreModelPipe::new()),
        Box::new(ModelPipe::new()),
        Box::new(LinearModelPipe::new())
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
