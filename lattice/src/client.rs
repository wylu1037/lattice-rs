use std::error::Error;

use async_trait::async_trait;
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use serde_json::json;

use model::block::DBlock;
use model::LatticeError;
use model::receipt::Receipt;

use crate::constants::JSON_RPC;

/// 定义一个异步的客户端trait
#[async_trait]
pub trait LatticeClient {
    async fn send(&self, message: &str) -> Result<String, Box<dyn Error>>;
}

/// Json-Rpc请求体
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcBody {
    id: u32,
    #[serde(rename = "jsonRpc")]
    json_rpc: String,
    method: String,
    params: Vec<serde_json::Value>,
}

impl JsonRpcBody {
    pub fn new(method: String, params: Vec<serde_json::Value>) -> Self {
        JsonRpcBody {
            id: 1,
            json_rpc: JSON_RPC.to_string(),
            method,
            params,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response<T> {
    #[serde(rename = "jsonRpc")]
    json_rpc: String,
    id: u32,
    result: Option<T>,
    error: Option<JsonRpcError>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct JsonRpcError {
    code: u16,
    message: String,
}

/// HTTP客户端
pub struct HttpClient {
    client: Client,
    pub ip: String,
    pub port: u16,
    url: String,
}

impl HttpClient {
    pub fn new(ip: &str, port: u16) -> Self {
        HttpClient {
            client: Client::new(),
            ip: ip.to_string(),
            port,
            url: format!("http://{}:{}", ip, port),
        }
    }

    /// # 查询最新的守护区块信息
    /// ## Parameters
    ///
    /// ## Returns
    /// + DBlock
    pub async fn get_current_daemon_block(&self) -> Box<DBlock> {
        let body = JsonRpcBody::new("latc_getCurrentDBlock".to_string(), vec![]);
        let message = serde_json::to_string(&body).unwrap();
        let response = self.send(message.as_str()).await.unwrap();
        let response: Response<DBlock> = serde_json::from_str(&response).unwrap();
        Box::new(response.result.unwrap())
    }

    /// # 发送已签名的交易
    pub async fn send_raw_tx() {}

    /// # 查询交易回执
    /// ## Parameters
    /// + `hash: &str`: 交易哈希，示例：`0xe8df1f1e250cd0eac75eee3f8733e26e9422ef5ea88650ab54498cd8e4928144`
    ///
    /// ## Returns
    /// + Box<Receipt>
    pub async fn get_receipt(&self, hash: &str) -> Result<Box<Receipt>, Box<dyn Error>> {
        let body = JsonRpcBody::new("latc_getReceipt".to_string(), vec![json!(hash)]);
        let message = serde_json::to_string(&body).unwrap();
        let response = self.send(message.as_str()).await.unwrap();
        let response: Response<Receipt> = serde_json::from_str(&response)?;
        match response.result {
            Some(receipt) => Ok(Box::new(receipt)),
            None => match response.error {
                Some(err) => Err(Box::new(LatticeError::ReceiptNotFound)),
                None => Err(Box::new(LatticeError::InternalError))
            }
        }
    }
}

#[async_trait]
impl LatticeClient for HttpClient {
    async fn send(&self, message: &str) -> Result<String, Box<dyn Error>> {
        let res = self.client.post(&self.url)
            .body(message.to_string())
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?
            .text()
            .await?;
        Ok(res)
    }
}

/// Websocket客户端
pub struct WsClient {}

#[cfg(test)]
mod tests {
    use crate::client::HttpClient;

    #[tokio::test]
    async fn test_get_current_daemon_block() {
        let client = HttpClient::new("192.168.1.185", 13000);
        let response = client.get_current_daemon_block().await;
        println!("{:?}", response);
    }

    #[tokio::test]
    async fn test_get_receipt() {
        let client = HttpClient::new("192.168.1.185", 13000);
        let response = client.get_receipt("0xe8df1f1e250cd0eac75eee3f8733e26e9422ef5ea88650ab54498cd8e4928144").await;
        match response {
            Ok(receipt) => println!("{:?}", receipt),
            Err(err) => println!("{:?}", err.to_string())
        }
    }
}