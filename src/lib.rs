use anyhow::{Error, Result, anyhow, bail};
use ring::digest::{Digest, SHA256, digest};
use siderite::{Connection, protocol::Message};
use serde_json::{self, json, Value};

pub enum Credentials {
    Clear { user: String, password: String }
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

    pub async fn recv(&mut self) -> Result<Message> {
        self.connection.recv().await.ok_or(anyhow!("fail"))
    }

}