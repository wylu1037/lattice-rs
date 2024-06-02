use std::error::Error;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use reqwest::{Client, Url};
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;

use model::block::DBlock;
use model::LatticeError;
use model::receipt::Receipt;

use crate::constants::JSON_RPC;

/// 定义一个异步的客户端trait
#[async_trait]
pub trait HttpRequest {
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
impl HttpRequest for HttpClient {
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

#[async_trait]
pub trait WsRequest {
    async fn send(&self, message: &str, sender: mpsc::Sender<String>);
}

/// Websocket客户端
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WsClient {
    ip: String,
    port: u16,
    url: String,
}

// type alias
type WsWrite = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WsRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

impl WsClient {
    pub fn new(ip: &str, port: u16) -> Self {
        WsClient {
            ip: ip.to_string(),
            port,
            url: format!("ws://{}:{}", ip, port),
        }
    }

    /// 建立websocket连接
    async fn connect_ws(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn Error>> {
        let (ws_stream, _) = connect_async(Url::parse(&self.url).unwrap()).await.expect("Failed to build ws connect");
        Ok(ws_stream)
    }

    /// 建立websocket连接
    async fn connect(&self) -> (WsWrite, WsRead) {
        let (ws_stream, _) = connect_async(Url::parse(&self.url).unwrap()).await.expect("Failed to build ws connect");
        let (mut write, mut read) = ws_stream.split();
        (write, read)
    }

    /// # 断开websocket连接
    /// ## Parameters
    ///
    /// ## Returns
    /// + bool: 是否成功关闭websocket连接
    pub async fn disconnect(mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>) -> bool {
        let result = write.send(Message::Close(None)).await;
        match result {
            Ok(result_) => true,
            Err(e) => {
                eprintln!("{}", e);
                false
            }
        }
    }
}

#[async_trait]
impl WsRequest for WsClient {
    async fn send(&self, message: &str, sender: mpsc::Sender<String>) {
        let (mut write, mut read) = self.connect().await;

        let message = Message::Text(message.to_string());
        write.send(message).await.expect("Failed to send message");

        while let Some(msg) = read.next().await {
            match msg {
                Ok(message) => {
                    let future = sender.send(message.to_string());
                    match future.await {
                        Ok(_) => println!("Success send message {} to channel", message),
                        Err(e) => println!("Failed send message to channel, err {}", e)
                    }
                }
                Err(e) => println!("Failed receive message, err {}", e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde_json::json;
    use tokio::sync::mpsc;

    use crate::client::{HttpClient, JsonRpcBody, WsClient, WsRequest};

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

    #[tokio::test]
    async fn test_monitor_data() {
        let (sender, mut receiver) = mpsc::channel(10);
        let body = JsonRpcBody::new("latc_subscribe".to_string(), vec![json!("monitorData")]);
        let message = serde_json::to_string(&body).unwrap();
        let send_handler = tokio::spawn(async move { WsClient::new("192.168.1.185", 12999).send(message.as_str(), sender).await });

        let consumer_handler = tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                println!("Start consumer channel message {}", msg);
            }
        });

        tokio::time::sleep(Duration::from_secs(10)).await;
        println!("{:?}", "🎉🎉🎉");
    }
}