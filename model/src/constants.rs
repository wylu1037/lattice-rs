pub const ADDRESS_TITLE: &str = "zltc_";

/// 零地址，由以太坊的零地址`0x0000000000000000000000000000000000000000`转换的
pub const ZERO_ZLTC_ADDRESS: &str = "zltc_QLbz7JHiBTspS962RLKV8GndWFwjA5K66";

/// 零哈希
pub const ZERO_HASH_STRING: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";

/// 压缩公钥的字节长度，第一个字节为02或03，用来表示y坐标的奇偶性
pub const COMPRESSED_PUBLIC_KEY_LENGTH: usize = 33;

/// 未压缩公钥的字节长度，第一个字节固定未04，表示这是一个未压缩的公钥
pub const UNCOMPRESSED_PUBLIC_KEY_LENGTH: usize = 65;

/// 未压缩公钥的字节长度，去除一个前缀字节04
pub const PUBLIC_KEY_LENGTH: usize = 64;

/// 私钥的字节长度
pub const PRIVATE_KEY_LENGTH: usize = 32;

pub const PREFIX_OF_HEX: &str = "0x";