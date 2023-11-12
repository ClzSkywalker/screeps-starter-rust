use std::collections::HashMap;

use log::{error, warn};
use screeps::{find, Creep, HasTypedId, ObjectId, SharedCreepProperties, Source};
use serde::{Deserialize, Serialize};

use crate::utils::{self, errorx::ScreepError};

// creep 管理器
pub struct SourceManager {
    pub room_item: HashMap<String, RoomSourceItem>,
}

impl SourceManager {
    pub fn new() -> Self {
        SourceManager {
            room_item: HashMap::new(),
        }
    }

    pub fn init(&mut self, room_creeps: Vec<RoomSourceItem>) {
        for ele in room_creeps {
            self.room_item.insert(ele.room_id.clone(), ele);
        }
    }

    pub fn find_and_bind_source(
        &mut self,
        room_id: String,
        creep: &Creep,
    ) -> Option<ObjectId<Source>> {
        if let Some(source_i) = self.find_source(room_id.clone(), creep) {
            if let Some(source) = source_i.resolve() {
                if self
                    .bind_source(room_id.clone(), source.id().to_string(), creep)
                    .is_ok()
                {
                    return Some(source_i);
                }
                warn!("source not found");
            }
            warn!("source not found");
        }
        None
    }

    pub fn find_source(&self, room_id: String, creep: &Creep) -> Option<ObjectId<Source>> {
        let room = match utils::find::find_room(room_id.clone()) {
            Ok(r) => r,
            Err(e) => {
                warn!("{:?}", e);
                return None;
            }
        };
        let mut source_list = Vec::new();
        for source in room.find(find::SOURCES_ACTIVE, None).iter() {
            let creep_ids = self.get_screeps(room_id.clone(), source.id().to_string());
            if creep_ids.contains(&creep.name().to_string()) {
                return Some(source.id());
            }
            if creep_ids.len() >= 2 {
                continue;
            }
            source_list.push(source.clone());
        }

        utils::find::get_near_site(creep, &source_list).map(|r| r.id())
    }

    pub fn bind_source(
        &mut self,
        room_id: String,
        source_id: String,
        creep: &Creep,
    ) -> anyhow::Result<()> {
        match self.room_item.get_mut(&room_id) {
            Some(r) => {
                r.bind_screep(source_id.clone(), creep.name().to_string());
                Ok(())
            }
            None => {
                error!("{:?}", ScreepError::RoomNotfound(room_id.clone()));
                Err(ScreepError::RoomNotfound(room_id.clone()).into())
            }
        }
    }

    // pub fn add_room(&mut self, item: RoomSourceItem) {
    //     self.room_item.insert(item.room_id.clone(), item);
    // }

    pub fn get_screeps(&self, room_id: String, source_id: String) -> Vec<String> {
        let mut res = Vec::new();
        if let Some(room) = self.room_item.get(&room_id) {
            if let Some(creep) = room.source_map.get(&source_id) {
                res = creep.clone();
            }
        }
        res
    }

    pub fn check(&mut self) {
        for ele in self.room_item.values_mut() {
            ele.check();
        }
    }

    pub fn get_memory(&self, room_id: String) -> Option<RoomSourceItem> {
        for (rid, value) in self.room_item.iter() {
            if rid != &room_id {
                continue;
            }
            return Some(value.clone());
        }
        None
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RoomSourceItem {
    pub room_id: String,
    pub source_map: HashMap<String, Vec<String>>,
}

impl RoomSourceItem {
    pub fn new(id: String) -> Self {
        RoomSourceItem {
            room_id: id,
            source_map: HashMap::new(),
        }
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        let room = match utils::find::find_room(self.room_id.clone()) {
            Ok(r) => r,
            Err(e) => {
                warn!("{:?}", e);
                return Err(e);
            }
        };

        for ele in utils::find::find_source_all(&room) {
            self.source_map.insert(ele.id().to_string(), Vec::new());
        }

        Ok(())
    }

    pub fn check(&mut self) {
        for (_, screeps) in self.source_map.iter_mut() {
            utils::remove_expire_screep(screeps);
            utils::remove_repeat_screep(screeps);
        }
    }

    pub fn bind_screep(&mut self, source_id: String, creep_id: String) {
        if let Some(screeps) = self.source_map.get_mut(&source_id) {
            if screeps.contains(&creep_id) {
                return;
            }
            screeps.push(creep_id);
        }
    }
}
