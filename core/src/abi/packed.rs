use ethabi::Token;
use thiserror::Error;
use Token::*;

/// An error thrown by [`encode_packed`].
#[derive(Debug, Error)]
pub enum EncodePackedError {
    #[error("This token cannot be encoded in pack mode: {0:?}")]
    InvalidToken(Token),

    #[error("FixedBytes token length > 32")]
    InvalidBytesLength,
}

pub fn encode_packed(tokens: &[Token]) -> Result<Vec<u8>, EncodePackedError> {
    // Get vec capacity and find invalid tokens
    let mut max = 0;
    for token in tokens {
        check(token)?;
        max += max_encoded_length(token);
    }

    // Encode the tokens
    let mut bytes = Vec::with_capacity(max);
    for token in tokens {
        encode_token(token, &mut bytes, false);
    }
    Ok(bytes)
}

/// The maximum byte length of the token encoded using packed mode.
fn max_encoded_length(token: &Token) -> usize {
    match token {
        Int(_) | Uint(_) | FixedBytes(_) => 32,
        Address(_) => 20,
        Bool(_) => 1,
        Array(vec) | FixedArray(vec) | Tuple(vec) => {
            vec.iter().map(|token| max_encoded_length(token).max(32)).sum()
        }
        Bytes(b) => b.len(),
        String(s) => s.len()
    }
}

/// Tuples and nested arrays are invalid in packed encoding.
fn check(token: &Token) -> Result<(), EncodePackedError> {
    match token {
        FixedBytes(vec) if vec.len() > 32 => Err(EncodePackedError::InvalidBytesLength),

        Tuple(_) => Err(EncodePackedError::InvalidToken(token.clone())),
        Array(vec) | FixedArray(vec) => {
            for t in vec.iter() {
                if t.is_dynamic() || matches!(t, Array(_)) {
                    return Err(EncodePackedError::InvalidToken(token.clone()));
                }
                check(t)?
            }
            Ok(())
        }

        _ => Ok(())
    }
}

/// Encodes `token` as bytes into `out`.
fn encode_token(token: &Token, out: &mut Vec<u8>, in_array: bool) {
    match token {
        Address(addr) => {
            if in_array {
                // 将地址扩展到32字节
                out.extend_from_slice(&[0; 12]);
            }
            // 地址是20字节
            out.extend_from_slice(&addr.0)
        }
        Int(n) | Uint(n) => {
            let mut buf = [0; 32];
            n.to_big_endian(&mut buf);
            let start = if in_array { 0 } else { 32 - ((n.bits() + 7) / 8) };
            out.extend_from_slice(&buf[start..32]);
        }
        Bool(b) => {
            if in_array {
                out.extend_from_slice(&[0, 31]);
            }
            out.push((*b) as u8);
        }
        FixedBytes(bytes) => {
            out.extend_from_slice(bytes);
            if in_array {
                let mut remaining = vec![0; 32 - bytes.len()];
                out.append(&mut remaining);
            }
        }

        // Encode dynamic types in-place, without their length
        // Bytes 和 String 是动态类型，编码时会编码一个偏移量
        Bytes(bytes) => out.extend_from_slice(bytes),
        String(s) => out.extend_from_slice(s.as_bytes()),
        Array(vec) | FixedArray(vec) => {
            for token in vec {
                encode_token(token, out, true);
            }
        }

        // Should never happen
        token => unreachable!("Uncaught invalid token: {token:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(tokens: &[Token]) -> Vec<u8> {
        encode_packed(tokens).unwrap()
    }

    fn string(s: &str) -> Token {
        String(s.into())
    }

    fn bytes(b: &[u8]) -> Token {
        Bytes(b.into())
    }

    #[test]
    fn encode_bytes0() {
        let expected = b"hello world";
        assert_eq!(encode_packed(&[string("hello world")]).unwrap(), expected);
    }
}