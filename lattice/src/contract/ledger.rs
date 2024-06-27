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
