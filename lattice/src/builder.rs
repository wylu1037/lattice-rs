use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crypto::Transaction;
use crypto::transaction::TxType;
use model::block::CurrentTDBlock;

/// 交易构造
pub trait TransactionBuilder {
    /// 关联类型：在一个trait内部定义一个占位类型

    fn builder() -> Self;

    fn set_current_block(self, block: CurrentTDBlock) -> Self;
    fn set_owner(self, owner: &str) -> Self;
    fn set_linker(self, linker: &str) -> Self;
    fn set_code(self, code: &str) -> Self;
    fn set_payload(self, payload: &str) -> Self;
    fn build(self) -> Transaction;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransferBuilder {
    transaction: Transaction,
}

pub struct DeployContractBuilder;

pub struct ExecuteContractBuilder;

impl TransactionBuilder for TransferBuilder {
    fn builder() -> Self {
        let mut transaction = Transaction::empty_tx();
        transaction.tx_type = TxType::Send;
        transaction.timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        transaction.version = 3;
        TransferBuilder { transaction }
    }

    fn set_current_block(mut self, block: CurrentTDBlock) -> Self {
        self.transaction.height = block.current_tblock_height + 1;
        self.transaction.parent_hash = block.current_tblock_hash;
        self.transaction.daemon_hash = block.current_dblock_hash;
        self
    }

    fn set_owner(mut self, owner: &str) -> Self {
        self.transaction.owner = owner.to_string();
        self
    }

    fn set_linker(mut self, linker: &str) -> Self {
        self.transaction.linker = Some(linker.to_string());
        self
    }

    fn set_code(mut self, code: &str) -> Self {
        self.transaction.code = Some(code.to_string());
        self
    }

    fn set_payload(mut self, payload: &str) -> Self {
        self.transaction.payload = Some(payload.to_string());
        self
    }

    fn build(self) -> Transaction {
        self.transaction
    }
}

#[cfg(test)]
mod test {
    use model::{Cryptography, HexString};
    use model::common::Address;

    use crate::client::HttpClient;

    use super::*;

    #[tokio::test]
    async fn test() {
        let client = HttpClient::new("192.168.1.185", 13000);
        let result = client.get_current_tx_daemon_block(&Address::new("zltc_UXpJCXdhTkg6edriiaRUVkYgTfv2Z5npe")).await;
        match result {
            Err(err) => println!("Error: {:?}", err),
            Ok(block) => {
                let mut transaction = TransferBuilder::builder()
                    .set_current_block(block)
                    .set_owner("zltc_UXpJCXdhTkg6edriiaRUVkYgTfv2Z5npe")
                    .set_linker("zltc_nbrZcx1AzBXC361nWSwry8JgSJNEzrNiD")
                    .set_payload("0x0102")
                    .build();
                let sk = HexString::new("0x00a50da54a1987bf5ddd773e9c151bd40aa5d1281b8936dbdec93a9d0a04e4ca").decode();
                let (pow, signature) = transaction.sign(1, &sk, Cryptography::Sm2p256v1);
                transaction.sign = signature;

                let result = client.send_raw_tx(transaction).await;
                match result {
                    Err(err) => println!("Error: {:?}", err),
                    Ok(hash) => println!("Hash: {:?}", hash)
                }
            }
        }
    }
}