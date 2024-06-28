/// 定义内置合约宏
#[macro_export]
macro_rules! impl_builtin_contract {
    ($builtin_contract:ident, $abi:expr, $address:expr) => {
        #[derive(serde::Deserialize, serde::Serialize, Debug)]
        pub struct $builtin_contract {
            /// 合约abi
            abi: String,
            /// 合约地址
            address: String,
        }

        impl $builtin_contract {
            /// # 初始化一个内置合约
            fn new() -> Self {
                $builtin_contract {
                    abi: $abi.to_string(),
                    address: $address.to_string(),
                }
            }
            
            /// # abi encode合约方法参数
            ///
            /// ## 入参
            /// + `fn_name: &str`
            /// + `args: Vec<Box<dyn std::any::Any>>`
            ///
            /// ## 出参
            /// + `String`: data
            fn encode_args(&self, fn_name: &str, args: Vec<Box<dyn std::any::Any>>) -> String {
                let abi = abi::Abi::new(&self.abi);
                abi.encode(fn_name, args)
            }
        }
    };
}
