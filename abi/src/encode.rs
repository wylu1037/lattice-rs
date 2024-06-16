use std::any::Any;
use std::str::FromStr;
use std::string::String;
use std::string::ToString;

use alloy_dyn_abi::DynSolValue;
use alloy_json_abi::Param;
use alloy_primitives::{Address as SolAddress, B256, I256, U256};
use regex::Regex;

use model::{Error, HexString};
use model::common::Address;

const BOOL_TY: &str = "bool";
const INT_TY: &str = "int";
const UINT_TY: &str = "uint";
const FIXED_BYTES_TY: &str = "";
const ADDRESS_TY: &str = "address";
const FUNCTION_TY: &str = "function";
const BYTES_TY: &str = "bytes";
const STRING_TY: &str = "string";
const ARRAY_TY: &str = "";
const FIXED_ARRAY_TY: &str = "[%d]";
const TUPLE_TY: &str = "tuple";

/// # 转换参数为Rust abi中对应的类型数据
/// ## 入参
/// + `types: Vec<Param>`
/// + `args: Vec<Box<dyn Any>>`
///
/// ## 出参
pub fn convert_arguments(types: Vec<Param>, args: Vec<Box<dyn Any>>) -> Result<Vec<DynSolValue>, Error> {
    if types.len() != args.len() {
        return Err(Error::new(format!("inputs len {} not equals args len {}", types.len(), args.len())));
    }

    let mut converted_args: Vec<DynSolValue> = Vec::new();
    for (i, ty) in types.iter().enumerate() {
        let c = ty.clone().components;
        let param_type = ty.ty.clone();
        let result = convert_argument(param_type.as_str(), c, args.get(i).unwrap());
        match result {
            Err(e) => return Err(e),
            Ok(v) => converted_args.push(v)
        }
    }
    Ok(converted_args)
}

pub fn convert_argument(ty: &str, components: Vec<Param>, arg: &Box<dyn Any>) -> Result<DynSolValue, Error> {
    match ty {
        _ if STRING_TY == ty => {
            let arg = arg.downcast_ref::<&str>();
            return match arg {
                None => Err(Error::new(format!("invalid arg type, {} expected input string value", ty))),
                Some(v) => {
                    Ok(DynSolValue::String((*v).to_string()))
                }
            };
        }
        _ if BOOL_TY == ty => {
            let arg = arg.downcast_ref::<&str>();
            return match arg {
                None => Err(Error::new(format!("invalid arg type, {} expected input string value", ty))),
                Some(v) => {
                    let v = *v;
                    let b: bool = v.to_lowercase().parse().unwrap();
                    Ok(DynSolValue::Bool(b))
                }
            };
        }
        _ if ADDRESS_TY == ty => {
            let arg = arg.downcast_ref::<&str>();
            return match arg {
                None => Err(Error::new(format!("invalid arg type, {} expected input string value", ty))),
                Some(v) => {
                    let addr = Address::new(*v);
                    Ok(DynSolValue::Address(SolAddress::parse_checksummed(addr.to_ethereum_address(), None).expect("invalid address checksum")))
                }
            };
        }
        _ if TUPLE_TY == ty => {
            let arg = arg.downcast_ref::<Vec<Box<dyn Any>>>();
            return match arg {
                None => Err(Error::new(format!("unsupported arg type, {}", ty))),
                Some(v) => {
                    if v.len() != components.len() {
                        return Err(Error::new(format!("{} expected field count is {}, but actual field count is {}", ty, components.len(), v.len())));
                    }
                    let mut converted_arg_vec: Vec<DynSolValue> = Vec::new();
                    for (i, elem) in v.iter().enumerate() {
                        let param_type = components.get(i).unwrap().ty.clone();
                        let converted = convert_argument(param_type.as_str(), vec![], elem).unwrap();
                        converted_arg_vec.push(converted);
                    }
                    Ok(DynSolValue::Tuple(converted_arg_vec))
                }
            };
        }
        _ if is_bytes(ty) => {
            let (_, size) = parse_bytes(ty);
            let arg = arg.downcast_ref::<&str>();
            return match arg {
                None => Err(Error::new(format!("invalid arg type, {} expected input &str value", ty))),
                Some(v) => {
                    let bytes = HexString::new(v).decode();
                    if size > 0 && bytes.len() != size {
                        return Err(Error::new(format!("{} expected length is {}, but actual length is {}", ty, size, bytes.len())));
                    }
                    if size > 0 {
                        Ok(DynSolValue::FixedBytes(B256::from_slice(bytes.as_slice()), size))
                    } else {
                        Ok(DynSolValue::Bytes(bytes))
                    }
                }
            };
        }
        // 兼容数组和固定长度的数组
        _ if is_array(ty) => {
            let (child_ty, size) = parse_array(ty);
            let arg = arg.downcast_ref::<Vec<&str>>();
            return match arg {
                None => Err(Error::new(format!("invalid arg type, {} expected input Vec<&str> value", ty))),
                Some(v) => {
                    if size > 0 && v.len() != size {
                        return Err(Error::new(format!("{} expected length is {}, but actual length is {}", ty, size, v.len())));
                    }
                    let mut converted_arg_vec: Vec<DynSolValue> = Vec::new();
                    for elem in v {
                        let boxed_arg: Box<dyn Any> = Box::new(*elem);
                        let converted = convert_argument(child_ty.as_str(), vec![], &boxed_arg).unwrap();
                        converted_arg_vec.push(converted);
                    }
                    if size > 0 {
                        Ok(DynSolValue::FixedArray(converted_arg_vec))
                    } else {
                        Ok(DynSolValue::Array(converted_arg_vec))
                    }
                }
            };
        }
        _ if is_uint(ty) => {
            let (_, size) = parse_uint(ty);
            if size == 0 {
                return Err(Error::new(format!("unsupported arg type, {}", ty)));
            }
            let arg = arg.downcast_ref::<&str>();
            return match arg {
                None => Err(Error::new(format!("invalid arg type, {} expected input &str value", ty))),
                Some(v) => {
                    let num = U256::from_str(*v).unwrap();
                    Ok(DynSolValue::Uint(num, size))
                }
            };
        }
        _ if is_int(ty) => {
            let (_, size) = parse_int(ty);
            if size == 0 {
                return Err(Error::new(format!("unsupported arg type, {}", ty)));
            }
            let arg = arg.downcast_ref::<&str>();
            return match arg {
                None => Err(Error::new(format!("invalid arg type, {} expected input &str value", ty))),
                Some(v) => {
                    let num = I256::from_str(*v).unwrap();
                    Ok(DynSolValue::Int(num, size))
                }
            };
        }
        _ => Err(Error::new(format!("unsupported arg type, {}", ty)))
    }
}

