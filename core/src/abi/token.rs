//! Contract Functions Output types.
//!
//! Adapted from [rust-web3](https://github.com/tomusdrw/rust-web3/blob/master/src/contract/tokens.rs).

use ethabi::Token;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("{0}")]
pub struct InvalidOutputType(pub String);

/// Output type possible to deserialize from Contract ABI
pub trait Detokenize {
    /// Creates a new instance from parsed ABI tokens.
    fn from_tokens(tokens: Vec<Token>) -> Result<Self, InvalidOutputType>
        where Self: Sized;
}

impl Detokenize for () {
    fn from_tokens(tokens: Vec<Token>) -> Result<Self, InvalidOutputType> where Self: Sized {
        Ok(())
    }
}

impl<T: Tokenizable> Detokenize for T {
    fn from_tokens(mut tokens: Vec<Token>) -> Result<Self, InvalidOutputType> where Self: Sized {
        let token = if tokens.len() == 1 { tokens.pop().unwrap() } else { Token::Tuple(tokens) };
        Self::from_token(token)
    }
}

/// Convert type into [`Token`]s.
pub trait Tokenize {
    /// Converts `self` into a `Vec<Token>`.
    fn into_tokens(self) -> Vec<Token>;
}

pub trait Tokenizable {
    /// Converts a `Token` into expected type.
    fn from_token(token: Token) -> Result<Self, InvalidOutputType>
        where Self: Sized;

    /// Converts a specified type back into token.
    fn into_token(self) -> Token;
}