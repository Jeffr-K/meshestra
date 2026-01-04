use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum SagaError {
    #[error("Saga execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Saga compensation failed: {0}")]
    CompensationFailed(String),
}

/// Represents a single step in a Saga
#[async_trait]
pub trait SagaStep<Context>: Send + Sync {
    /// Execute the step logic
    async fn execute(&self, context: &mut Context) -> Result<(), SagaError>;

    /// Compensate (rollback) the step if subsequent steps fail
    async fn compensate(&self, context: &mut Context) -> Result<(), SagaError>;

    /// Name of the step for logging
    fn name(&self) -> &str;
}

/// Orchestrates the execution of a Saga
pub struct SagaOrchestrator<Context> {
    steps: Vec<Box<dyn SagaStep<Context>>>,
}

impl<Context> SagaOrchestrator<Context>
where
    Context: Send + 'static,
{
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add_step<S: SagaStep<Context> + 'static>(mut self, step: S) -> Self {
        self.steps.push(Box::new(step));
        self
    }

    pub async fn execute(&self, mut context: Context) -> Result<Context, SagaError> {
        let mut executed_steps = Vec::new();

        for (index, step) in self.steps.iter().enumerate() {
            match step.execute(&mut context).await {
                Ok(_) => {
                    executed_steps.push(index);
                }
                Err(e) => {
                    // Start compensation in reverse order
                    eprintln!("Step {} failed: {}. Starting compensation.", step.name(), e);

                    for &executed_index in executed_steps.iter().rev() {
                        let executed_step = &self.steps[executed_index];
                        if let Err(comp_err) = executed_step.compensate(&mut context).await {
                            eprintln!(
                                "Compensation failed for step {}: {}",
                                executed_step.name(),
                                comp_err
                            );
                            return Err(SagaError::CompensationFailed(comp_err.to_string()));
                        }
                    }
                    return Err(e);
                }
            }
        }

        Ok(context)
    }
}