fn is_bytes(ty: &str) -> bool {
    let regex = Regex::new(r"^(bytes)([1-9]*)$").unwrap();
    regex.is_match(ty)
}

fn parse_bytes(ty: &str) -> (String, usize) {
    let regex = Regex::new(r"^(bytes)([1-9]*)$").unwrap();
    let c = regex.captures(ty).unwrap();
    let ty = c.get(1).unwrap();
    let size = c.get(2).unwrap();
    let size: usize = size.as_str().parse().unwrap_or_else(|_| 0);
    (ty.as_str().clone().to_string(), size)
}

fn is_uint(ty: &str) -> bool {
    let regex = Regex::new(r"^(uint)([1-9]*)$").unwrap();
    regex.is_match(ty)
}

fn parse_uint(ty: &str) -> (String, usize) {
    let regex = Regex::new(r"^(uint)([1-9]*)$").unwrap();
    let c = regex.captures(ty).unwrap();
    let ty = c.get(1).unwrap();
    let size = c.get(2).unwrap();
    let size: usize = size.as_str().parse().unwrap_or_else(|_| 0);
    (ty.as_str().clone().to_string(), size)
}

fn is_int(ty: &str) -> bool {
    let regex = Regex::new(r"^(int)([1-9]*)$").unwrap();
    regex.is_match(ty)
}

fn parse_int(ty: &str) -> (String, usize) {
    let regex = Regex::new(r"^(int)([1-9]*)$").unwrap();
    let c = regex.captures(ty).unwrap();
    let ty = c.get(1).unwrap();
    let size = c.get(2).unwrap();
    let size: usize = size.as_str().parse().unwrap_or_else(|_| 0);
    (ty.as_str().clone().to_string(), size)
}

fn is_array(ty: &str) -> bool {
    let regex = Regex::new(r"^([a-z1-9]+)(\[([1-9]*)])$").unwrap();
    regex.is_match(ty)
}

