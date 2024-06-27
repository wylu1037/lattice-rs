use std::any::Any;

use abi::abi::Abi;
use crypto::sign::KeyPair;
use model::{Cryptography, Error, HexString};
use model::common::Address;

use crate::builder::{DeployContractBuilder, TransactionBuilder};
use crate::client::HttpClient;

/// Lattice Client
pub struct LatticeClient<'a> {
    /// 区块链ID
    chain_id: u64,
    /// 区块链节点ip
    node_ip: &'a str,
    /// 区块链节点端口
    node_port: u16,

    /// 账户私钥
    sk: &'a str,
    /// 账户地址
    account_address: String,

    /// 可选项目
    options: Options,

    /// 节点的http client
    pub http_client: HttpClient,
}

/// 可选项
pub struct Options {
    /// Default Sm2p256v1
    cryptography: Option<Cryptography>,
}

impl Options {
    fn get_cryptography(&self) -> Cryptography {
        self.cryptography.unwrap_or(Cryptography::Sm2p256v1)
    }
}

impl<'a> LatticeClient<'a> {
    fn new(chain_id: u64, node_ip: &'a str, node_port: u16, sk: &'a str, account_address: Option<String>, options: Option<Options>) -> Self {
        let options: Options = options.unwrap_or_else(|| Options { cryptography: Some(Cryptography::Sm2p256v1) });
        let address = match account_address {
            None => {
                let key_pair = KeyPair::from_secret_key(&HexString::new(sk).decode(), options.get_cryptography());
                key_pair.address()
            }
            Some(v) => v
        };

        LatticeClient {
            chain_id,
            node_ip,
            node_port,
            sk,
            account_address: address,

            options,
            http_client: HttpClient::new(node_ip, node_port),
        }
    }

    async fn call_contract(&self, contract_address: &str, abi: &str, fn_name: &str, args: Vec<Box<dyn Any>>, payload: Option<&str>) -> Result<String, Error> {
        // Get latest block
        let block = self.http_client.get_current_tx_daemon_block(&Address::new(&self.account_address)).await.unwrap();
        let abi = Abi::new(abi);
        let data = abi.encode(fn_name, args);

        let mut transaction = DeployContractBuilder::builder()
            .set_current_block(block)
            .set_owner(&self.account_address)
            .set_linker(contract_address)
            .set_code(&data)
            .set_payload(payload.unwrap_or("0x"))
            .build();

        // Sign transaction
        let sk = HexString::new(&self.sk).decode();
        let options = &self.options;
        let (_, signature) = transaction.sign(self.chain_id, &sk, options.get_cryptography());
        transaction.sign = signature;

        self.http_client.send_raw_tx(transaction).await
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_call_contract() {
        let abi_string = r#"[{"inputs":[],"name":"decrementCounter","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[],"name":"getCount","outputs":[{"internalType":"int256","name":"","type":"int256"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"incrementCounter","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#;
        let lattice = LatticeClient::new(1, "192.168.1.185", 13000, "0x23d5b2a2eb0a9c8b86d62cbc3955cfd1fb26ec576ecc379f402d0f5d2b27a7bb", None, None);
        let result = lattice.call_contract("zltc_nbrZcx1AzBXC361nWSwry8JgSJNEzrNiD", abi_string, "incrementCounter", vec![], None).await;
    }
}
