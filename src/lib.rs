use anyhow::{Result, anyhow};
use ring::digest::{Digest, SHA256, digest};
use schema::Room;
use siderite::{Connection, protocol::ServerMessage};
use serde_json::{self, json, Value};

pub mod schema;

#[derive(Debug)]
pub enum Credentials {
    Clear { user: String, password: String },
    Token(String),
}

trait HexDigest {
    fn hexdigest(&self) -> String;
}

impl HexDigest for Digest {
    fn hexdigest(&self) -> String {
        use std::fmt::Write;
        let v = self.as_ref();
        let mut s = String::with_capacity(2 * v.len());
        for byte in v {
            write!(s, "{:0x}", *byte).unwrap();
        }
        s 
    }
}

impl Credentials {
    fn json(self) -> Value {
        match self {
            Self::Clear { user, password } => {
                let digest = digest(&SHA256, password.as_bytes())
                    .hexdigest();
                json!({
                    "user": {"username": user},
                    "password: ": {
                        "algorithm": "sha-256",
                        "digest": digest
                    }
                })
            },
            Self::Token(tok) => {
                json!({"resume": tok})
            }
        }
    }
}


pub struct Rasta {
    connection: Connection,
}

impl Rasta {

    pub async fn connect(hostname: &str) -> Result<Self> {

        let url = format!("wss://{}/websocket", hostname);
        Ok(Self { connection: Connection::connect(&url).await? })
    }

    pub async fn login(&mut self, creds: Credentials) -> Result<Value> {
        self.connection.call("login".to_string(), vec![creds.json()]).await
    }

    pub async fn rooms(&mut self) -> Result<Vec<Room>> {
        let reply = self.connection.call("rooms/get".to_string(), vec![]).await?;
        Ok(serde_json::from_value(reply)?)
    }

    pub async fn recv(&mut self) -> Result<ServerMessage> {
        self.connection.recv().await.ok_or(anyhow!("fail"))
    }

}