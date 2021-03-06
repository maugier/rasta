use std::collections::HashMap;
use crate::{Handle, Rasta, schema::Room};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone)]
pub struct Session {
    rooms: Vec<Room>,
    directs: HashMap<String,String>,
}


impl Session {

    pub fn rooms(&self) -> &[Room] {
        &self.rooms
    }

    pub async fn from(client: &mut Rasta) -> Result<Self> {
        Ok(Self{ rooms: client.rooms().await?, directs: HashMap::new()})
    }

    pub fn room_by_id(&mut self, id: &str) -> Option<&mut Room> {
        for room in &mut self.rooms {
            if room.id() == id {
                return Some(room)
            }
        }
        None
    }

    pub fn room_by_name(&mut self, name: &str) -> Option<&mut Room> {
        let search = name;
        for room in &mut self.rooms {
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


    pub async fn direct_room(&mut self, handle: &mut Handle, user: &str) -> Result<&mut Room> {
        if let Some(id) = self.directs.get(user).cloned() {
            self.room_by_id(&id).ok_or(anyhow!(""))     
        } else {
            let room = handle.create_direct(user.to_string()).await?;
            self.directs.insert(user.into(), room.id().into());
            self.rooms.push(room);
            Ok(self.rooms.last_mut().unwrap())
        }
    }

    pub async fn room_by_target(&mut self, handle: &mut Handle, target: &str) -> Option<&mut Room> {
        match target.strip_prefix('#') {
            Some(chan) => self.room_by_name(chan),
            None => self.direct_room(handle, target).await.ok(),
        } 
    }

}