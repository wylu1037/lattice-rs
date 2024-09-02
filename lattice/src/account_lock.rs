use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

/// 定义账户锁的trait
pub trait AccountLockTrait {
    /// # 获取账户锁
    ///
    /// ## 入参
    /// + `chain_id: u64`: 链ID
    /// + `account_address: &str`: 账户地址
    ///
    /// ## 出参
    /// + `Arc<Mutex<()>>`: Mutex锁
    fn obtain(&self, chain_id: u64, account_address: &str) -> Arc<Mutex<()>>;

    /// # 减少账户持有的Mutex在HashMap中的引用次数，当引用次数为0时，释放Mutex在HashMap中的缓存
    ///
    /// ## 入参
    /// + `chain_id: u64`: 链ID
    /// + `account_address: &str`: 账户地址
    fn dec_ref(&self, chain_id: u64, account_address: &str);

    /// # 释放账户持有的Mutex在HashMap中的缓存
    ///
    /// ## 入参
    /// + `chain_id: u64`: 链ID
    /// + `account_address: &str`: 账户地址
    fn release(&self, chain_id: u64, account_address: &str);
}

pub struct DefaultAccountLock {
    /// 设计思路：
    /// + `RwLock` 允许并发读和独占写操作，通过write方法阻塞其它线程，保证对HashMap的操作是并发安全的
    /// + `Arc`（原子引用计数，Atomic Reference Counted）允许多个线程安全地共享同一个数据。通过`Arc::clone()`在多个线程间共享锁的所有权。
    /// + `Mutex` 是一种同步原语，用于在多线程环境中保护共享数据的访问。Mutex 提供了独占访问的机制，
    /// 确保同一时间只有一个线程可以访问被保护的数据。以此来保证一个账户的并发请求在服务端是串行执行的。
    /// + `usize`: 对同一账户的获取Mutex锁进行记录，获取一次，次数+1，释放时，次数-1，为0时，将账户的缓存在HashMap的Mutex锁删除掉。
    locks: RwLock<HashMap<String, (Arc<Mutex<()>>, usize)>>,
}

impl DefaultAccountLock {
    pub fn new() -> Self {
        DefaultAccountLock {
            locks: RwLock::new(HashMap::new())
        }
    }
}

impl AccountLockTrait for DefaultAccountLock {
    fn obtain(&self, chain_id: u64, account_address: &str) -> Arc<Mutex<()>> {
        let key = format!("{}_{}", chain_id, account_address);
        /// 当 RwLockWriteGuard 离开其作用域时，会自动释放锁，允许其他线程访问数据。
        let mut locks = self.locks.write().unwrap(); // 使用写锁阻塞其它线程

        let (lock, count) = locks.entry(key)
            .or_insert_with(|| (Arc::new(Mutex::new(())), 0));
        *count += 1;
        lock.clone() // 写锁离开作用域，自动释放锁
    }

    fn dec_ref(&self, chain_id: u64, account_address: &str) {
        println!("开始释放资源");
        let key = format!("{}_{}", chain_id, account_address);
        let mut locks = self.locks.write().unwrap(); // 使用写锁阻塞其它线程

        if let Some((_, count)) = locks.get_mut(&key) {
            println!("count: {}", count);
            if *count > 1 {
                *count -= 1;
            } else {
                println!("删除Mutex的缓存");
                locks.remove(&key);
            }
        } // RwLockWriteGuard<HashMap<...>> 离开作用域，自动释放锁
    }

    fn release(&self, chain_id: u64, account_address: &str) {
        println!("释放账户的全部资源");
        let key = format!("{}_{}", chain_id, account_address);
        let mut locks = self.locks.write().unwrap(); // 使用写锁阻塞其它线程

        if locks.contains_key(&key) {
            locks.remove(&key);
        }
    }
}

/*struct AccountLockGuard<'a> {
    account_lock: &'a dyn AccountLockTrait,
    chain_id: u64,
    account_address: String,
}

impl<'a> AccountLockGuard<'a> {
    fn new(account_lock: &'a dyn AccountLockTrait, chain_id: u64, account_address: &str) -> Self {
        AccountLockGuard {
            account_lock,
            chain_id,
            account_address: account_address.to_string(),
        }
    }
}

impl<'a> Drop for AccountLockGuard<'a> {
    fn drop(&mut self) {
        self.account_lock.release(self.chain_id, &self.account_address)
    }
}*/

/// 模拟耗时操作
fn handle_request(lock: Arc<Mutex<()>>, request_id: usize) {
    let _guard = lock.lock().unwrap();
    println!("Handling request {} for the account", request_id);
    sleep(Duration::from_secs(1)); // 模拟请求处理时间
    println!("Finished request {}", request_id);
}

/// Box 是 Rust 中的一种智能指针，用于在堆上分配数据并提供对该数据的所有权管理。它允许你在编译时确定大小的情况下，在堆上存储数据。
/// 1.在堆上分配数据
/// 2.递归数据结构
/// 3.动态分发与多态
/// 4.在栈上存储大量数据
/// 5.防止大型数据结构的拷贝
fn handle_locks(account_lock: Box<dyn AccountLockTrait>) {
    let chain_id = 1;
    let address = "zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi";

    let mut handles = vec![];

    for i in 0..3 {
        let lock = account_lock.obtain(chain_id, address);
        let handle = thread::spawn(move || handle_request(lock, i));
        handles.push(handle);
    }

    for handle in handles {
        account_lock.dec_ref(chain_id, address);
        handle.join().unwrap();
    }
    // account_lock.release(chain_id, address);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_account_lock() {
        let account_lock = DefaultAccountLock::new();
        handle_locks(Box::new(account_lock));
    }
}