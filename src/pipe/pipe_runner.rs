use crate::pipe::pipe::{Pipeable, PipeableData, PipeError};

pub struct PipeRunner {
    pipes: Vec<Box<dyn Pipeable>>,
}

impl PipeRunner {
    pub fn new(pipes: Vec<Box<dyn Pipeable>>) -> PipeRunner {
        PipeRunner { pipes }
    }

    pub fn run(&self, data: PipeableData) -> Result<Vec<PipeableData>, (PipeError, Vec<PipeableData>)> {
        if self.pipes.is_empty() {
            return Ok(vec![data]);
        }
        let mut results = vec![data];
        for pipe in &self.pipes {
            let next = results.last_mut().unwrap();
            let result = match pipe.pipe(next) {
                Ok(data) => data,
                Err(e) => return Err((e, results)),
            };
            results.push(result);
        }
        Ok(results)
    }
}
