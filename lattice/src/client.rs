use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use reqwest::{Client, Url};
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;

use crypto::Transaction;
use model::block::{DBlock, LatestBlock};
use model::common::Address;
use model::Error;
use model::receipt::Receipt;

use crate::constants::JSON_RPC;

/// 定义一个异步的客户端trait
#[async_trait]
pub trait HttpRequest {
    async fn send(&self, message: &str, headers: HashMap<String, String>) -> Result<String, Error>;
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

    pub fn new_ws_monitor() -> String {
        let body = JsonRpcBody::new("latc_subscribe".to_string(), vec![json!("monitorData")]);
        serde_json::to_string(&body).unwrap()
    }

    pub fn new_ws_transaction_block() -> String {
        let body = JsonRpcBody::new("latc_subscribe".to_string(), vec![json!("newTBlock")]);
        serde_json::to_string(&body).unwrap()
    }

    pub fn new_ws_daemon_block() -> String {
        let body = JsonRpcBody::new("latc_subscribe".to_string(), vec![json!("newDBlock")]);
        serde_json::to_string(&body).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response<T> {
    #[serde(rename = "jsonRpc")]
    json_rpc: String,
    id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
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

    /// # 创建http的请求头
    ///
    /// ## 入参
    /// + `chai_id: u64`: 链ID，为0时不设置ChainID的请求头
    ///
    /// ## 出参
    /// + `HashMap<String, String>`
    fn new_headers(chain_id: u64) -> HashMap<String, String> {
        let mut chain_id_as_string = String::new();
        if chain_id > 0 {
            chain_id_as_string = chain_id.to_string();
        }
        let headers = HashMap::from([
            (CONTENT_TYPE.to_string(), String::from("application/json")),
            (String::from("ChainID"), chain_id_as_string)
        ]);
        headers
    }

    /// # 发送json-rpc请求
    ///
    /// ## 入参
    /// + `body: &JsonRpcBody`: 请求体
    /// + `headers: HashMap<String, String>`: 请求头
    ///
    /// ## 出参
    /// + `Result<T, Error>`
    async fn send_json_rpc_request<T>(&self, body: &JsonRpcBody, headers: HashMap<String, String>) -> Result<T, Error>
        where
            T: for<'a> Deserialize<'a>
    {
        let message = serde_json::to_string(&body)?;
        let response = self.send(message.as_str(), headers).await?;
        let response: Response<T> = serde_json::from_str(&response)?;
        let err_option = response.error;
        if let Some(err) = err_option {
            return Err(Error::custom(err.code as i32, format!("{}", err.message)));
        }
        response.result.ok_or(Error::new("结果为空"))
    }

    /// # 查询最新的守护区块信息
    ///
    /// ## Parameters
    /// + `chain_id: u64`: 链ID
    ///
    /// ## Returns
    /// + `Box<DBlock>`
    pub async fn get_latest_daemon_block(&self, chain_id: u64) -> Result<DBlock, Error> {
        let body = JsonRpcBody::new("latc_getCurrentDBlock".to_string(), vec![]);
        let result: Result<DBlock, Error> = self.send_json_rpc_request(&body, Self::new_headers(chain_id)).await;
        result
    }

    /// # 查询最新的区块（包括账户和守护区块的信息）
    ///
    /// ## 入参
    /// + `chain_id: u64`: 链ID
    /// + `addr: &Address`: 账户地址
    ///
    /// ## 出参
    /// + `Result<CurrentTDBlock, Error>`
    ///   + `Ok(CurrentTDBlock)`
    ///   + `Err(err)`
    pub async fn get_latest_block(&self, chain_id: u64, addr: &Address) -> Result<LatestBlock, Error> {
        let body = JsonRpcBody::new("latc_getCurrentTBDB".to_string(), vec![json!(addr.to_zltc_address())]);
        let result: Result<LatestBlock, Error> = self.send_json_rpc_request(&body, Self::new_headers(chain_id)).await;
        result
    }

    /// # 获取当前账户的最新的区块信息，包括pending中的交易
    ///
    /// ## 入参
    /// + `chain_id: u64`
    /// + `addr: &Address`
    ///
    /// ## 出参
    /// + `Result<CurrentTDBlock, Error>`
    ///   + `Ok(CurrentTDBlock)`
    ///   + `Err(err)`
    pub async fn get_latest_block_with_pending(&self, chain_id: u64, addr: &Address) -> Result<LatestBlock, Error> {
        let body = JsonRpcBody::new("latc_getPendingTBDB".to_string(), vec![json!(addr.to_zltc_address())]);
        let result: Result<LatestBlock, Error> = self.send_json_rpc_request(&body, Self::new_headers(chain_id)).await;
        result
    }

    /// # 发送已签名的交易
    ///
    /// ## 入参
    /// + `&self`:
    /// + `chain_id: u64`: 链ID
    /// + `signed_tx`: 已签名的交易
    ///
    /// ## 出参
    /// + `Result<String, Error>`
    ///   + `Ok(String)`
    ///   + `Err(err)`
    pub async fn send_raw_tx(&self, chain_id: u64, signed_tx: Transaction) -> Result<String, Error> {
        let body = JsonRpcBody::new("wallet_sendRawTBlock".to_string(), vec![json!(signed_tx.to_raw_tx())]);
        let result: Result<String, Error> = self.send_json_rpc_request(&body, Self::new_headers(chain_id)).await;
        result
    }

    /// # 预执行合约
    ///
    /// ## 入参
    /// + `&self`:
    /// + `chain_id: u64`: 链ID
    /// + `unsigned_tx`: 未签名的交易
    ///
    /// ## 出参
    /// + `Result<Receipt, Error>`
    ///   + `Ok(Receipt)`
    ///   + `Err(err)`
    pub async fn pre_call_contract(&self, chain_id: u64, unsigned_tx: Transaction) -> Result<Receipt, Error> {
        let body = JsonRpcBody::new("wallet_preExecuteContract".to_string(), vec![json!(unsigned_tx.to_raw_tx())]);
        let result: Result<Receipt, Error> = self.send_json_rpc_request(&body, Self::new_headers(chain_id)).await;
        result
    }

    /// # 查询交易回执
    ///
    /// ## Parameters
    /// + `chain_id: u64`: 链ID
    /// + `hash: &str`: 交易哈希，示例：`0xe8df1f1e250cd0eac75eee3f8733e26e9422ef5ea88650ab54498cd8e4928144`
    ///
    /// ## Returns
    /// + `Box<Receipt>`
    pub async fn get_receipt(&self, chain_id: u64, hash: &str) -> Result<Receipt, Error> {
        let body = JsonRpcBody::new("latc_getReceipt".to_string(), vec![json!(hash)]);
        let result: Result<Receipt, Error> = self.send_json_rpc_request(&body, Self::new_headers(chain_id)).await;
        result
    }
}

#[async_trait]
impl HttpRequest for HttpClient {
    async fn send(&self, message: &str, headers: HashMap<String, String>) -> Result<String, Error> {
        let mut header_map = HeaderMap::new();
        header_map.insert(
            HeaderName::from_str(CONTENT_TYPE.as_str()).unwrap(),
            HeaderValue::from_str("application/json").unwrap(),
        );
        for (k, v) in headers {
            let key = HeaderName::from_str(&k).unwrap();
            let value = HeaderValue::from_str(&v).unwrap();
            header_map.insert(key, value);
        }
        let res = self.client.post(&self.url)
            .body(message.to_string())
            .headers(header_map)
            .send()
            .await?
            .text()
            .await?;
        Ok(res)
    }
}

#[async_trait]
pub trait WsRequest {
    async fn send(&self, write: WsWrite, message: &str);
}

/// Websocket客户端
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct WsClient<'a> {
    ip: &'a str, // ip address
    port: u16, // websocket port
}

// type alias
type WsWrite = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WsRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

impl<'a> WsClient<'a> {
    pub fn new(ip: &'a str, port: u16) -> Self {
        WsClient {
            ip,
            port,
        }
    }

