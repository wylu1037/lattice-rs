use std::any::Any;

use abi::abi::Abi;

use crate::builder::TransactionBuilder;

/// 预置合约
pub struct BuiltinContract<'a> {
    name: &'a str,
    abi: &'a str,
    address: &'a str,
}

impl<'a> BuiltinContract<'a> {
    fn new(name: &'a str, abi: &'a str, address: &'a str) -> Self {
        BuiltinContract {
            name,
            abi,
            address,
        }
    }

    fn encode_args(&self, fn_name: &str, args: Vec<Box<dyn Any>>) -> String {
        let abi = Abi::new(&self.abi);
        abi.encode(fn_name, args)
    }
}