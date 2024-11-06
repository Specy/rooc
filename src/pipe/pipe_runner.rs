use crate::pipe::pipe_definitions::{PipeError, Pipeable, PipeableData};
use crate::pipe::PipeContext;

/// Pipe runner that runs a sequence of pipes
pub struct PipeRunner {
    pipes: Vec<Box<dyn Pipeable>>,
}

impl PipeRunner {
    pub fn new(pipes: Vec<Box<dyn Pipeable>>) -> PipeRunner {
        PipeRunner { pipes }
    }

    /// Runs the pipe runner
    ///
    /// # Arguments
    /// * `data` - The data to be piped to the first pipe
    /// * `context` - The context that will be available to all pipes
    pub fn run(
        &self,
        data: PipeableData,
        context: &PipeContext,
    ) -> Result<Vec<PipeableData>, (PipeError, Vec<PipeableData>)> {
        if self.pipes.is_empty() {
            return Ok(vec![data]);
        }
        let mut results = vec![data];
        for pipe in &self.pipes {
            let next = results.last_mut().unwrap();
            let result = match pipe.pipe(next, context) {
                Ok(data) => data,
                Err(e) => return Err((e, results)),
            };
            results.push(result);
        }
        Ok(results)
    }
}
