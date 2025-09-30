use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemImgError {
    #[error("Command failure: {0}")]
    CommandFailure(FailureOutcome),

    #[error("System failure: {0}")]
    SystemFailure(FailureOutcome),
}

#[derive(Debug)]
pub struct FailureOutcome {
    pub source: Box<dyn std::error::Error + Send + Sync>,
    pub context: String,
    pub command_type: String,
}

impl FailureOutcome {
    pub fn new(
        source: Box<dyn std::error::Error + Send + Sync>,
        context: &str,
        command_type: &str,
    ) -> Self {
        Self {
            source,
            context: context.to_string(),
            command_type: command_type.to_string(),
        }
    }
}

impl fmt::Display for FailureOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error while {} {}: {}",
            self.context, self.command_type, self.source
        )
    }
}
