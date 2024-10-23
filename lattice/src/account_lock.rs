use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use log::info;

/// 定义账户锁的trait
pub trait AccountLockTrait: Sync + Send {
    /// # 获取账户锁
    ///
    /// ## 入参
    /// + `chain_id: u64`: 链ID
    /// + `account_address: &str`: 账户地址
    ///
    /// ## 出参
    /// + `Arc<Mutex<()>>`: Mutex锁
    fn obtain(&self, chain_id: u64, account_address: &str) -> Arc<Mutex<()>>;
}

pub struct DefaultAccountLock {
    /// 设计思路：
    /// + `RwLock` 允许`并发读`和`独占写`操作，通过`write`方法阻塞其它线程，保证对`HashMap`的操作是并发安全的
    /// + `Arc`（原子引用计数，Atomic Reference Counted）允许多个线程安全地共享同一个数据。通过`Arc::clone()`在多个线程间共享锁的所有权。
    /// + `Mutex` 是一种同步原语，用于在多线程环境中保护共享数据的访问。Mutex 提供了独占访问的机制，确保同一时间只有一个线程可以访问被保护的数据。以此来保证一个账户的并发请求在服务端是串行执行的。
    locks: RwLock<HashMap<String, Arc<Mutex<()>>>>,
}

/// 创建一个默认的账户锁
impl DefaultAccountLock {
    pub fn new() -> Self {
        DefaultAccountLock {
            locks: RwLock::new(HashMap::new()),
        }
    }
}

impl AccountLockTrait for DefaultAccountLock {
    fn obtain(&self, chain_id: u64, account_address: &str) -> Arc<Mutex<()>> {
        let key = format!("{}_{}", chain_id, account_address);
        // 当 RwLockWriteGuard 离开其作用域时，会自动释放锁，允许其他线程访问数据。
        let mut locks = self.locks.write().unwrap(); // 使用写锁阻塞其它线程

        let lock = locks
            .entry(key)
            .or_insert_with(|| Arc::new(Mutex::new(())));
        
        info!(
            "Lock obtained for account: {}", account_address
        );
        lock.clone() // 写锁离开作用域，自动释放锁
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::thread;
    use std::thread::sleep;
    use std::time::Duration;

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

        for i in 0..100 {
            let lock: Arc<Mutex<()>> = account_lock.obtain(chain_id, address);
            let handle = thread::spawn(move || handle_request(lock, i));
            handles.push(handle);
        }

        // wait for all threads to finish
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_account_lock_in_multi_thread() {
        let account_lock = DefaultAccountLock::new();
        handle_locks(Box::new(account_lock));
    }
}
