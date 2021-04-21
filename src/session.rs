use std::collections::HashMap;
use crate::{Handle, Rasta, schema::Room};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone)]
pub struct Session {
    pub rooms: Vec<Room>,
    pub directs: HashMap<String,String>,
}


impl Session {
    pub async fn from(client: &mut Rasta) -> Result<Self> {
        Ok(Self{ rooms: client.rooms().await?, directs: HashMap::new() })
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

    pub async fn direct_room(&mut self, handle: &mut Handle, user: &str) -> Result<&Room> {
        if let Some(id) = self.directs.get(user) {
            self.room_by_id(id).ok_or(anyhow!(""))     
        } else {
            let room = handle.create_direct(user.to_string()).await?;
            self.directs.insert(user.into(), room.id().into());
            self.rooms.push(room);
            Ok(self.rooms.last().unwrap())
        }
    
    }

}