fn parse_array(ty: &str) -> (String, usize) {
    let regex = Regex::new(r"^([a-z1-9]+)(\[([1-9]*)])$").unwrap();
    let c = regex.captures(ty).unwrap();
    let ty = c.get(1).unwrap();
    let size = c.get(3).unwrap();
    let size: usize = size.as_str().parse().unwrap_or_else(|_| 0);
    (ty.as_str().clone().to_string(), size)
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use alloy_dyn_abi::{DynSolType, DynSolValue, JsonAbiExt};
    use alloy_json_abi::JsonAbi;
    use alloy_primitives::{b256, U256};
    use alloy_primitives::hex;
    use regex::Regex;

    use crate::encode::convert_arguments;

    const LEDGER_ABI: &str = r#"[
      {
        "inputs": [
          {
            "internalType": "uint64",
            "name": "protocolSuite",
            "type": "uint64"
          },
          {
            "internalType": "bytes32[]",
            "name": "data",
            "type": "bytes32[]"
          }
        ],
        "name": "addProtocol",
        "outputs": [
          {
            "internalType": "uint64",
            "name": "protocolUri",
            "type": "uint64"
          }
        ],
        "stateMutability": "nonpayable",
        "type": "function"
      },
      {
        "inputs": [
          {
            "internalType": "uint64",
            "name": "protocolUri",
            "type": "uint64"
          }
        ],
        "name": "getAddress",
        "outputs": [
          {
            "components": [
              {
                "internalType": "address",
                "name": "updater",
                "type": "address"
              },
              {
                "internalType": "bytes32[]",
                "name": "data",
                "type": "bytes32[]"
              }
            ],
            "internalType": "struct credibilidity.Protocol[]",
            "name": "protocol",
            "type": "tuple[]"
          }
        ],
        "stateMutability": "view",
        "type": "function"
      },
      {
        "inputs": [
          {
            "internalType": "uint64",
            "name": "protocolUri",
            "type": "uint64"
          },
          {
            "internalType": "bytes32[]",
            "name": "data",
            "type": "bytes32[]"
          }
        ],
        "name": "updateProtocol",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
      },
      {
        "inputs": [
          {
            "internalType": "string",
            "name": "hash",
            "type": "string"
          },
          {
            "internalType": "address",
            "name": "address",
            "type": "address"
          }
        ],
        "name": "getTraceability",
        "outputs": [
          {
            "components": [
              {
                "internalType": "uint64",
                "name": "number",
                "type": "uint64"
              },
              {
                "internalType": "uint64",
                "name": "protocol",
                "type": "uint64"
              },
              {
                "internalType": "address",
                "name": "updater",
                "type": "address"
              },
              {
                "internalType": "bytes32[]",
                "name": "data",
                "type": "bytes32[]"
              }
            ],
            "internalType": "struct credibilidity.Evidence[]",
            "name": "evi",
            "type": "tuple[]"
          }
        ],
        "stateMutability": "view",
        "type": "function"
      },
      {
        "inputs": [
          {
            "internalType": "string",
            "name": "hash",
            "type": "string"
          },
          {
            "internalType": "address",
            "name": "address",
            "type": "address"
          }
        ],
        "name": "setDataSecret",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
      },
      {
        "inputs": [
          {
            "internalType": "uint64",
            "name": "protocolUri",
            "type": "uint64"
          },
          {
            "internalType": "string",
            "name": "hash",
            "type": "string"
          },
          {
            "internalType": "bytes32[]",
            "name": "data",
            "type": "bytes32[]"
          },
          {
            "internalType": "address",
            "name": "address",
            "type": "address"
          }
        ],
        "name": "writeTraceability",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
      },
      {
        "inputs": [
          {
            "components": [
              {
                "internalType": "uint64",
                "name": "protocolUri",
                "type": "uint64"
              },
              {
                "internalType": "string",
                "name": "hash",
                "type": "string"
              },
              {
                "internalType": "bytes32[]",
                "name": "data",
                "type": "bytes32[]"
              },
              {
                "internalType": "address",
                "name": "address",
                "type": "address"
              }
            ],
            "internalType": "struct Business.batch[]",
            "name": "bt",
            "type": "tuple[]"
          }
        ],
        "name": "writeTraceabilityBatch",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
      }
    ]"#;

    #[test]
    fn test_encode_ledger_add_protocol() {
        let abi: JsonAbi = serde_json::from_str(LEDGER_ABI).unwrap();
        let f = abi.functions.get("addProtocol").unwrap().get(0).unwrap();
        //let b = hex::decode("516482b2880721149f75c9aea3b6a6a700022c78561f6e22fbd0d4f73e5e7432").unwrap();
        //let w = B256::from_slice(b.as_slice());
        let input = [
            DynSolValue::Uint(U256::from(100u64), 64),
            //DynSolValue::Array(vec![DynSolValue::FixedBytes(w, 32)])
            DynSolValue::Array(vec![DynSolValue::FixedBytes(b256!("516482b2880721149f75c9aea3b6a6a700022c78561f6e22fbd0d4f73e5e7432"), 32)])
        ];
        let result = f.abi_encode_input(&input).unwrap();
        let excepted_data = "ef7e9858000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000001516482b2880721149f75c9aea3b6a6a700022c78561f6e22fbd0d4f73e5e7432";
        assert_eq!(excepted_data, hex::encode(result));
    }

    #[test]
    fn test_encode() {
        // parse a type from a string
        // note: eip712 `CustomStruct`s cannot be parsed this way.
        let my_type: DynSolType = "uint16[2][]".parse().unwrap();

        // decode
        let my_data = hex!(
            "0000000000000000000000000000000000000000000000000000000000000020" // offset
            "0000000000000000000000000000000000000000000000000000000000000001" // length
            "0000000000000000000000000000000000000000000000000000000000000002" // .[0][0]
            "0000000000000000000000000000000000000000000000000000000000000003" // .[0][1]
        );
        let decoded = my_type.abi_decode(&my_data).unwrap();

        let expected = DynSolValue::Array(vec![DynSolValue::FixedArray(vec![2u16.into(), 3u16.into()])]);
        assert_eq!(decoded, expected);

        // round trip
        let encoded = decoded.abi_encode();
        assert_eq!(encoded, my_data);
    }

    #[test]
    fn test_encode_arguments() {
        let abi: JsonAbi = serde_json::from_str(LEDGER_ABI).unwrap();
        let f = abi.functions.get("addProtocol").unwrap().get(0).unwrap();
        let args = convert_arguments(f.inputs.clone(), vec![Box::new("100"), Box::new(vec!["0x516482b2880721149f75c9aea3b6a6a700022c78561f6e22fbd0d4f73e5e7432"])]).unwrap();
        let data = f.abi_encode_input(args.as_slice()).unwrap();
        let excepted_data = "ef7e9858000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000001516482b2880721149f75c9aea3b6a6a700022c78561f6e22fbd0d4f73e5e7432";
        assert_eq!(excepted_data, hex::encode(data));
    }

    #[test]
    fn test_encode_tuple_arguments() {
        let abi: JsonAbi = serde_json::from_str("[{\"inputs\":[],\"name\":\"getUser\",\"outputs\":[{\"components\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"},{\"internalType\":\"string\",\"name\":\"name\",\"type\":\"string\"},{\"internalType\":\"bool\",\"name\":\"isMan\",\"type\":\"bool\"},{\"internalType\":\"string[]\",\"name\":\"tags\",\"type\":\"string[]\"},{\"internalType\":\"uint32[]\",\"name\":\"levels\",\"type\":\"uint32[]\"}],\"internalType\":\"struct Test.User\",\"name\":\"\",\"type\":\"tuple\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"components\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"},{\"internalType\":\"string\",\"name\":\"name\",\"type\":\"string\"},{\"internalType\":\"bool\",\"name\":\"isMan\",\"type\":\"bool\"},{\"internalType\":\"string[]\",\"name\":\"tags\",\"type\":\"string[]\"},{\"internalType\":\"uint32[]\",\"name\":\"levels\",\"type\":\"uint32[]\"}],\"internalType\":\"struct Test.User\",\"name\":\"newUser\",\"type\":\"tuple\"}],\"name\":\"setUser\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint32[]\",\"name\":\"newLevels\",\"type\":\"uint32[]\"}],\"name\":\"updateLevels\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"newName\",\"type\":\"string\"}],\"name\":\"updateName\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"string[]\",\"name\":\"newTags\",\"type\":\"string[]\"}],\"name\":\"updateTags\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]").unwrap();
        let f = abi.functions.get("setUser").unwrap().get(0).unwrap();
        let args: Vec<Box<dyn Any>> = vec![
            Box::new("100"),
            Box::new("Jack"),
            Box::new("true"),
            Box::new(vec!["man", "good"]),
            Box::new(vec!["1", "2", "3"]),
        ];
        let args: Vec<Box<dyn Any>> = vec![Box::new(args)];
        let args = convert_arguments(f.inputs.clone(), args).unwrap();
        let data = f.abi_encode_input(args.as_slice()).unwrap();
        let excepted_data = "66e334840000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000000000000044a61636b0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000036d616e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004676f6f64000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003";
        assert_eq!(excepted_data, hex::encode(data));
    }

    #[test]
    fn test_regex_match_fixed_bytes() {
        let fixed_bytes_sample = vec![
            "bytes1",
            "bytes2[]",
            "bytes3[]",
        ];
        let regex = Regex::new(r"^(bytes[1-9]+)(\[])$").unwrap();
        for s in fixed_bytes_sample {
            let m = regex.is_match(s);
            if m {
                let c = regex.captures(s).unwrap();
                let ty = c.get(1).unwrap();
                let size = c.get(2).unwrap();
                println!("ty {:?}, size {:?}", ty.as_str(), size.as_str())
            }
        }
    }
}