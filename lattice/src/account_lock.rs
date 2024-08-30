use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

struct AccountLock {
    /// 设计思路：
    /// + `RwLock` 允许并发读和独占写操作，通过write方法阻塞其它线程，保证对HashMap的操作是并发安全的
    /// + `Arc`（原子引用计数，Atomic Reference Counted）允许多个线程安全地共享同一个数据。通过`Arc::clone()`在多个线程间共享锁的所有权。
    /// + `Mutex` 是一种同步原语，用于在多线程环境中保护共享数据的访问。Mutex 提供了独占访问的机制，
    /// 确保同一时间只有一个线程可以访问被保护的数据。以此来保证一个账户的并发请求在服务端是串行执行的。
    /// + `usize`: 对同一账户的获取Mutex锁进行记录，获取一次，次数+1，释放时，次数-1，为0时，将账户的缓存在HashMap的Mutex锁删除掉。
    locks: RwLock<HashMap<String, (Arc<Mutex<()>>, usize)>>,
}

impl AccountLock {
    fn new() -> Self {
        AccountLock {
            locks: RwLock::new(HashMap::new())
        }
    }

    /// # 获取账户锁
    ///
    /// ## 入参
    /// + `chain_id: u64`: 链ID
    /// + `account_address: &str`: 账户地址
    ///
    /// ## 出参
    /// + `Arc<Mutex<()>>`: Mutex锁
    fn obtain(&self, chain_id: u64, account_address: &str) -> Arc<Mutex<()>> {
        let key = format!("{}_{}", chain_id, account_address);
        /// 当 RwLockWriteGuard 离开其作用域时，会自动释放锁，允许其他线程访问数据。
        let mut locks = self.locks.write().unwrap(); // 使用写锁阻塞其它线程

        let (lock, count) = locks.entry(key)
            .or_insert_with(|| (Arc::new(Mutex::new(())), 0));
        *count += 1;
        lock.clone() // 写锁离开作用域，自动释放锁
    }

    /// # 释放账户持有的Mutex在HashMap中的缓存
    ///
    /// ## 入参
    /// + `chain_id: u64`: 链ID
    /// + `account_address: &str`: 账户地址
    fn release(&self, chain_id: u64, account_address: &str) {
        let key = format!("{}_{}", chain_id, account_address);
        let mut locks = self.locks.write().unwrap(); // 使用写锁阻塞其它线程

        if let Some((_, count)) = locks.get_mut(&key) {
            if *count > 1 {
                *count -= 1;
            } else {
                locks.remove(&key);
            }
        } // RwLockWriteGuard<HashMap<...>> 离开作用域，自动释放锁
    }
}

fn handle_request(lock: Arc<Mutex<()>>, request_id: usize) {
    let _guard = lock.lock().unwrap();
    println!("Handling request {} for the account", request_id);
    thread::sleep(Duration::from_secs(2)); // 模拟请求处理时间
    println!("Finished request {}", request_id);
}

#[cfg(test)]
mod test {
    use std::thread;

    use super::*;

    #[test]
    fn test_account_lock() {
        let account_lock = AccountLock::new();

        let chain_id = 1;
        let address = "zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi";

        let mut handles = vec![];

        for i in 0..3 {
            let lock = account_lock.obtain(chain_id, address);
            let handle = thread::spawn(move || {
                handle_request(lock, i);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        account_lock.release(chain_id, address);
        account_lock.release(chain_id, address);
        account_lock.release(chain_id, address);
    }
}