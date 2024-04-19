/// 定义宏
macro_rules! unrecognised {
    ($l:ident,$r:ident,$t:expr) => {
        return Err(LexerError::UnrecognisedToken($l, $r, $t));
    };
}

#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error)]
pub enum LexerError {
    #[error("UnrecognisedToken {0}:{1} `{2}`")]
    UnrecognisedToken(usize, usize, String),
    #[error("Expected token `{2}` at {0}:{1}")]
    ExpectedToken(usize, usize, String),
    #[error("EndOfFileInHex {0}:{1}")]
    EndOfFileInHex(usize, usize),
    #[error("MissingNumber {0}:{1}")]
    MissingNumber(usize, usize),
    #[error("end of file but expected `{0}`")]
    EndOfFileExpectedToken(String),
    #[error("end of file")]
    EndOfFile,
}