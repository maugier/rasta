use serde::{Serialize, Deserialize};
use serde_json::{Map, Value};
use siderite::protocol::Timestamp;
use log::debug;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserID(String);

impl PartialEq<str> for UserID {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
} 

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MessageID(String);

impl MessageID {
    pub fn new() -> Self {
        let mut x = vec![0; 16];
        for x in &mut x {
            *x = fastrand::alphanumeric() as u8;
        }
        MessageID(String::from_utf8(x).unwrap())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Presence {
    Online,
    Busy,
    Away,
    Offline,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "_id")] pub id: UserID,
    pub created_at: Timestamp,
    pub roles: Vec<String>,
    #[serde(rename = "type")] pub usertype: String,
    pub active: bool,
    pub username: Option<String>,
    pub name: Option<String>,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortUser {
    #[serde(rename = "_id")]
    pub id: UserID,
    pub username: String,
    #[serde(rename = "name")]
    pub realname: String,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub status: Option<Presence>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct LoginReply {
    pub id: UserID,
    pub token: String,
    pub token_expires: Timestamp,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "t")]
pub enum Room {
    #[serde(rename = "d")]
    Direct {
        #[serde(rename = "_id")]
        id: String,
        #[serde(default, skip_serializing_if="Option::is_none")]
        lm: Option<Timestamp>
    },
    #[serde(rename = "c")]
    Chat {
        #[serde(rename = "_id")]
        id: String,
        name: String,
        //#[serde(rename = "u")]
        //creator: User,
        #[serde(default, skip_serializing_if="Option::is_none")]
        topic: Option<String>,
        #[serde(default)]
        muted: Vec<String>,

        #[serde(default, skip_serializing_if="Option::is_none")]
        lm: Option<Timestamp>,
    },
    #[serde(rename = "p")]
    Private {
        #[serde(rename = "_id")]
        id: String,
        name: String,
        //#[serde(rename = "u")]
        //creator: User,
        #[serde(default, skip_serializing_if="Option::is_none")]
        topic: Option<String>,
        #[serde(default)]
        muted: Vec<String>,

        #[serde(default, skip_serializing_if="Option::is_none")]
        lm: Option<Timestamp>,

        ro: bool,       
    },
    #[serde(rename = "l")]
    LiveChat {
        #[serde(rename = "_id")]
        id: String,

        #[serde(default, skip_serializing_if="Option::is_none")]
        lm: Option<Timestamp>,
    },
}

impl Room {
    pub fn id(&self) -> &str {
        match self {
            Room::Chat{id,..} => id,
            Room::Direct{id,..} => id,
            Room::LiveChat{id,..} => id,
            Room::Private{id,..} => id,
        }
    }

    fn lm(&mut self) -> &mut Option<Timestamp> {
        match self {
            Room::Chat{lm,..} => lm,
            Room::Direct{lm,..} => lm,
            Room::LiveChat{lm,..} => lm,
            Room::Private{lm,..} => lm,
        }
    }

    pub fn is_timestamp_fresh(&mut self, ts: Timestamp) -> bool {
        let lm = self.lm();
        debug!("Message timestamp = {:?}, room timestamp = {:?}", ts, lm);
        if lm.map(|ts2| ts2 > ts).unwrap_or(false) {
            false
        } else {
            *lm = Some(ts);
            true
        }
    }

}
//TODO proper timestamp serde

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomExtraInfo {
    pub room_name: Option<String>,
    pub room_participant: bool,
    pub room_type: char,
}
#[derive(Debug, Deserialize)]
pub struct Attachment {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RoomEventData {
    #[serde(rename="_id")]
    pub id: MessageID,
    pub msg: String,
    pub rid: String,
    pub ts: Timestamp,
    #[serde(default)]
    pub t: Option<String>,
    pub u: ShortUser,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(default, skip_serializing_if="serde_json::map::Map::is_empty")]
    pub reactions: Map<String, Value>,
}
#[derive(Debug, Deserialize)]
pub struct RoomEvent {
    pub args: (RoomEventData ,RoomExtraInfo)
}

#[derive(Debug, Deserialize)]
pub struct ShortRoom {
    #[serde(rename="_id")] pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Spotlight {
    pub users: Vec<ShortUser>,
    pub rooms: Vec<ShortRoom>,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn serde_presence() {
        assert_eq!( &serde_json::to_string(&Presence::Online).unwrap(), "\"online\"" );
    }

    #[test]
    fn deserialize_sample_room() {
        let room: Room = serde_json::from_str(r#"{"_id":"GENERAL","t":"c","name":"general","usernames":[],"usersCount":36,"default":true,"_updatedAt":{"$date":1618846200120},"lm":{"$date":1618836010099},"lastMessage":{"_id":"hCJtirxRuGAgN5Bom","rid":"GENERAL","msg":"Nice :moneybag:","ts":{"$date":1618836010099},"u":{"_id":"sSKs766qYEiKF8tss","username":"qwerty","name":"qwerty"},"_updatedAt":{"$date":1618836010128},"mentions":[],"channels":[]}}"#)
            .unwrap();

        assert_eq!(room,
            Room::Chat {id: "GENERAL".to_string(), 
                        name: "general".to_string(), 
                        topic: None,
                        muted: vec![],
                        lm: None });
    }

    #[test]
    fn deserialize_public_message() {

        let source = r#"
        {
            "args": [
              {
                "_id": "BFa2866ehEnpHCmsc",
                "_updatedAt": {
                  "$date": 1618995166590
                },
                "channels": [],
                "mentions": [],
                "msg": "tralala pouet",
                "rid": "fjGcXmddo5h8sp85n",
                "ts": {
                  "$date": 1618995166553
                },
                "u": {
                  "_id": "hza29JX8SbnwqJwwh",
                  "name": "syn",
                  "username": "syn"
                }
              },
              {
                "roomName": "test",
                "roomParticipant": true,
                "roomType": "c"
              }
            ],
            "eventName": "__my_messages__"
          }      
        "#;
        serde_json::from_str::<RoomEvent>(source).unwrap();
    }

}

