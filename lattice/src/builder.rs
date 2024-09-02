use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crypto::Transaction;
use crypto::transaction::TxType;
use model::block::LatestBlock;

/// 交易构造
pub trait TransactionBuilder {
    /// 关联类型：在一个trait内部定义一个占位类型
    fn builder() -> Self;
    fn set_current_block(self, block: LatestBlock) -> Self;
    fn set_owner(self, owner: &str) -> Self;
    fn set_linker(self, linker: &str) -> Self;
    fn set_code(self, code: &str) -> Self;
    fn set_payload(self, payload: &str) -> Self;
    fn set_amount(self, amount: Option<u128>) -> Self;
    fn set_joule(self, joule: Option<u128>) -> Self;
    fn build(self) -> Transaction;
}

/// # 定义建造者模式宏
macro_rules! impl_transaction_builder {
    ($builder:ident, $tx_type:expr) => {
        
        #[derive(Serialize, Deserialize, Debug)]
        pub struct $builder {
            transaction: Transaction,
        }

        impl TransactionBuilder for $builder {
            fn builder() -> Self {
                let mut transaction = Transaction::empty_tx();
                transaction.tx_type = $tx_type;
                transaction.timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                $builder { transaction }
            }

            /// # 设置交易的当前区块信息，包括账户高度，父交易哈希，父区块哈希等等
            ///
            /// ## 入参
            /// + `block: CurrentTDBlock`:
            ///
            /// ## 出参
            /// + `Self`
            fn set_current_block(mut self, block: LatestBlock) -> Self {
                self.transaction.height = block.height + 1;
                self.transaction.parent_hash = block.hash;
                self.transaction.daemon_hash = block.daemon_hash;
                self
            }

            /// # 设置交易的所有者（发送者）
            ///
            /// ## 入参
            /// + `owner: &str`: 账户地址，示例：zltc_UXpJCXdhTkg6edriiaRUVkYgTfv2Z5npe
            ///
            /// ## 出参
            /// + `Self`
            fn set_owner(mut self, owner: &str) -> Self {
                self.transaction.owner = owner.to_string();
                self
            }

            /// # 设置交易的接收者
            ///
            /// ## 入参
            /// + `linker: &str`: 账户地址，示例：zltc_nbrZcx1AzBXC361nWSwry8JgSJNEzrNiD
            ///
            /// ## 出参
            /// + `Self`
            fn set_linker(mut self, linker: &str) -> Self {
                self.transaction.linker = Some(linker.to_string());
                self
            }

            /// # 设置合约的data
            ///
            /// ## 入参
            /// + `code: &str`:
            ///
            /// ## 出参
            /// + `Self`
            fn set_code(mut self, code: &str) -> Self {
                self.transaction.code = Some(code.to_string());
                self
            }

            /// # 设置交易的备注，hex string
            ///
            /// ## 入参
            /// + `payload: &str`: payload, 示例：0x0102
            ///
            /// ## 出参
            /// + `Self`
            fn set_payload(mut self, payload: &str) -> Self {
                self.transaction.payload = Some(payload.to_string());
                self
            }

            /// # 设置交易的amount
            ///
            /// ## 入参
            /// + `amount: Option<u128>`: 转账数量
            ///
            /// ## 出参
            /// + `Self`
            fn set_amount(mut self, amount: Option<u128>) -> Self {
                self.transaction.amount = amount;
                self
            }

            /// # 设置交易的手续费
            ///
            /// ## 入参
            /// + `joule: Option<u128>`: 交易手续费
            ///
            /// ## 出参
            /// + `Self`
            fn set_joule(mut self, joule: Option<u128>) -> Self {
                self.transaction.joule = joule;
                self
            }

            fn build(self) -> Transaction {
                self.transaction
            }
        }
    };
}

impl_transaction_builder!(TransferBuilder, TxType::Send);
impl_transaction_builder!(DeployContractBuilder, TxType::Contract);
impl_transaction_builder!(CallContractBuilder, TxType::Execute);

#[cfg(test)]
mod test {
    use std::thread;
    use std::time::Duration;

    use abi::abi::Abi;
    use model::{Curve, HexString};
    use model::common::Address;

    use crate::client::HttpClient;

    use super::*;

    const CHAIN_ID: u64 = 1;

