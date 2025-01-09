use model::convert::string_to_bytes32_array;
use model::HexString;

use crate::impl_builtin_contract;

pub(crate) const LEDGER_ABI_DEFINITION: &str = r#"[
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

const LEDGER_CONTRACT_ADDRESS: &str = "zltc_QLbz7JHiBTspUvTPzLHy5biDS9mu53mmv";

impl_builtin_contract!(LedgerBuiltinContract, LEDGER_ABI_DEFINITION, LEDGER_CONTRACT_ADDRESS);

impl LedgerBuiltinContract {
    pub fn create_business(&self) -> String {
        HexString::from(&[49u8]).hex_string
    }

    /// # 创建协议
    ///
    /// ## 入参
    /// + `trade_number`: 行业号
    ///
    /// ## 出参
    /// + `String`: encoded code
    pub fn create_protocol(&self, trade_number: u64, proto: &str) -> String {
        self.encode_args("addProtocol", vec![Box::new(trade_number.to_string()), Box::new(string_to_bytes32_array(proto))])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_business() {
        let contract = LedgerBuiltinContract::new();
        let actual = contract.create_business();
        let expected = "0x31";
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_create_protocol() {
        let contract = LedgerBuiltinContract::new();
        let actual = contract.create_protocol(1, "syntax = \"proto3\";\n\nmessage Student {\n\tstring id = 1;\n\tstring name = 2;\n}");
        let expected = "0xef7e985800000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000373796e746178203d202270726f746f33223b0a0a6d6573736167652053747564656e74207b0a09737472696e67206964203d20313b0a09737472696e67206e616d65203d20323b0a7d0000000000000000000000000000000000000000000000";
        assert_eq!(expected, actual);
    }
}