use std::any::Any;
use std::time::Duration;

use regex::Regex;

use abi::abi::Abi;
use crypto::Transaction;
use model::{Curve, Error, HexString};
use model::block::LatestBlock;
use model::common::Address;
use model::constants::{PREFIX_OF_HEX, ZERO_HASH_STRING, ZERO_ZLTC_ADDRESS};
use model::receipt::Receipt;
use wallet::file_key::FileKey;

use crate::account_cache::{AccountCacheTrait, DefaultAccountCache};
use crate::account_lock::{AccountLockTrait, DefaultAccountLock};
use crate::builder::{CallContractBuilder, DeployContractBuilder, TransactionBuilder, TransferBuilder};
use crate::client::HttpClient;
use crate::constants::REGEX_PRIVATE_KEY;

/// 链配置
#[derive(Debug, Clone, Copy)]
pub struct ChainConfig {
    /// 椭圆曲线，Default Sm2p256v1
    pub curve: Curve,
    /// 是否不包含通证，false:有通证 true:无通证
    pub token_less: bool,
}

/// 连接节点配置
#[derive(Debug, Clone)]
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
    /// 账户地址，示例：zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi
    pub account_address: String,
    /// 私钥，示例：0x23d5b2a2eb0a9c8b86d62cbc3955cfd1fb26ec576ecc379f402d0f5d2b27a7bb
    pub sk: String,
    /// 身份密码，需要和 FileKey 一起使用
    pub passphrase: Option<String>,
    /// file_key
    pub file_key: Option<String>,
}

impl Credentials {
    /// # 获取私钥
    ///
    /// ## 出参
    /// + `String`: 示例，0x23d5b2a2eb0a9c8b86d62cbc3955cfd1fb26ec576ecc379f402d0f5d2b27a7bb
    fn get_sk(&self) -> String {
        let regex = Regex::new(REGEX_PRIVATE_KEY).unwrap();
        if regex.is_match(&self.sk) {
            return self.sk.clone();
        } else {
            let passphrase = self.passphrase.as_ref().expect("身份密码不能为空");
            let file_key_json = self.file_key.as_ref().expect("FileKey不能为空");
            let file_key = FileKey::new(file_key_json);
            let keypair = file_key.decrypt(passphrase).unwrap();
            let sk_bytes = keypair.secret_key.to_bytes_be();
            HexString::from(&sk_bytes).hex_string
        }
    }

    /// # 从credential中获取账户地址
    ///
    /// ## 出参
    /// + `String`: 账户地址，示例`zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi`
    fn get_account_address(&self) -> String {
        let addr = &self.account_address;
        addr.to_string()
    }
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

    /// 可选配置
    options: Options,

    /// 节点的http client
    pub http_client: HttpClient,

    /// 账户锁
    account_lock: Box<dyn AccountLockTrait>,

    /// 账户缓存
    account_cache: Box<dyn AccountCacheTrait>,
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
    pub fn new(chain_config: ChainConfig, connecting_node_config: ConnectingNodeConfig, options: Option<Options>, account_lock: Option<Box<dyn AccountLockTrait>>, account_cache: Option<Box<dyn AccountCacheTrait>>) -> Self {
        let options: Options = options.unwrap_or_else(|| Options::default());
        let http_client = connecting_node_config.new_http_client();
        let account_lock = account_lock.unwrap_or_else(|| Box::new(DefaultAccountLock::new()));
        let account_cache = account_cache.unwrap_or_else(|| Box::new(DefaultAccountCache::new(true, Duration::from_secs(10), http_client.clone())));

        LatticeClient {
            chain_config,
            connecting_node_config,
            options,
            http_client,
            account_lock,
            account_cache,
        }
    }

    /// # 转账
    ///
    /// ## 入参
    /// + `credentials: Credentials`: 身份凭证
    /// + `chain_id: u64`:
    /// + `payload: &str`:
    /// + `amount: Option<u128>`:
    /// + `joule: Option<u128>`:
    ///
    /// ## 出参
    /// + `Result<String, Error>`
    pub fn transfer(&self, credentials: Credentials, chain_id: u64, payload: &str, amount: Option<u128>, joule: Option<u128>) -> Result<String, Error> {
        let account_lock = self.account_lock.obtain(chain_id, credentials.account_address.as_str());
        let _guard = account_lock.lock().unwrap();

        let mut block = self.account_cache.get(chain_id, credentials.account_address.as_str());
        // let block = self.http_client.get_latest_block(chain_id, &Address::new(credentials.get_account_address().as_str())).await.unwrap();

        let mut transaction = TransferBuilder::builder()
            .set_current_block(block.clone())
            .set_owner(credentials.account_address.as_str())
            .set_linker(ZERO_ZLTC_ADDRESS)
            .set_payload(payload)
            .set_amount(amount)
            .set_joule(joule)
            .build();

        // Sign transaction
        let sk = HexString::new(credentials.get_sk().as_str()).decode();
        let (_, signature) = transaction.sign(chain_id, &sk, self.chain_config.curve);
        transaction.sign = signature;

        let result = self.http_client.send_raw_tx(chain_id, transaction);

        match result {
            Ok(hash) => {
                block.hash = hash.clone();
                block.height = block.height + 1;
                self.account_cache.set(chain_id, credentials.account_address.as_str(), block);

                Ok(hash)
            }
            Err(e) => Err(e)
        }
    }

