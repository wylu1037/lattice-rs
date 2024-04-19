use thiserror::Error;
use crate::human_readable;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("{0}")]
    Messages(String),
    // ethabi parser error
    #[error(transparent)]
    ParseError(#[from] ethabi::Error),
    // errors from human readable lexer
    LexerError(#[from] human_readable::)
}