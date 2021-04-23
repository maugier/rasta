use anyhow::{Result, anyhow};
use ring::digest::{Digest, SHA256, digest};
use schema::{LoginReply, MessageID, Presence, Room, ShortUser, Spotlight};
use siderite::{Connection, connection::MethodResult};
use serde_json::{self, json, Value};
use futures::Stream;
use log::{debug};
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

    pub async fn login(&mut self, creds: Credentials) -> Result<Option<LoginReply>> {
        Ok(self.connection.call("login".to_string(), vec![creds.json()]).await?
            .ok()
            .map(serde_json::from_value)
            .transpose()?
        )
        
    }

    pub async fn rooms(&mut self) -> Result<Vec<Room>> {
        let reply = self.connection.call("rooms/get".to_string(), vec![]).await??;
        Ok(serde_json::from_value(reply)?)
    }

    pub fn stream(&mut self) -> &mut impl Stream<Item=ServerMessage> {
        self.connection.stream() 
    }

    pub async fn recv(&mut self) -> Result<ServerMessage> {
        self.connection.recv().await.ok_or(anyhow!("fail"))
    }

    pub async fn subscribe(&mut self, name: String, params: Vec<Value>) -> Result<()> {
        let mut id = vec![0; 10];
        random_id(&mut id);
        let id = String::from_utf8(id)?;
        self.connection.subscribe(id, name, params).await
    }

    pub async fn subscribe_room(&mut self, room_id: String) -> Result<()> {
        let id = room_id.clone();
        Ok(self.connection.subscribe(id, "stream-room-messages".to_string(),
         vec![ Value::String(room_id), Value::Bool(false) ])
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
    pub async fn send_message(&mut self, id: MessageID, room: &Room, msg: String) -> Result<()> {
        // Ignore result, we can't do anything about it anyway
        let _ = self.handle.call("sendMessage".to_string(), vec![json!(
            { "_id": id, "rid": room.id().to_string(), "msg": msg }
        )]).await?;
        Ok(())
    }

    pub async fn create_direct(&mut self, user: String) -> Result<Room> { 
        self.handle.call("createDirectMessage".into()
                                      , vec![user.into()])
            .await??
            .as_object_mut()
            .and_then(|o| o.get_mut("rid"))
            .and_then(|v| 
                if let Value::String(id) = v.take() {
                    Some(Room::Direct { id, lm: None }) //TODO check this
                } else { None }
            )
            .ok_or(anyhow!("malformed createDirectMessage reply"))
    }

    pub async fn set_default_status(&mut self, p: Presence) -> Result<()> {
        self.handle.call("UserPresence:setDefaultStatus".into(), vec![ serde_json::to_value(p)? ]).await??;
        Ok(())
    }

    pub async fn set_away(&mut self, away: bool) -> Result<()> {
        let method = "UserPresence:".to_string() +
            if away { "away" } else { "online" };

        self.handle.call(method, vec![]).await??;
        Ok(())
    }

    pub async fn set_room(&mut self, room: &Room, name: String, value: Value) -> Result<MethodResult> {
        self.handle.call("saveRoomSettings".into(), vec![ room.id().clone().into(), name.into(), value ]).await
    }

    pub async fn set_topic(&mut self, room: &Room, topic: Option<String>) -> Result<bool> {
        debug!("Setting topic of {} to: {:?}", room.id(), topic);
        let topic = topic.map(Value::String).unwrap_or(Value::Null);
        Ok(self.set_room(room, "roomTopic".into(), topic).await?.is_ok())
    }

    pub async fn get_room_users(&mut self, room: &Room) -> Result<Vec<ShortUser>> {
        let name: &str = if let Room::Chat { name, ..} = room { name } else { return Ok(vec![]) };
        let r = self.handle.call("getRoomUsers".into(), vec![ name.into() ])
            .await??;
        debug!("=== get_room_users\n{:?}\n\n", r);
        if let Value::Array(arr) = r {
            Ok(arr.into_iter().map(|v| Ok(serde_json::from_value(v)?)).collect::<Result<Vec<_>>>()?)
        } else {
            Err(anyhow!("getRoomUsers reply wasn't an array"))
        }
    }

    pub async fn lookup_room_id(&mut self, name: String) -> Result<Option<String>> {
        let params = vec![name.clone().into(), json!([]), json!({ "users": false, "rooms": true})];
        let response = self.handle.call("spotlight".into(), params)
            .await??;
        let data: Spotlight = serde_json::from_value(response)?;

        debug!("Room lookup result: {:?}", data);
        
        for room in data.rooms {
            if room.name == name {
                return Ok(Some(room.id))
            }
        }

        Ok(None)

    }

    pub async fn join_room(&mut self, rid: String, code: Option<String>) -> Result<bool> {
        debug!("Joining {}", rid);
        let rid: Value = rid.into();
        let params = match code {
            Some(code) => vec![rid, code.into()],
            None => vec![rid]
        };
        Ok(self.handle.call("joinRoom".into(), params).await?.is_ok())
    }

    pub async fn leave_room(&mut self, rid: String) -> Result<bool> {
        Ok(self.handle.call("leaveRoom".into(), vec![rid.into()]).await?.is_ok())
    }

}