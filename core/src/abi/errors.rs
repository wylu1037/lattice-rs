use thiserror::Error;

use crate::abi::human_readable;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("{0}")]
    Messages(String),
    // ethabi parser error
    #[error(transparent)]
    ParseError(#[from] ethabi::Error),
    // errors from human readable lexer
    #[error(transparent)]
    LexerError(#[from] human_readable::lexer::LexerError),
}

/// ABI codec related errors
#[derive(Error, Debug)]
pub enum AbiError {
    /// Thrown when the ABI decoding fails
    #[error(transparent)]
    DecodingError(#[from] ethabi::Error)
}