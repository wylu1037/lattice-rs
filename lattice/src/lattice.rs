use std::any::Any;

use abi::abi::Abi;
use crypto::sign::KeyPair;
use crypto::Transaction;
use model::{Curve, Error, HexString};
use model::block::CurrentTDBlock;
use model::common::Address;
use model::constants::ZERO_HASH_STRING;
use model::receipt::Receipt;
use wallet::file_key::FileKey;

use crate::builder::{CallContractBuilder, TransactionBuilder};
use crate::client::HttpClient;

/// 链配置
pub struct ChainConfig {
    /// 椭圆曲线，Default Sm2p256v1
    pub curve: Curve,
    /// 是否不包含通证，false:有通证 true:无通证
    pub token_less: bool,
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
pub struct Credentials {
    /// 账户地址
    pub account_address: Option<String>,
    /// 私钥
    pub sk: String,
    /// 身份密码，需要和 FileKey 一起使用
    pub passphrase: Option<String>,
    /// file_key
    pub file_key: Option<String>,
}


/// 重试策略
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

    /// 凭证信息
    credentials: Credentials,

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
    daemon_block_hash: String,
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
        let credentials = &self.credentials;
        let owner = credentials.account_address.as_ref().unwrap();
        owner.to_string()
    }

    /// # 从credential中获取私钥
    ///
    /// ## 出参
    /// + `String`: 16进制的私钥
    fn get_sk(&self) -> String {
        let credentials = &self.credentials;
        return if credentials.sk.is_empty() {
            // decrypt the private key
            let file_key = credentials.file_key.clone().expect("FileKey不能为空").as_str();
            let file_key = FileKey::new(file_key);
            let passphrase = credentials.passphrase.clone().expect("身份密码不能为空").as_str();
            let result = file_key.decrypt(passphrase);
            let sk = result.unwrap().secret_key.to_str_radix(16);
            sk
        } else {
            credentials.sk.to_string()
        };
    }

    pub fn new(chain_config: ChainConfig, connecting_node_config: ConnectingNodeConfig, mut credentials: Credentials, options: Option<Options>) -> Self {
        let options: Options = options.unwrap_or_else(|| Options::default());
        let address = match credentials.account_address {
            Some(addr) => addr,
            None => {
                let key_pair = KeyPair::from_secret_key(&HexString::new(&credentials.sk).decode(), chain_config.curve);
                key_pair.address()
            }
        };
        credentials.account_address = Some(address);
        let http_client = connecting_node_config.new_http_client();

        LatticeClient {
            chain_config,
            connecting_node_config,
            credentials,
            options,
            http_client,
        }
    }

    /// # 调用合约
    ///
    /// ## 入参
    /// + `chain_id: u64`: 链ID
    /// + `contract_address: &str`: 合约地址
    /// + `code: &str`:
    /// + `amount: Option<u128>`
    /// + `joule: Option<u128>`
    /// + `payload: Option<&str>`
    ///
    /// ## 出参
    pub async fn call_contract(&self, chain_id: u64, contract_address: &str, code: &str, amount: Option<u128>, joule: Option<u128>, payload: Option<&str>) -> Result<String, Error> {
        // Get latest block
        let block = self.http_client.get_latest_block(&Address::new(&self.get_owner())).await.unwrap();

        let mut transaction = CallContractBuilder::builder()
            .set_current_block(block)
            .set_owner(&self.get_owner())
            .set_linker(contract_address)
            .set_code(code)
            .set_payload(payload.unwrap_or("0x"))
            .build();

        // Sign transaction
        let sk = HexString::new(&self.get_sk()).decode();
        let (_, signature) = transaction.sign(chain_id, &sk, self.chain_config.curve);
        transaction.sign = signature;

        self.http_client.send_raw_tx(transaction).await
    }

    /// # 预调用合约（不会上链）
    ///
    /// ## 入参
    /// + `chain_id: u64`: 链ID
    /// + `contract_address: &str`: 合约地址
    /// + `code: &str`: 合约代码
    /// + `payload: Option<&str>`: 交易备注
    ///
    /// ## 出参
    /// + `Result<Receipt, Error>`
    pub async fn pre_call_contract(&self, chain_id: u64, contract_address: &str, code: &str, payload: Option<&str>) -> Result<Receipt, Error> {
        let transaction = CallContractBuilder::builder()
            .set_current_block(
                CurrentTDBlock {
                    current_dblock_hash: ZERO_HASH_STRING.to_string(),
                    current_tblock_hash: ZERO_HASH_STRING.to_string(),
                    current_tblock_height: 0,
                })
            .set_owner(&self.get_owner())
            .set_linker(contract_address)
            .set_code(code)
            .set_payload(payload.unwrap_or("0x"))
            .build();

        self.http_client.pre_call_contract(transaction).await
    }

    pub async fn sign_and_send_tx(self, chain_id: u64, mut tx: Transaction) -> Result<String, Error> {
        if tx.owner.is_empty() {
            tx.owner = self.get_owner();
        }

        let sk = HexString::new(&self.get_sk()).decode();
        let (_, signature) = tx.sign(chain_id, &sk, self.chain_config.curve);
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
            curve: Curve::Sm2p256v1,
            token_less: true,
        };
        let connecting_node_config = ConnectingNodeConfig {
            ip: String::from("192.168.1.185"),
            http_port: 13800,
            websocket_port: 13001,
        };
        let credentials = Credentials {
            sk: String::from("0xdbd91293f324e5e49f040188720c6c9ae7e6cc2b4c5274120ee25808e8f4b6a7"),
            account_address: Some(String::from("zltc_dS73XWcJqu2uEk4cfWsX8DDhpb9xsaH9s")),
            passphrase: None,
            file_key: None,
        };
        let lattice = LatticeClient::new(chain_config, connecting_node_config, credentials, None);
        let abi = Abi::new(abi_string);
        let code = abi.encode("getCount", vec![]);
        let _result = lattice.pre_call_contract(2, "zltc_d1pTRCCH2F6McFCmXYCB743L7spuNtw31", &code, None).await;
        if let Ok(hash) = _result {
            println!("{:?}", hash)
        }
    }
}
