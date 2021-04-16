use anyhow::{Error, Result, anyhow, bail};
use serde::{Serialize, Deserialize};
use serde_json;
use serde_json::{json, Value};
use futures::stream::StreamExt;
use futures::sink::SinkExt;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use async_tungstenite::tungstenite::Message;
use slab::Slab;

mod protocol;

macro_rules! json_msg {
    ($data:tt) => { async_tungstenite::tungstenite::Message::Text((json!($data)).to_string()) };
}

pub enum Credentials {
    Clear { user: String, password: String }
}

#[derive(Debug)]
struct Method {
    name: String,
    params: Value,
    result: oneshot::Sender<MethodResult>,
}

type MethodResult = std::result::Result<Value,Value>;

pub struct Rasta {
    stream: mpsc::Receiver<Value>,
    rpc: mpsc::Sender<Method>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RoomId(String);


async fn json_recv<E: Into<Error>,S: Unpin + StreamExt<Item = Result<Message,E>>>(stream: &mut S) -> Result<serde_json::Value> {
    if let Message::Text(msg) = stream.next()
                            .await
                            .ok_or(anyhow!("stream was empty"))?
                            .map_err(Into::into)?
    {
        Ok(serde_json::from_str(&msg)?)
    } else {
        Err(anyhow!("Non-text message received"))
    }
          
}

impl Rasta {

    pub async fn connect(hostname: &str, creds: Credentials) -> Result<Self> {

        let url = format!("wss://{}/websocket", hostname);
        let ping: Value = json!( {"msg": "ping" });
        let pong: Value = json!( {"msg": "pong" });

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

        let _server_version = json_recv(&mut stream).await?;
        let _connected = json_recv(&mut stream).await?;

        let (down_tx, down_rx) = mpsc::channel(16);
        let (up_tx, up_rx) = mpsc::channel(16);

        let mut pending: Slab<oneshot::Sender<MethodResult>> = Slab::new();

        tokio::spawn(async move {

            while let Ok(msg) = json_recv(&mut stream).await {

                eprintln!("Received message {}", msg);

                let msgtype = match msg.get("msg")
                    .and_then(Value::as_str) {
                    None => { eprint!("malformed message (no msg field)"); continue },
                    Some(t) => t,       
                };

                match msgtype {
                    "ping" => {
                        eprintln!("Sending pong");
                        let reply = pong.to_string();
                        if let Err(e) = stream.send(Message::Text(reply)).await {
                            eprintln!("upstream send error: {}", e);
                            break
                        }
                    },
                    "result" => {
                        if let Some(id) = msg.get("id")
                                .and_then(Value::as_str)
                                .and_then(|s| s.parse().ok()) {

                            if !pending.contains(id) {
                                eprintln!("Invalid response id");
                                continue
                            }

                            let r: MethodResult = if let Some(v) = msg.get("result") {
                                Ok(v.clone())
                            } else if let Some(v) = msg.get("error") {
                                Err(v.clone())
                            } else {
                                Err("invalid rpc".into())
                            };

                            let callback = pending.remove(id);
                            if let Err(e) = callback.send(r) {
                                eprintln!("could not send {:?}", e);
                            }

                        } else {
                            eprintln!("message result without payload");
                            continue
                        }
                        
                    }
                    _ => {
                        eprintln!("Unsupported message type");
                    }
                }

            }
    
        });


        Ok(Self { stream: down_rx, rpc: up_tx })
    }

    pub async fn recv(&mut self) -> Option<Value> {
        self.stream.recv().await
    }

    pub async fn call(&mut self, name: String, params: Value) -> Result<Value> {
        let (tx, rx) = oneshot::channel();
        let request = Method { name, params, result: tx };
        self.rpc.send(request).await?;
        rx.await?.map_err(|e| anyhow!("RPC Call returned an error: {}", e))
    }

}