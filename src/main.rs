use rooc::pipe::pipe::PipeableData;
use rooc::pipe::pipe_executors::{
    CompilerPipe, LinearModelPipe, ModelPipe, OptimalTableauPipe, PreModelPipe,
    StandardLinearModelPipe, TableauPipe,
};
use rooc::pipe::pipe_runner::PipeRunner;

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
        min sum(id in 0..len(C)) { v_{vars[id]} * C[id] }
        s.t.
            sum((a, j) in enumerate(A[i])) { v_{vars[j]} * a  } = B[i]  for i in 0..len(A)
        where
            let C = [3, 4, 6]
            let A = [
               [2, 1, 3],
               [1, 3, 4]
            ]
            let B = [2, 1]
            let vars = ["x", "y", "z"]
        define
            v_x  as Real for x in vars
    "#
    .to_string();

    let pipe_runner = PipeRunner::new(vec![
        Box::new(CompilerPipe::new()),
        Box::new(PreModelPipe::new()),
        Box::new(ModelPipe::new()),
        Box::new(LinearModelPipe::new()),
        Box::new(StandardLinearModelPipe::new()),
        Box::new(TableauPipe::new()),
        Box::new(OptimalTableauPipe::new()),
    ]);

    let (result) = pipe_runner.run(PipeableData::String(source));
    match result {
        Ok(data) => {
            let str = data
                .iter()
                .map(|data| format!("//--------{}--------//\n\n{}", data.get_type(), data))
                .collect::<Vec<String>>()
                .join("\n\n");
            println!("{}", str)
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
