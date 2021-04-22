use serde::{Serialize, Deserialize};
use siderite::protocol::Timestamp;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserID(String);

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
pub struct LoginReply {
    #[serde(rename = "_id")]
    pub id: UserID,
    pub token: String,
    pub expires: Timestamp,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "t")]
pub enum Room {
    #[serde(rename = "d")]
    Direct {
        #[serde(rename = "_id")]
        id: String,
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
        ro: bool,       
    },
    #[serde(rename = "l")]
    LiveChat {
        #[serde(rename = "_id")]
        id: String
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
    title: String,
}

#[derive(Debug, Deserialize)]
pub struct RoomEventData {
    pub msg: String,
    pub rid: String,
    #[serde(default)]
    pub t: Option<String>,
    pub u: ShortUser,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
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
                        muted: vec![] });
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

