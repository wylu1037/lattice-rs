use std::fmt::Formatter;
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::{MapAccess, SeqAccess, Visitor};
use crate::types::bytes::{Bytes, deserialize_bytes};

/// Vec<Item>为匿名字段
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct RawAbi(Vec<Item>);

impl IntoIterator for RawAbi {
    type Item = Item;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

struct RawAbiVisitor;

impl<'de> Visitor<'de> for RawAbiVisitor {
    type Value = RawAbi;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a sequence or map with `abi` key")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let mut vec = Vec::new();

        while let Some(element) = seq.next_element()? {
            vec.push(element);
        }

        Ok(RawAbi(vec))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
        let mut vec = None;

        while let Some(key) = map.next_key::<String>()? {
            if key == "abi" {
                vec = Some(RawAbi(map.next_value::<Vec<Item>>()?));
            } else {
                map.next_value::<serde::de::IgnoredAny>()?;
            }
        }

        vec.ok_or_else(|| serde::de::Error::missing_field("abi"))
    }
}

impl<'de> Deserialize<'de> for RawAbi {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_any(RawAbiVisitor)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    #[serde(default)]
    pub inputs: Vec<Component>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_mutability: Option<String>,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub outputs: Vec<Component>,
    // required to satisfy solidity events
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anonymous: Option<bool>,
}

/// Either an input/output or a nested component of an input/output
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    #[serde(rename = "internalType", default, skip_serializing_if = "Option::is_none")]
    pub internal_type: Option<String>,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub components: Vec<Component>,
    /// Indexed flag. for solidity events
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexed: Option<bool>,
}

/// Represents contract ABI input variants
#[derive(Deserialize)]
#[serde(untagged)]
pub enum JsonAbi {
    /// json object input as `{"abi": [...], "bin": "..."}`
    Object(AbiObject),
    /// json array input as `[]`
    #[serde(deserialize_with = "deserialize_abi_array")]
    Array(RawAbi),
}

/// implement JsonAbi
impl JsonAbi {
    /// Returns the bytecode object
    pub fn bytecode(&self) -> Option<Bytes> {
        match self {
            JsonAbi::Object(abi) => abi.bytecode.clone(),
            JsonAbi::Array(_) => None,
        }
    }

    /// Returns the deployed bytecode object
    pub fn deployed_bytecode(&self) -> Option<Bytes> {
        match self {
            JsonAbi::Object(abi) => abi.deployed_bytecode.clone(),
            JsonAbi::Array(_) => None,
        }
    }
}

fn deserialize_abi_array<'de, D>(deserializer: D) -> Result<RawAbi, D::Error>
    where
        D: Deserializer<'de>,
{
    deserializer.deserialize_seq(RawAbiVisitor)
}

/// Contract ABI and optional bytecode as JSON object
pub struct AbiObject {
    pub abi: RawAbi,
    pub bytecode: Option<Bytes>,
    pub deployed_bytecode: Option<Bytes>,
}

impl<'de> Deserialize<'de> for AbiObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_map(AbiObjectVisitor)
    }
}

struct AbiObjectVisitor;

impl<'de> Visitor<'de> for AbiObjectVisitor {
    type Value = AbiObject;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a sequence or map with `abi` key")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
    {
        let mut abi = None;
        let mut bytecode = None;
        let mut deployed_bytecode = None;

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Bytecode {
            Object { object: Bytes },
            Bytes(Bytes),
        }

        impl Bytecode {
            fn into_bytes(self) -> Option<Bytes> {
                let bytecode = match self {
                    Bytecode::Object { object } => object,
                    Bytecode::Bytes(bytes) => bytes,
                };
                if bytecode.is_empty() {
                    None
                } else {
                    Some(bytecode)
                }
            }
        }

        /// represents nested bytecode objects of the `evm` value
        #[derive(Deserialize)]
        struct EvmObj {
            bytecode: Option<Bytecode>,
            #[serde(rename = "deployedBytecode")]
            deployed_bytecode: Option<Bytecode>,
        }

        struct DeserializeBytes(Bytes);

        impl<'de> Deserialize<'de> for DeserializeBytes {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
            {
                Ok(DeserializeBytes(deserialize_bytes(deserializer)?.into()))
            }
        }

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "abi" => {
                    abi = Some(RawAbi(map.next_value::<Vec<Item>>()?));
                }
                "evm" => {
                    if let Ok(evm) = map.next_value::<EvmObj>() {
                        bytecode = evm.bytecode.and_then(|b| b.into_bytes());
                        deployed_bytecode = evm.deployed_bytecode.and_then(|b| b.into_bytes())
                    }
                }
                "bytecode" | "byteCode" => {
                    bytecode = map.next_value::<Bytecode>().ok().and_then(|b| b.into_bytes());
                }
                "deployedbytecode" | "deployedBytecode" => {
                    deployed_bytecode =
                        map.next_value::<Bytecode>().ok().and_then(|b| b.into_bytes());
                }
                "bin" => {
                    bytecode = map
                        .next_value::<DeserializeBytes>()
                        .ok()
                        .map(|b| b.0)
                        .filter(|b| !b.0.is_empty());
                }
                "runtimebin" | "runtimeBin" => {
                    deployed_bytecode = map
                        .next_value::<DeserializeBytes>()
                        .ok()
                        .map(|b| b.0)
                        .filter(|b| !b.0.is_empty());
                }
                _ => {
                    map.next_value::<serde::de::IgnoredAny>()?;
                }
            }
        }

        let abi = abi.ok_or_else(|| serde::de::Error::missing_field("abi"))?;
        Ok(AbiObject { abi, bytecode, deployed_bytecode })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 解析ABI
    #[test]
    fn can_parse_raw_abi() {
        const ABI_STR: &str = include_str!("./verifier_abi.json");
        let abi = serde_json::from_str::<RawAbi>(ABI_STR).unwrap();
        println!("{:?}", abi)
    }
}