    /// # 部署合约
    ///
    /// ## 入参
    /// + `credentials: Credentials`:
    /// + `chain_id: u64`:
    /// + `code: &str`:
    /// + `amount: Option<u128>`:
    /// + `joule: Option<u128>`:
    /// + `payload: Option<&str>`:
    ///
    /// ## 出参
    /// + `Result<String, Error>`
    pub fn deploy_contract(&self, credentials: Credentials, chain_id: u64, code: &str, amount: Option<u128>, joule: Option<u128>, payload: Option<&str>) -> Result<String, Error> {
        // Get latest block
        let block = self.http_client.get_latest_block(chain_id, &Address::new(credentials.get_account_address().as_str())).unwrap();

        let mut transaction = DeployContractBuilder::builder()
            .set_current_block(block)
            .set_owner(credentials.account_address.as_str())
            .set_linker(ZERO_ZLTC_ADDRESS)
            .set_code(code)
            .set_payload(payload.unwrap_or(PREFIX_OF_HEX))
            .set_amount(amount)
            .set_joule(joule)
            .build();

        // Sign transaction
        let sk = HexString::new(credentials.get_sk().as_str()).decode();
        let (_, signature) = transaction.sign(chain_id, &sk, self.chain_config.curve);
        transaction.sign = signature;

        self.http_client.send_raw_tx(chain_id, transaction)
    }

