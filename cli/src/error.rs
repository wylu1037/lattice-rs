use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Custom error: {0}")]
    Custom(String),
}
