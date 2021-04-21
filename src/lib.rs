use anyhow::{Result, anyhow};
use ring::digest::{Digest, SHA256, digest};
use schema::Room;
use siderite::Connection;
use serde_json::{self, json, Value};
use futures::Stream;
pub use siderite::protocol::ServerMessage;

pub mod schema;
pub mod session;

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

pub struct Handle {
    handle: siderite::connection::Handle,
}


impl Rasta {

    pub async fn connect(hostname: &str) -> Result<Self> {

        let url = format!("wss://{}/websocket", hostname);
        Ok(Self { connection: Connection::connect(&url).await? })
    }

    pub fn handle(&self) -> Handle {
        Handle { handle: self.connection.handle() }
    }

    pub async fn login(&mut self, creds: Credentials) -> Result<Value> {
        self.connection.call("login".to_string(), vec![creds.json()]).await
    }

    pub async fn rooms(&mut self) -> Result<Vec<Room>> {
        let reply = self.connection.call("rooms/get".to_string(), vec![]).await?;
        Ok(serde_json::from_value(reply)?)
    }

    pub fn stream(&mut self) -> &mut impl Stream<Item=ServerMessage> {
        self.connection.stream() 
    }

    pub async fn recv(&mut self) -> Result<ServerMessage> {
        self.connection.recv().await.ok_or(anyhow!("fail"))
    }

    pub async fn subscribe_room(&mut self, room_id: String) -> Result<()> {
        let id = room_id.clone();
        Ok(self.connection.subscribe(id, "stream-room-messages".to_string(),
         vec![ Value::String(room_id) , Value::Bool(false)])
            .await?)
    }

    pub async fn subscribe_my_messages(&mut self) -> Result<()> {
        self.subscribe_room("__my_messages__".to_string()).await
    }

}

fn random_id(buf: &mut [u8]) {
    for b in buf.iter_mut() {
        *b = fastrand::alphabetic() as u8;
    }
}

impl Handle {
    pub async fn send_message(&mut self, room: &Room, msg: String) -> Result<()> {
        let mut id = vec![0; 12];
        random_id(&mut id);
        let id = String::from_utf8(id)?;
        self.handle.call("sendMessage".to_string(), vec![json!(
            { "_id": id, "rid": room.id().to_string(), "msg": msg }
        )]).await?;
        Ok(())
    }

    pub async fn create_direct(&mut self, user: String) -> Result<Room> { 
        self.handle.call("createDirectMessage".into()
                                      , vec![user.into()])
            .await?
            .as_object_mut()
            .and_then(|o| o.get_mut("rid"))
            .and_then(|v| 
                if let Value::String(id) = v.take() {
                    Some(Room::Direct { id })
                } else { None }
            )
            .ok_or(anyhow!("malformed createDirectMessage reply"))
    }

}