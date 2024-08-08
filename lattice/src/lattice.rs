use std::any::Any;

use abi::abi::Abi;
use crypto::sign::KeyPair;
use crypto::Transaction;
use model::{Cryptography, Error, HexString};
use model::block::CurrentTDBlock;
use model::common::Address;
use model::constants::ZERO_HASH_STRING;
use model::receipt::Receipt;

use crate::builder::{CallContractBuilder, TransactionBuilder};
use crate::client::HttpClient;

/// 链配置
pub struct ChainConfig {
    /// 区块链ID
    pub chain_id: u64,
    /// Default Sm2p256v1
    pub cryptography: Cryptography,
}

/// 连接节点配置
pub struct ConnectingNodeConfig {
    /// 节点IP 或者 节点域名
    pub ip: String,
    /// 节点http端口
    pub http_port: u16,
    /// websocket端口
    pub websocket_port: u16,
}

impl ConnectingNodeConfig {
    /// # 初始化一个节点的http客户端
    ///
    /// ## 入参
    /// + `&self`:
    ///
    /// ## 出参
    /// + `HttpClient`
    fn new_http_client(&self) -> HttpClient {
        HttpClient::new(&self.ip, self.http_port)
    }
}

/// 凭证配置
pub struct CredentialConfig {
    /// 私钥
    pub sk: String,
    /// 账户地址
    pub account_address: Option<String>,
    /// 身份密码，需要和 FileKey 一起使用
    pub passphrase: Option<String>,
}

pub struct RetryPolicy {}

/// 重试类型枚举
pub enum Retry {
    /// 退避算法
    BackOff,
    /// 固定间隔
    FixedInterval,
    /// 随机间隔
    RandomInterval,
}

/// Lattice Client
pub struct LatticeClient {
    /// 链配置
    chain_config: ChainConfig,

    /// 连接节点的配置
    connecting_node_config: ConnectingNodeConfig,

    /// 凭证信息配置
    credential_config: CredentialConfig,

    /// 可选配置
    options: Options,

    /// 节点的http client
    pub http_client: HttpClient,
}

/// 可选项
pub struct Options {
    /// 是否启用缓存
    enable_cache: Option<bool>,
    /// 缓存的过期时间，默认 5s
    cache_expiration_seconds: Option<u16>,
    /// daemon hash的过期时间，默认 15s
    daemon_hash_expiration_seconds: Option<u16>,
}

/// 缓存的区块信息
pub struct CachedBlock {
    /// 账户高度
    height: u64,
    /// 父交易哈希
    parent_hash: String,
    /// 守护区块哈希
    daemon_hash: String,
}

impl Options {
    fn default() -> Self {
        Options {
            enable_cache: Some(false),
            cache_expiration_seconds: Some(5),
            daemon_hash_expiration_seconds: Some(15),
        }
    }
}

impl LatticeClient {
    /// # 从credential中获取账户地址
    ///
    /// ## 入参
    /// + `&self`:
    ///
    /// ## 出参
    /// + `String`
    fn get_owner(&self) -> String {
        let credential_config = &self.credential_config;
        let owner = credential_config.account_address.as_ref().unwrap();
        owner.to_string()
    }

    /// # 从credential中获取私钥
    ///
    /// ## 入参
    /// + `&self`:
    ///
    /// ## 出参
    /// + `String`
    fn get_sk(&self) -> String {
        let credential_config = &self.credential_config;
        credential_config.sk.to_string()
    }

    fn get_chain_id(&self) -> u64 {
        let chain_config = &self.chain_config;
        chain_config.chain_id
    }

    pub fn new(chain_config: ChainConfig, connecting_node_config: ConnectingNodeConfig, mut credential_config: CredentialConfig, options: Option<Options>) -> Self {
        let options: Options = options.unwrap_or_else(|| Options::default());
        let address = match credential_config.account_address {
            Some(addr) => addr,
            None => {
                let key_pair = KeyPair::from_secret_key(&HexString::new(&credential_config.sk).decode(), chain_config.cryptography);
                key_pair.address()
            }
        };
        credential_config.account_address = Some(address);
        let http_client = connecting_node_config.new_http_client();

        LatticeClient {
            chain_config,
            connecting_node_config,
            credential_config,
            options,
            http_client,
        }
    }

    pub async fn call_contract(&self, contract_address: &str, abi: &str, fn_name: &str, args: Vec<Box<dyn Any>>, payload: Option<&str>) -> Result<String, Error> {
        // Get latest block
        let block = self.http_client.get_current_tx_daemon_block(&Address::new(&self.get_owner())).await.unwrap();
        let abi = Abi::new(abi);
        let data = abi.encode(fn_name, args);

        let mut transaction = CallContractBuilder::builder()
            .set_current_block(block)
            .set_owner(&self.get_owner())
            .set_linker(contract_address)
            .set_code(&data)
            .set_payload(payload.unwrap_or("0x"))
            .build();

        // Sign transaction
        let sk = HexString::new(&self.get_sk()).decode();
        let (_, signature) = transaction.sign(self.get_chain_id(), &sk, self.chain_config.cryptography);
        transaction.sign = signature;

        self.http_client.send_raw_tx(transaction).await
    }

    pub async fn pre_call_contract(&self, contract_address: &str, abi: &str, fn_name: &str, args: Vec<Box<dyn Any>>, payload: Option<&str>) -> Result<Receipt, Error> {
        let abi = Abi::new(abi);
        let data = abi.encode(fn_name, args);

        let transaction = CallContractBuilder::builder()
            .set_current_block(
                CurrentTDBlock {
                    current_dblock_hash: ZERO_HASH_STRING.to_string(),
                    current_tblock_hash: ZERO_HASH_STRING.to_string(),
                    current_tblock_height: 0,
                })
            .set_owner(&self.get_owner())
            .set_linker(contract_address)
            .set_code(&data)
            .set_payload(payload.unwrap_or("0x"))
            .build();

        self.http_client.pre_call_contract(transaction).await
    }

    pub async fn sign_and_send_tx(self, mut tx: Transaction) -> Result<String, Error> {
        if tx.owner.is_empty() {
            tx.owner = self.get_owner();
        }

        let sk = HexString::new(&self.get_sk()).decode();
        let (_, signature) = tx.sign(self.get_chain_id(), &sk, self.chain_config.cryptography);
        tx.sign = signature;

        self.http_client.send_raw_tx(tx).await
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_pre_call_contract() {
        let abi_string = r#"[{"inputs":[],"name":"decrementCounter","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[],"name":"getCount","outputs":[{"internalType":"int256","name":"","type":"int256"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"incrementCounter","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#;
        let chain_config = ChainConfig {
            chain_id: 1,
            cryptography: Cryptography::Sm2p256v1,
        };
        let connecting_node_config = ConnectingNodeConfig {
            ip: String::from("192.168.1.185"),
            http_port: 13000,
            websocket_port: 13001,
        };
        let credential_config = CredentialConfig {
            sk: String::from("0xdbd91293f324e5e49f040188720c6c9ae7e6cc2b4c5274120ee25808e8f4b6a7"),
            account_address: Some(String::from("zltc_dS73XWcJqu2uEk4cfWsX8DDhpb9xsaH9s")),
            passphrase: None,
        };
        let lattice = LatticeClient::new(chain_config, connecting_node_config, credential_config, None);
        let _result = lattice.pre_call_contract("zltc_d1pTRCCH2F6McFCmXYCB743L7spuNtw31", abi_string, "getCount", vec![], None).await;
        if let Ok(hash) = _result {
            println!("{:?}", hash)
        }
    }
}
