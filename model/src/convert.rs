use num_bigint::BigUint;

trait IntoBigUint {
    fn into_big_uint(self) -> BigUint;
}

macro_rules! impl_into_big_uint {
    ($($t:ty)*) => {
        $(
            impl IntoBigUint for $t {
                fn into_big_uint(self) -> BigUint {
                    BigUint::from(self)
                }
            }
        )*
    };
}

impl_into_big_uint! {u8 u16 u32 u64 u128 usize}

pub fn option_number_to_vec<T>(num: Option<T>) -> Vec<u8>
    where
        T: IntoBigUint + Copy
{
    match num {
        Some(num) => {
            num.into_big_uint().to_bytes_be()
        }
        None => Vec::new()
    }
}

pub fn number_to_vec<T>(num: T) -> Vec<u8>
    where
        T: IntoBigUint + Copy
{
    num.into_big_uint().to_bytes_be()
}


#[cfg(test)]
mod tests {
    use crate::convert::option_number_to_vec;

    #[test]
    fn convert() {
        let num_some_u8 = Some(18u8);
        let num_some_u16 = Some(12345u16);
        let num_some_u32 = Some(123456789u32);
        let num_some_u64 = Some(1234567890123456789u64);
        let num_some_u128 = Some(123456789012345678901234567890u128);
        let num_none: Option<u128> = None;

        let vec_some_u8 = option_number_to_vec(num_some_u8);
        let vec_some_u16 = option_number_to_vec(num_some_u16);
        let vec_some_u32 = option_number_to_vec(num_some_u32);
        let vec_some_u64 = option_number_to_vec(num_some_u64);
        let vec_some_u128 = option_number_to_vec(num_some_u128);
        let vec_none = option_number_to_vec(num_none);

        println!("vec_some_u8: {:?}", vec_some_u8);
        println!("vec_some_u16: {:?}", vec_some_u16);
        println!("vec_some_u32: {:?}", vec_some_u32);
        println!("vec_some_u64: {:?}", vec_some_u64);
        println!("vec_some_u128: {:?}", vec_some_u128);
        println!("vec_none: {:?}", vec_none);
    }
}