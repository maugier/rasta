use crate::{Rasta, schema::Room};
use anyhow::Result;

pub struct Session {
    pub rooms: Vec<Room>
}


impl Session {
    pub async fn from(client: &mut Rasta) -> Result<Self> {
        Ok(Self{ rooms: client.rooms().await? })
    }

    pub fn room_by_id(&self, id: &str) -> Option<&Room> {
        let search = id;
        for room in &self.rooms {
            match room {
                Room::Chat { id, ..} if id == search => {
                    return Some(room);
                },
                Room::Direct { id, ..} if id == search => {
                    return Some(room);
                },
                Room::LiveChat { id, ..} if id == search => {
                    return Some(room);
                },
                Room::Private { id, ..} if id == search => {
                    return Some(room);
                },
                _ => {},
            }
        }

        None
    }

    pub fn room_by_name(&self, name: &str) -> Option<&Room> {
        let search = name;
        for room in &self.rooms {
            match room {
                Room::Chat { name, ..} if name == search => {
                    return Some(room);
                },
                Room::Private { name, ..} if name == search => {
                    return Some(room);
                },
                _ => {},
            }
        }
        None
    }


}