use indexmap::IndexMap;
use serde::Serialize;
use rooc::pipe::{CompilerPipe, PipeRunner, PipeableData, PreModelPipe};
use rooc::primitives::{Graph, GraphEdge, GraphNode};

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
min x
s.t.
    x >= 2
where
    let g = Graph {
        A -> [B:2],
        B
    }
define
    x as Boolean
    
    "#
    .to_string();
    
    let pipe_runner = PipeRunner::new(vec![
        Box::new(CompilerPipe::new()),
        Box::new(PreModelPipe::new()),
    ]);

    let (result) = pipe_runner.run(PipeableData::String(source), &IndexMap::new());
    match result {
        Ok(data) => {
            let last = data.last().unwrap();
            let str = data
                .iter()
                .map(|data| format!("//--------{}--------//\n\n{}", data.get_type(), data))
                .collect::<Vec<String>>()
                .join("\n\n");
            println!("{}", str)
        }
        Err((error, context)) => {
            return;
            let context = context
                .iter()
                .map(|data| format!("//--------{}--------//\n\n{}", data.get_type(), data))
                .collect::<Vec<String>>()
                .join("\n\n");
            println!("Context:\n{}\n\nError:\n{}", context, error)
        }
    }
}
