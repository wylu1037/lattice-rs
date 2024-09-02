use std::collections::HashMap;
use std::ops::Add;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use moka::sync::Cache;

use model::block::LatestBlock;
use model::common::Address;

use crate::client::HttpClient;

/// 账户缓存的实现
pub trait AccountCacheTrait: Sync + Send {
    /// # 设置账户的区块缓存
    ///
    /// ## 入参
    /// + `chain_id: u64`:
    /// + `account_address: &str`:
    /// + `block: LatestBlock`:
    ///
    /// ## 出参
    fn set(&self, chain_id: u64, account_address: &str, block: LatestBlock);

    /// # 获取账户的区块缓存，缓存失效时，可从链上查询
    ///
    /// ## 入参
    /// + `chain_id: u64`:
    /// + `account_address: &str`:
    ///
    /// ## 出参
    /// + `LatestBlock`
    fn get(&self, chain_id: u64, account_address: &str) -> LatestBlock;

    /// # 设置http client
    ///
    /// ## 入参
    /// + `http_client: HttpClient`:
    fn set_http_client(&mut self, http_client: HttpClient);
}

/// 账户缓存的默认实现
pub struct DefaultAccountCache {
    /// 是否启用缓存
    enable: bool,
    /// 持有一个内存缓存的管理器
    cache: Cache<String, LatestBlock>,
    /// 持有一个链的http客户端
    http_client: HttpClient,
    /// 维护一个链（子链/通道）和其对应的守护区块过期时间的Map
    daemon_hash_expire_at_map: Mutex<HashMap<u64, SystemTime>>,
    /// 守护区块哈希的过期时长
    daemon_hash_expiration_duration: Duration,
}

impl DefaultAccountCache {
    pub fn new(enable: bool, daemon_hash_expiration_duration: Duration, http_client: HttpClient) -> Self {
        let cache = Cache::builder()
            // .time_to_live(Duration::from_secs(30 * 60)) // 固定时长后过期，每次访问不会续期
            .time_to_idle(Duration::from_secs(5 * 60))
            .build();

        let daemon_hash_expire_at_map = Mutex::new(HashMap::new());

        DefaultAccountCache {
            enable,
            cache,
            http_client,
            daemon_hash_expire_at_map,
            daemon_hash_expiration_duration,
        }
    }
}

impl AccountCacheTrait for DefaultAccountCache {
    fn set(&self, chain_id: u64, account_address: &str, block: LatestBlock) {
        if !&self.enable {
            return;
        }
        let key = format!("{}_{}", chain_id, account_address);
        let _cache = self.cache.clone();
        _cache.insert(key.clone(), block);

        let mut map = self.daemon_hash_expire_at_map.lock().unwrap();
        if !map.contains_key(&chain_id) {
            map.insert(chain_id, SystemTime::now().add(self.daemon_hash_expiration_duration));
        }
    }

    fn get(&self, chain_id: u64, account_address: &str) -> LatestBlock {
        if !&self.enable {
            let result = self.http_client.get_latest_block(chain_id, &Address::new(account_address));
            return result.unwrap();
        }

        let key = format!("{}_{}", chain_id, account_address);
        let cached_block_option = self.cache.get(&key);
        let mut cached_block: LatestBlock;
        match cached_block_option {
            Some(block) => {
                cached_block = block
            }
            None => {
                let result = self.http_client.get_latest_block(chain_id, &Address::new(account_address));
                cached_block = result.unwrap();
            }
        }

        // 判断守护区块的哈希是否过期
        let mut map = self.daemon_hash_expire_at_map.lock().unwrap();
        if map.contains_key(&chain_id) {
            let daemon_hash_expire_at = map.get(&chain_id).unwrap();
            if SystemTime::now() > *daemon_hash_expire_at {
                let latest_daemon_block = self.http_client.get_latest_daemon_block(chain_id).unwrap();
                let daemon_hash_expire_at = SystemTime::now().add(self.daemon_hash_expiration_duration);
                map.insert(chain_id, daemon_hash_expire_at);
                cached_block.daemon_hash = latest_daemon_block.hash;
            }
        } else {
            let daemon_hash_expire_at = SystemTime::now().add(self.daemon_hash_expiration_duration);
            map.insert(chain_id, daemon_hash_expire_at);
        }

        return cached_block;
    }

    fn set_http_client(&mut self, http_client: HttpClient) {
        self.http_client = http_client
    }
}

#[cfg(test)]
mod test {
    use std::thread;

    use super::*;

    #[test]
    fn test() {
        let http_client = HttpClient::new("192.168.1.185", 13800);
        let default = DefaultAccountCache::new(true, Duration::from_secs(1), http_client);
        let mut block = default.get(2, "zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi");
        println!("block: {:?}", block);
        thread::sleep(Duration::from_secs(2));
        block = default.get(2, "zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi");
        println!("block: {:?}", block);
    }
}