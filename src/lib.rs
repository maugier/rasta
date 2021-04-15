use anyhow::{Error, Result, anyhow};
use serde::{Serialize, Deserialize};
use serde_json::json;
use futures::stream::StreamExt;
use futures::sink::SinkExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use async_tungstenite::tungstenite::Message;

macro_rules! json_msg {
    ($data:tt) => { async_tungstenite::tungstenite::Message::Text((json!($data)).to_string()) };
}

pub enum Credentials {
    Clear { user: String, password: String }
}

pub struct Rasta {
    stream: mpsc::Receiver<Message>
}

#[derive(Serialize, Deserialize, Debug)]
struct RoomId(String);

impl Rasta {

    pub async fn connect(hostname: &str, creds: Credentials) -> Result<Self> {

        let url = format!("wss://{}/websocket", hostname);

        let tlsconfig = {
            let mut tlsconfig = tokio_rustls::rustls::ClientConfig::new();
            tlsconfig.root_store = rustls_native_certs::load_native_certs()
                .map_err(|(_store, err)| err)?;
            Arc::new(tlsconfig)
        };

        let tls = tokio_rustls::TlsConnector::from(tlsconfig);

        let (mut stream, response) =
            async_tungstenite::tokio::connect_async_with_tls_connector(url, Some(tls)).await?;
        eprintln!("Got response from websocket: {:?}", response);

        stream.send(json_msg!({
            "msg": "connect",
            "version": "1",
            "support": ["1"]
        })).await?;

        //let server_id = stream.next().await.ok_or(anyhow!("did not receive server id"))?;

        let (down_tx, down_rx) = mpsc::channel(16);

        tokio::spawn(async move {

            while let Some(Ok(msg)) = stream.next().await {
                    if let Err(e) = down_tx.send(msg).await {
                        eprintln!("Tokio send error: {}", e);
                        break
                }
            }
    
        });


        Ok(Self { stream: down_rx })
    }

    pub async fn recv(&mut self) -> Option<Message> {
        self.stream.recv().await
    }

}