    /// # 调用合约
    ///
    /// ## 入参
    /// + `credentials: Credentials`: 上链的凭证
    /// + `chain_id: u64`: 链ID
    /// + `contract_address: &str`: 合约地址
    /// + `code: &str`:
    /// + `amount: Option<u128>`
    /// + `joule: Option<u128>`
    /// + `payload: Option<&str>`
    ///
    /// ## 出参
    /// + `Result<String, Error>`
    pub fn call_contract(&self, credentials: Credentials, chain_id: u64, contract_address: &str, code: &str, amount: Option<u128>, joule: Option<u128>, payload: Option<&str>) -> Result<String, Error> {
        // Get latest block
        let block = self.http_client.get_latest_block(chain_id, &Address::new(credentials.get_account_address().as_str())).unwrap();

        let mut transaction = CallContractBuilder::builder()
            .set_current_block(block)
            .set_owner(credentials.account_address.as_str())
            .set_linker(contract_address)
            .set_code(code)
            .set_payload(payload.unwrap_or(PREFIX_OF_HEX))
            .set_amount(amount)
            .set_joule(joule)
            .build();

        // Sign transaction
        let sk = HexString::new(credentials.get_sk().as_str()).decode();
        let (_, signature) = transaction.sign(chain_id, &sk, self.chain_config.curve);
        transaction.sign = signature;

        self.http_client.send_raw_tx(chain_id, transaction)
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
    pub fn pre_call_contract(&self, chain_id: u64, owner: &str, contract_address: &str, code: &str, payload: Option<&str>) -> Result<Receipt, Error> {
        let transaction = CallContractBuilder::builder()
            .set_current_block(
                LatestBlock {
                    height: 0,
                    hash: ZERO_HASH_STRING.to_string(),
                    daemon_hash: ZERO_HASH_STRING.to_string(),
                })
            .set_owner(owner)
            .set_linker(contract_address)
            .set_code(code)
            .set_payload(payload.unwrap_or("0x"))
            .build();

        self.http_client.pre_call_contract(chain_id, transaction)
    }

    /// # 签名交易并发送交易
    ///
    /// ## 入参
    /// + `chain_id: u64`: 链ID
    /// + `tx: Transaction`: 交易
    ///
    /// ## 出参
    /// + `Result<String, Error>`
    pub fn sign_and_send_tx(self, credentials: Credentials, chain_id: u64, mut tx: Transaction) -> Result<String, Error> {
        let sk = HexString::new(&credentials.get_sk()).decode();
        let (_, signature) = tx.sign(chain_id, &sk, self.chain_config.curve);
        tx.sign = signature;

        self.http_client.send_raw_tx(chain_id, tx)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const COUNTER_ABI: &str = r#"[
        {
            "inputs": [],
            "name": "decrementCounter",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "getCount",
            "outputs": [
                {
                    "internalType": "int256",
                    "name": "",
                    "type": "int256"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "incrementCounter",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }
    ]"#;

    const COUNTER_BYTECODE: &str = "0x60806040526000805534801561001457600080fd5b50610278806100246000396000f3fe608060405234801561001057600080fd5b50600436106100415760003560e01c80635b34b96614610046578063a87d942c14610050578063f5c5ad831461006e575b600080fd5b61004e610078565b005b610058610093565b60405161006591906100d0565b60405180910390f35b61007661009c565b005b600160008082825461008a919061011a565b92505081905550565b60008054905090565b60016000808282546100ae91906101ae565b92505081905550565b6000819050919050565b6100ca816100b7565b82525050565b60006020820190506100e560008301846100c1565b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6000610125826100b7565b9150610130836100b7565b9250817f7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0383136000831215161561016b5761016a6100eb565b5b817f80000000000000000000000000000000000000000000000000000000000000000383126000831216156101a3576101a26100eb565b5b828201905092915050565b60006101b9826100b7565b91506101c4836100b7565b9250827f8000000000000000000000000000000000000000000000000000000000000000018212600084121516156101ff576101fe6100eb565b5b827f7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff018213600084121615610237576102366100eb565b5b82820390509291505056fea2646970667358221220d841351625356129f6266ada896818d690dbc4b0d176774a97d745dfbe2fe50164736f6c634300080b0033";
    const CHAIN_ID: u64 = 2;

    struct Setup {
        chain_config: ChainConfig,
        connecting_node_config: ConnectingNodeConfig,
        credentials: Credentials,
        lattice: LatticeClient,
    }

    impl Setup {
        fn new() -> Self {
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
                account_address: String::from("zltc_dS73XWcJqu2uEk4cfWsX8DDhpb9xsaH9s"),
                passphrase: None,
                file_key: None,
            };
            let lattice = LatticeClient::new(chain_config.clone(), connecting_node_config.clone(), None, None, None);
            // 浅浅青末云顶款
            Setup {
                chain_config,
                connecting_node_config,
                credentials,
                lattice,
            }
        }
    }

    #[test]
    fn test_transfer() {
        let setup = Setup::new();
        let result = setup.lattice.transfer(setup.credentials, CHAIN_ID, "0x01", None, None);
        match result {
            Ok(hash) => { println!("转账交易的哈希：{}", hash) }
            Err(e) => { println!("转账错误，{}", e); }
        }
    }

    #[test]
    fn test_deploy_counter_contract() {
        let setup = Setup::new();
        let deploy_result = setup.lattice.deploy_contract(setup.credentials, 2, COUNTER_BYTECODE, None, None, None);
        match deploy_result {
            Ok(hash) => { println!("部署合约的交易哈希：{}", hash); }
            Err(e) => { println!("部署合约错误，{}", e); }
        }
    }

    #[test]
    fn test_pre_call_contract() {
        let setup = Setup::new();
        let abi = Abi::new(COUNTER_ABI);
        let code = abi.encode("getCount", vec![]);
        let _result = setup.lattice.pre_call_contract(2, "zltc_dS73XWcJqu2uEk4cfWsX8DDhpb9xsaH9s", "zltc_Yw1XgbrmeEdJcQJcofN48XD5vxwST4uiy", &code, None);
        match _result {
            Ok(receipt) => { println!("预调用合约，{:?}", serde_json::to_string(&receipt)) }
            Err(e) => { println!("预调用合约错误，{}", e) }
        }
    }

    #[test]
    fn test_decrypt_file_key_from_credentials() {
        let file_key = r#"{"uuid":"123f1bf5-5599-45c4-8566-9a6440ba359f","address":"zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi","cipher":{"aes":{"cipher":"aes-128-ctr","cipherText":"8f6de52c0be43ae438feddea4c210772da23b9333242b7416446eae889b594e0","iv":"1ad693b4d8089da0492b9c8c49bc60d3"},"kdf":{"kdf":"scrypt","kdfParams":{"DKLen":32,"n":262144,"p":1,"r":8,"salt":"309210a97fbf705eed7bf3485c16d6922a21591297b52c0c59b4f7495863e300"}},"cipherText":"8f6de52c0be43ae438feddea4c210772da23b9333242b7416446eae889b594e0","mac":"335fab3901f8f5c4408b7d6a310ec29cf5bd3792deb696f1b10282e823241c96"},"isGM":true}"#;
        let credentials = Credentials {
            account_address: String::from(""),
            sk: String::from(""),
            passphrase: Some(String::from("Root1234")),
            file_key: Some(file_key.to_string()),
        };
        let sk = credentials.get_sk();
        let expect = "0x23d5b2a2eb0a9c8b86d62cbc3955cfd1fb26ec576ecc379f402d0f5d2b27a7bb".to_string();
        assert_eq!(expect, sk)
    }
}
