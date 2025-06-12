use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum Status {
    #[error("cancelled")]
    Cancelled,
    #[error("unknown")]
    Unknown,
    #[error("invalid argument")]
    InvalidArgument,
    #[error("deadline exceeded")]
    DeadlineExceeded,
    #[error("the data `{0}` is not available")]
    NotFound(String),
    #[error("already exists")]
    AlreadyExists,
    #[error("permission denied")]
    PermissionDenied,
    #[error("resource exhausted")]
    ResourceExhausted,
    #[error("failed precondition")]
    FailedPrecondition,
    #[error("aborted")]
    Aborted,
    #[error("out of range")]
    OutOfRange,
    #[error("unimplemented")]
    Unimplemented,
    #[error("internal")]
    Internal,
    #[error("unavailable")]
    Unavailable,
    #[error("data loss")]
    DataLoss,
    #[error("unauthenticated")]
    Unauthenticated,
}

#[allow(dead_code)]
impl Status {
    pub fn code(&self) -> u16 {
        match self {
            Status::Cancelled => 1001,
            Status::Unknown => 1002,
            Status::InvalidArgument => 1003,
            Status::DeadlineExceeded => 1004,
            Status::NotFound(_) => 1005,
            Status::AlreadyExists => 1006,
            Status::PermissionDenied => 1007,
            Status::ResourceExhausted => 1008,
            Status::FailedPrecondition => 1009,
            Status::Aborted => 1010,
            Status::OutOfRange => 1011,
            Status::Unimplemented => 1012,
            Status::Internal => 1013,
            Status::Unavailable => 1014,
            Status::DataLoss => 1015,
            Status::Unauthenticated => 1016,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_code() {
        let status = Status::NotFound("test".to_string());
        assert_eq!(status.code(), 1);
    }

    #[test]
    fn test_status_message() {
        let status = Status::NotFound("test".to_string());
        assert_eq!(status.to_string(), "the data `test` is not available");
    }
}