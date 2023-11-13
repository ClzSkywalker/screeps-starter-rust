use std::collections::HashMap;

use screeps::{
    find, look, Creep, HasPosition, HasTypedId, ObjectId, Room, SharedCreepProperties, Source,
};
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

    /// 发现并绑定资源
    ///
    /// * `room_id`:
    /// * `creep`:
    pub fn find_and_bind_source(
        &mut self,
        room_id: String,
        creep: &Creep,
    ) -> Option<ObjectId<Source>> {
        if let Some(source_i) = self.find_source(room_id.clone(), creep) {
            if let Some(source) = source_i.resolve() {
                if let Ok(r) = self.bind_source(room_id.clone(), source.id().to_string(), creep) {
                    if r.is_some() {
                        return Some(source_i);
                    }
                }
                return None;
            }
            log::warn!("source not found");
        }
        None
    }

    /// 发现资源
    ///
    /// * `room_id`:
    /// * `creep`:
    pub fn find_source(&self, room_id: String, creep: &Creep) -> Option<ObjectId<Source>> {
        let room = match utils::find::find_room(room_id.clone()) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("{:?}", e);
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

    /// 绑定资源
    ///
    /// * `room_id`:
    /// * `source_id`:
    /// * `creep`:
    pub fn bind_source(
        &mut self,
        room_id: String,
        source_id: String,
        creep: &Creep,
    ) -> anyhow::Result<Option<()>> {
        match self.room_item.get_mut(&room_id) {
            Some(r) => {
                if r.check_bind(source_id.clone()).is_none() {
                    return Ok(None);
                }
                r.bind_screep(source_id.clone(), creep.name().to_string());
                Ok(Some(()))
            }
            None => {
                log::error!("{:?}", ScreepError::RoomNotfound(room_id.clone()));
                Err(ScreepError::RoomNotfound(room_id.clone()).into())
            }
        }
    }

    /// 找到挖资源的creep
    ///
    /// * `room_id`:
    /// * `source_id`:
    pub fn get_screeps(&self, room_id: String, source_id: String) -> Vec<String> {
        let mut res = Vec::new();
        if let Some(room) = self.room_item.get(&room_id) {
            if let Some(creep) = room.source_map.get(&source_id) {
                res = creep.creeps.clone();
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

impl Default for SourceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RoomSourceItem {
    pub room_id: String,
    pub max_count: usize, // 挖掘资源的最大人数
    pub source_map: HashMap<String, SourceInfoItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfoItem {
    pub source_id: String,
    pub creeps: Vec<String>, // 挖掘的creep id
    pub max_count: usize,    // 最大的source 挖掘人数
}

impl SourceInfoItem {
    pub fn new(room: &Room, item: &Source) -> Self {
        let count = utils::find::get_area_range(room, look::TERRAIN, item.pos(), 1).len();
        let max_count: usize = 12 - count;
        log::info!("max_count:{}-{}", max_count, count);
        Self {
            source_id: item.id().to_string(),
            creeps: Vec::default(),
            max_count,
        }
    }
}

impl RoomSourceItem {
    pub fn new(id: String) -> Self {
        RoomSourceItem {
            room_id: id,
            source_map: HashMap::new(),
            ..Default::default()
        }
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        let room = match utils::find::find_room(self.room_id.clone()) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("{:?}", e);
                return Err(e);
            }
        };
        for ele in utils::find::find_source_all(&room) {
            self.source_map
                .insert(ele.id().to_string(), SourceInfoItem::new(&room, &ele));
        }
        self.max_count += self.source_map.values().map(|x| x.max_count).sum::<usize>();

        Ok(())
    }

    pub fn check(&mut self) {
        for (_, screeps) in self.source_map.iter_mut() {
            utils::remove_expire_screep(&mut screeps.creeps);
            utils::remove_repeat_screep(&mut screeps.creeps);
        }
    }
    pub fn check_bind(&mut self, source_id: String) -> Option<()> {
        if let Some(screeps) = self.source_map.get(&source_id) {
            if screeps.creeps.len() >= screeps.max_count {
                return None;
            }
            return Some(());
        }
        None
    }
    pub fn bind_screep(&mut self, source_id: String, creep_id: String) {
        if let Some(screeps) = self.source_map.get_mut(&source_id) {
            if screeps.creeps.contains(&creep_id) {
                return;
            }
            screeps.creeps.push(creep_id);
        }
    }
}

