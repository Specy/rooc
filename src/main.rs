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
//how much each product will earn you
max sum((v, i) in enumerate(value)) { x_i * v }
subject to
    sum((time, j) in enumerate(machiningTime[i])){ x_j * time } <= fora for i in len(maxHours)
    //production limit of machine 1
where 
    let value = [10, 15]
    let fora = 10
    let maxHours = [8, 6]
    let machiningTime = [
        [1, 2],
        [2, 1]
    ]
define 
    a, b as NonNegativeReal
    "#
    .to_string();
    let pipe_runner = PipeRunner::new(vec![
        Box::new(CompilerPipe::new()),
        Box::new(PreModelPipe::new()),
        Box::new(ModelPipe::new()),
        Box::new(LinearModelPipe::new()),
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
