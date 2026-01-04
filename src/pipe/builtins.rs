use crate::pipe::{Pipe, PipeError, PipeResult};
use async_trait::async_trait;

/// A pipe that parses a string into an integer
#[derive(Default)]
pub struct ParseIntPipe;

#[async_trait]
impl Pipe for ParseIntPipe {
    type Input = String;
    type Output = i32;

    async fn transform(&self, input: String) -> PipeResult<i32> {
        input.parse::<i32>().map_err(|_| PipeError::Validation("Invalid integer".to_string()))
    }
}