    /// 获取websocket连接地址
    pub fn get_ws_conn_url(&self) -> String {
        return format!("ws://{}:{}", self.ip, self.port);
    }

    /// 建立websocket连接
    async fn connect(&self) -> (WsWrite, WsRead) {
        let (ws_stream, _) = connect_async(Url::parse(self.get_ws_conn_url().as_str()).unwrap()).await.expect("Failed to build ws connect");
        let (write, read) = ws_stream.split();
        (write, read)
    }

    /// # 接收消息流
    /// ## Parameters
    /// + `mut read: WsRead`
    /// + `sender: Sender<String>`
    ///
    /// ## Returns
    async fn receive(mut read: WsRead, sender: Sender<String>) {
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

    /// # 从channel中消费消息
    /// ## Parameters
    /// + `mut receiver: Receiver<String>`: a channel receiver
    /// + `processor: F`: F is a closures, signature is Fn(String)
    ///
    /// ## Returns
    async fn consumer<F>(mut receiver: Receiver<String>, processor: F)
        where
            F: Fn(String) + Send + 'static,
    {
        while let Some(msg) = receiver.recv().await {
            processor(msg)
        }
    }

    /// # 断开websocket连接
    /// ## Parameters
    ///
    /// ## Returns
    /// + bool: 是否成功关闭websocket连接
    pub async fn disconnect(mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>) -> bool {
        let result = write.send(Message::Close(None)).await;
        match result {
            Ok(_) => true,
            Err(e) => {
                eprintln!("{}", e);
                false
            }
        }
    }
}

#[async_trait]
impl<'a> WsRequest for WsClient<'a> {
    /// # 发送消息
    /// ## Parameters
    /// + `mut write: WsWrite`: ws write
    /// + `message: &str`: 消息
    ///
    /// ## Returns
    async fn send(&self, mut write: WsWrite, message: &str) {
        let message = Message::Text(message.to_string());
        write.send(message).await.expect("Failed to send message");
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::mpsc;

    use model::common::Address;

    use crate::client::{HttpClient, JsonRpcBody, WsClient, WsRequest};

    const CHAIN_ID: u64 = 1;

    #[tokio::test]
    async fn test_get_current_daemon_block() {
        let client = HttpClient::new("192.168.1.185", 13000);
        let response = client.get_latest_daemon_block(CHAIN_ID).await;
        match response {
            Ok(block) => println!("{:?}", block),
            Err(err) => println!("{:?}", err)
        }
    }

    #[tokio::test]
    async fn test_get_receipt() {
        let client = HttpClient::new("192.168.1.185", 13000);
        let response = client.get_receipt(CHAIN_ID, "0x616bf03baa685df9fddeff4701f170b30176e54120df726142a534f8f2b51873").await;
        match response {
            Ok(receipt) => println!("{:?}", receipt),
            Err(err) => println!("{:?}", err.to_string())
        }
    }

    #[tokio::test]
    async fn test_get_current_tx_daemon_block() {
        let client = HttpClient::new("192.168.1.185", 13000);
        let response = client.get_latest_block(CHAIN_ID, &Address::new("zltc_RvRUFNUYCg2vsjHii713Gc9Y3VNauM46J")).await;
        match response {
            Ok(block) => println!("{:?}", block),
            Err(err) => println!("{:?}", err)
        }
    }

    #[tokio::test]
    async fn test_monitor_data() {
        // create multi-producer single-consumer channel
        let (sender, receiver) = mpsc::channel(10);
        let client = WsClient::new("192.168.1.185", 12999);

        let (write, read) = client.connect().await;

        let _send_handler = tokio::spawn(async move {
            client.send(write, JsonRpcBody::new_ws_monitor().as_str()).await;
        });
        let _receive_handler = tokio::spawn(async move {
            WsClient::receive(read, sender).await;
        });

        tokio::spawn(async move {
            WsClient::consumer(receiver, |msg| println!("START {}", msg)).await
        });

        tokio::time::sleep(Duration::from_secs(30)).await;
        println!("{:?}", "🎉🎉🎉");
    }

    #[tokio::test]
    async fn test_monitor_daemon_block() {}
}