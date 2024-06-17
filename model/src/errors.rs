use std::fmt;
use std::fmt::Formatter;

use thiserror::Error;

/// 定义一个宏来简化自定义错误类型的创建
macro_rules! create_error {
    ($name:ident, $($variant:ident => ($message_en:expr, $message_cn:expr)),*) => {
        #[derive(Debug, Error)]
        pub enum $name {
            $(
                #[error("{}, {}", $message_en, $message_cn)]
                $variant,
            )*
        }

        impl $name {
            pub fn message_en(&self) -> &str {
                match self {
                    $(
                        $name::$variant => $message_en,
                    )*
                }
            }

            pub fn message_cn(&self) -> &str {
                match self {
                    $(
                        $name::$variant => $message_cn,
                    )*
                }
            }
        }
    };
}

// 使用宏来定义自定义错误类型
create_error!(BusinessError,
    NotFound => ("Resource not found", "资源未找到"),
    Unauthorized => ("Unauthorized access", "未经授权的访问"),
    InternalError => ("Internal server error", "服务内部错误")
);

create_error!(LatticeError,
    InternalError => ("Internal error", "内部错误"),
    ReceiptNotFound => ("Receipt not found, contract is not execute or tx is not on-chain", "收据信息不存在，合约未被执行或者交易未被上链")
);

#[derive(Debug)]
pub struct Error {
    code: i32,
    message: String,
}

impl Error {
    pub fn new(message: &str) -> Self {
        Error {
            code: -1,
            message: message.to_string(),
        }
    }

    pub fn custom(code: i32, message: String) -> Self {
        Error {
            code,
            message,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Err code: {}, Err message: {}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::new(err.to_string().as_str())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::new(err.to_string().as_str())
    }
}