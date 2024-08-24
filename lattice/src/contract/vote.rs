use crypto::Transaction;

use crate::builder::{CallContractBuilder, TransactionBuilder};
use crate::impl_builtin_contract;

/// 内置的投票合约
const VOTE_ABI_DEFINITION: &str = r#"[
    {
        "inputs": [
            {
                "internalType": "string",
                "name": "ProposalId",
                "type": "string"
            },
            {
                "internalType": "uint8",
                "name": "VoteSuggestion",
                "type": "uint8"
            }
        ],
        "name": "vote",
        "outputs": [
            {
                "internalType": "bytes",
                "name": "",
                "type": "bytes"
            }
        ],
        "stateMutability": "pure",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "string",
                "name": "ProposalId",
                "type": "string"
            }
        ],
        "name": "refresh",
        "outputs": [
            {
                "internalType": "bytes",
                "name": "",
                "type": "bytes"
            }
        ],
        "stateMutability": "pure",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "string",
                "name": "proposalId",
                "type": "string"
            }
        ],
        "name": "cancel",
        "outputs": [
            {
                "internalType": "bytes",
                "name": "",
                "type": "bytes"
            }
        ],
        "stateMutability": "pure",
        "type": "function"
    }
]"#;

const VOTE_CONTRACT_ADDRESS: &str = "zltc_amgWuhifLRUoZc3GSbv9wUUz6YUfTuWy5";


impl_builtin_contract!(VoteBuiltinContract, VOTE_ABI_DEFINITION, VOTE_CONTRACT_ADDRESS);

impl VoteBuiltinContract {
    pub fn new_vote_tx(&self, proposal_id: &str, approve: bool) -> Transaction {
        let approve: String = if approve { String::from("1") } else { String::from("0") };
        let data = self.encode_args("vote", vec![Box::new(proposal_id.to_string()), Box::new(approve)]);

        CallContractBuilder::builder()
            .set_linker(&self.address)
            .set_code(&data)
            .build()
    }
}

#[cfg(test)]
mod test {
    use model::common::Address;
    use model::Curve;

    use crate::lattice::{ChainConfig, ConnectingNodeConfig, CredentialConfig, LatticeClient};

    use super::*;

    #[tokio::test]
    async fn test_new_vote_tx() {
        let owner = "zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi";
        let chain_config = ChainConfig {
            chain_id: 1,
            curve: Curve::Sm2p256v1,
        };
        let connecting_node_config = ConnectingNodeConfig {
            ip: String::from("192.168.1.185"),
            http_port: 13000,
            websocket_port: 13001,
        };
        let credential_config = CredentialConfig {
            sk: String::from("0x23d5b2a2eb0a9c8b86d62cbc3955cfd1fb26ec576ecc379f402d0f5d2b27a7bb"),
            account_address: Some(String::from("zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi")),
            passphrase: None,
        };
        let lattice = LatticeClient::new(chain_config, connecting_node_config, credential_config, None);
        let block = lattice.http_client.get_current_tx_daemon_block(&Address::new(owner)).await.unwrap();

        let mut tx = VoteBuiltinContract::new().new_vote_tx("0x012629af43a2e7cf024cdaeb8c108078b3b62a9f171300000000000000", true);

        tx.height = block.current_tblock_height + 1;
        tx.parent_hash = block.current_tblock_hash;
        tx.daemon_hash = block.current_dblock_hash;

        let res = lattice.sign_and_send_tx(tx).await;
        match res {
            Err(err) => println!("Err {}", err),
            Ok(v) => println!("Hash {}", v)
        }
    }
}