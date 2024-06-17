use std::any::Any;

use alloy_dyn_abi::JsonAbiExt;
use alloy_json_abi::{Function, JsonAbi};
use alloy_primitives::hex;

use model::Error;

use crate::encode::convert_arguments;

pub struct Abi<'a> {
    abi: &'a str,
}

impl<'a> Abi<'a> {
    pub fn parse(&self) -> JsonAbi {
        let abi: JsonAbi = serde_json::from_str(&self.abi).unwrap();
        abi
    }

    pub fn function(&self, function_name: String) -> Result<Function, Error> {
        let abi = self.parse();
        let functions = abi.functions;
        if !functions.contains_key(&function_name) {
            return Err(Error::new(&format!("function {} not found in abi", function_name)));
        }
        let function = functions.get(&function_name).unwrap().get(0).unwrap().clone();
        Ok(function)
    }

    pub fn encode(&self, function_name: String, args: Vec<Box<dyn Any>>) -> String {
        let function = &self.function(function_name).unwrap();
        let args = convert_arguments(function.inputs.clone(), args).unwrap();
        let data_bytes = function.abi_encode_input(args.as_slice()).unwrap();
        format!("0x{}", hex::encode(data_bytes))
    }
}

#[cfg(test)]
mod tests {
    use alloy_dyn_abi::DynSolType;

    use crate::abi::Abi;

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
    fn test_parse_abi() {
        let abi = Abi { abi: LEDGER_ABI }.parse();
        println!("{:?}", abi)
    }

    #[test]
    fn get_function() {
        let abi = Abi { abi: LEDGER_ABI };
        let f = abi.function("addProtocol".to_string()).unwrap();
        let i = f.inputs;
        println!("{}", DynSolType::Uint(256).sol_type_name())
    }

    #[test]
    fn test_encode() {
        let abi = Abi { abi: LEDGER_ABI };
        let data = abi.encode("addProtocol".to_string(), vec![Box::new("100"), Box::new(vec!["0x516482b2880721149f75c9aea3b6a6a700022c78561f6e22fbd0d4f73e5e7432"])]);
        let expected = "0xef7e9858000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000001516482b2880721149f75c9aea3b6a6a700022c78561f6e22fbd0d4f73e5e7432";
        assert_eq!(expected, data);
    }
}