use serde::{Serialize, Deserialize};

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "_id")]
    id: String,
    created_at: Timestamp,
    roles: Vec<String>,
    #[serde(rename = "type")]
    usertype: String,
    active: bool,
    username: Option<String>,
    name: Option<String>,
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

type Timestamp = String; //TODO proper timestamp serde

#[derive(Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "_id")]
    id: String,
    rid: String,
    msg: String,
    ts: Timestamp,
    u: String,
    updated: Timestamp,
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