    #[test]
    fn test_transfer() {
        let client = HttpClient::new("192.168.1.185", 13000);
        let result = client.get_latest_block(CHAIN_ID, &Address::new("zltc_UXpJCXdhTkg6edriiaRUVkYgTfv2Z5npe"));
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
                let (_pow, signature) = transaction.sign(1, &sk, Curve::Sm2p256v1);
                transaction.sign = signature;

                let result = client.send_raw_tx(CHAIN_ID, transaction);
                match result {
                    Err(err) => println!("Error: {:?}", err),
                    Ok(hash) => println!("Hash: {:?}", hash)
                }
            }
        }
    }

    #[test]
    fn test_deploy_counter_contract() {
        let data = "0x60806040526000805534801561001457600080fd5b50610278806100246000396000f3fe608060405234801561001057600080fd5b50600436106100415760003560e01c80635b34b96614610046578063a87d942c14610050578063f5c5ad831461006e575b600080fd5b61004e610078565b005b610058610093565b60405161006591906100d0565b60405180910390f35b61007661009c565b005b600160008082825461008a919061011a565b92505081905550565b60008054905090565b60016000808282546100ae91906101ae565b92505081905550565b6000819050919050565b6100ca816100b7565b82525050565b60006020820190506100e560008301846100c1565b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6000610125826100b7565b9150610130836100b7565b9250817f7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0383136000831215161561016b5761016a6100eb565b5b817f80000000000000000000000000000000000000000000000000000000000000000383126000831216156101a3576101a26100eb565b5b828201905092915050565b60006101b9826100b7565b91506101c4836100b7565b9250827f8000000000000000000000000000000000000000000000000000000000000000018212600084121516156101ff576101fe6100eb565b5b827f7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff018213600084121615610237576102366100eb565b5b82820390509291505056fea2646970667358221220d841351625356129f6266ada896818d690dbc4b0d176774a97d745dfbe2fe50164736f6c634300080b0033";
        let client = HttpClient::new("192.168.1.185", 13000);
        let result = client.get_latest_block(CHAIN_ID, &Address::new("zltc_UXpJCXdhTkg6edriiaRUVkYgTfv2Z5npe"));
        match result {
            Err(err) => println!("Error: {:?}", err),
            Ok(block) => {
                let mut transaction = DeployContractBuilder::builder()
                    .set_current_block(block)
                    .set_owner("zltc_UXpJCXdhTkg6edriiaRUVkYgTfv2Z5npe")
                    .set_linker("zltc_nbrZcx1AzBXC361nWSwry8JgSJNEzrNiD")
                    .set_code(data)
                    .build();
                let sk = HexString::new("0x00a50da54a1987bf5ddd773e9c151bd40aa5d1281b8936dbdec93a9d0a04e4ca").decode();
                let (_pow, signature) = transaction.sign(1, &sk, Curve::Sm2p256v1);
                transaction.sign = signature;

                let result = client.send_raw_tx(CHAIN_ID, transaction);
                match result {
                    Err(err) => println!("Error: {:?}", err),
                    Ok(hash) => {
                        println!("Hash: {}", hash);
                        thread::sleep(Duration::from_secs(1));
                        let result = client.get_receipt(CHAIN_ID, &hash);
                        match result {
                            Err(err) => println!("Get receipt err: {:?}", err),
                            Ok(receipt) => println!("Receipt: {:?}", receipt)
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_execute_counter_contract() {
        let client = HttpClient::new("192.168.1.185", 13000);
        let result = client.get_latest_block(CHAIN_ID, &Address::new("zltc_UXpJCXdhTkg6edriiaRUVkYgTfv2Z5npe"));
        match result {
            Err(err) => println!("Error: {:?}", err),
            Ok(block) => {
                let abi_string = r#"[{"inputs":[],"name":"decrementCounter","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[],"name":"getCount","outputs":[{"internalType":"int256","name":"","type":"int256"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"incrementCounter","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#;
                let code = Abi::new(abi_string).encode("incrementCounter", vec![]);

                let mut transaction = CallContractBuilder::builder()
                    .set_current_block(block)
                    .set_owner("zltc_UXpJCXdhTkg6edriiaRUVkYgTfv2Z5npe")
                    .set_linker("zltc_dqUuNMBGSKWC6nquq18SNPRBftBp7Qm6g")
                    .set_code(&code)
                    .build();
                let sk = HexString::new("0x00a50da54a1987bf5ddd773e9c151bd40aa5d1281b8936dbdec93a9d0a04e4ca").decode();
                let (_pow, signature) = transaction.sign(1, &sk, Curve::Sm2p256v1);
                transaction.sign = signature;

                let result = client.send_raw_tx(CHAIN_ID, transaction);
                match result {
                    Err(err) => println!("Error: {:?}", err),
                    Ok(hash) => println!("Hash: {:?}", hash)
                }
            }
        }
    }
}