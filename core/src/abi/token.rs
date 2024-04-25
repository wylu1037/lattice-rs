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

impl<'a> Tokenize for &'a [Token] {
    fn into_tokens(self) -> Vec<Token> {
        let mut tokens = self.to_vec();
        if tokens.len() == 1 {
            flatten_token(tokens.pop().unwrap())
        } else {
            tokens
        }
    }
}

impl<T: Tokenizable> Tokenize for T {
    fn into_tokens(self) -> Vec<Token> {
        flatten_token(self.into_token())
    }
}

impl Tokenize for () {
    fn into_tokens(self) -> Vec<Token> {
        vec![]
    }
}

/// Simplified output type for single value.
pub trait Tokenizable {
    /// Converts a `Token` into expected type.
    fn from_token(token: Token) -> Result<Self, InvalidOutputType>
        where
            Self: Sized;

    /// Converts a specified type back into token.
    fn into_token(self) -> Token;
}

/// todo
macro_rules! impl_tuples {
    ($num:expr, $($ty:ident:$no:tt),+$(,)?) => {

    };
}
/// Helper for flattening non-nested tokens into their inner types;
///
/// e.g. `(A,B,C)`would get tokenized to `Tuple([A,B,C])` when in fact we need `[A,B,C]`.
fn flatten_token(token: Token) -> Vec<Token> {
    /// flatten the tokens if required and there is no nesting
    match token {
        Token::Tuple(inner) => inner,
        token => vec![token]
    }
}