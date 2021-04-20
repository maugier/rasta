use serde::{Serialize, Deserialize};
use siderite::protocol::Timestamp;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub created_at: Timestamp,
    pub roles: Vec<String>,
    #[serde(rename = "type")]
    pub usertype: String,
    pub active: bool,
    pub username: Option<String>,
    pub name: Option<String>,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortUser {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
    pub name: String,
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
        #[serde(skip_serializing_if="Option::is_none")]
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
        #[serde(skip_serializing_if="Option::is_none")]
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

#[derive(Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: String,
    pub rid: String,
    pub msg: String,
    pub ts: Timestamp,
    pub u: ShortUser,
    pub updated: Timestamp,
}

#[cfg(test)]
mod tests {

    use super::*;

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

}