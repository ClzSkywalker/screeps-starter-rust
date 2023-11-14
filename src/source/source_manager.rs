use std::collections::HashMap;

use screeps::{
    constants::Terrain,
    find,
    look::{self},
    Creep, HasPosition, HasTypedId, ObjectId, Room, SharedCreepProperties, Source,
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
        if let Some(source_i) = self.find_source_can_work(room_id.clone(), creep) {
            let s = source_i.id();
            if let Ok(r) = self.bind_source(room_id.clone(), s.to_string(), creep) {
                if r.is_some() {
                    return Some(s);
                }
            }
            log::info!("source not found");
            return None;
        }
        None
    }

    /// 发现资源
    ///
    /// * `room_id`:
    /// * `creep`:
    pub fn find_source_can_work(&self, room_id: String, creep: &Creep) -> Option<Source> {
        let room = match utils::find::find_room(room_id.clone()) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("{:?}", e);
                return None;
            }
        };
        let mut source_list = Vec::new();
        for source in room.find(find::SOURCES_ACTIVE, None).iter() {
            if !self.check_can_work(
                room_id.clone(),
                source.id().to_string(),
                creep.name().to_string(),
            ) {
                continue;
            }

            source_list.push(source.clone());
        }
        utils::find::get_near_site(creep, &source_list)
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
                r.bind_screep(source_id.clone(), creep.name().to_string());
                Ok(Some(()))
            }
            None => {
                log::error!("{:?}", ScreepError::RoomNotfound(room_id.clone()));
                Err(ScreepError::RoomNotfound(room_id.clone()).into())
            }
        }
    }

    pub fn check_can_work(&self, room_id: String, source_id: String, creep_id: String) -> bool {
        if let Some(room) = self.room_item.get(&room_id) {
            return room.check_bind(source_id, creep_id);
        }
        false
    }

    /// 检测creep是否正常，把不正常的解除绑定
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

impl RoomSourceItem {
    pub fn new(id: String) -> Self {
        RoomSourceItem {
            room_id: id,
            source_map: HashMap::new(),
            ..Default::default()
        }
    }

    /// 初始化结构，查询能够该source能够最多被多少个creep挖掘
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

    /// 检测自身数据是否正常
    pub fn check(&mut self) {
        for (_, screeps) in self.source_map.iter_mut() {
            utils::remove_expire_screep(&mut screeps.creeps);
            if screeps.max_count >= screeps.creeps.len() {
                continue;
            }
            screeps.creeps.truncate(screeps.max_count);
            // utils::remove_repeat_screep(&mut screeps.creeps);
        }
    }
    /// 检测能否进行绑定
    ///
    /// * `source_id`:
    /// * `creep_id`:
    pub fn check_bind(&self, source_id: String, creep_id: String) -> bool {
        if let Some(screeps) = self.source_map.get(&source_id) {
            if screeps.creeps.len() >= screeps.max_count && !screeps.creeps.contains(&creep_id) {
                return false;
            }
            return true;
        }
        false
    }
    /// 绑定新关系，移除旧关系
    ///
    /// * `source_id`:
    /// * `creep_id`:
    pub fn bind_screep(&mut self, source_id: String, creep_id: String) {
        if let Some(screeps) = self.source_map.get_mut(&source_id) {
            if screeps.creeps.contains(&creep_id) {
                return;
            }
            screeps.creeps.push(creep_id.clone());
        }
        for (s_id, source_info) in self.source_map.iter_mut() {
            if s_id == &source_id {
                continue;
            }
            source_info.creeps.retain(|item| item != &creep_id);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfoItem {
    pub source_id: String,
    pub creeps: Vec<String>, // 挖掘的creep id
    pub max_count: usize,    // 最大的source 挖掘人数
}

impl SourceInfoItem {
    pub fn new(room: &Room, item: &Source) -> Self {
        let count = utils::find::get_area_range(room, look::TERRAIN, item.pos(), 1)
            .iter()
            .filter(|item| match item.look_result {
                look::LookResult::Terrain(a) => {
                    if let Terrain::Wall = a {
                        return true;
                    }
                    false
                }
                _ => false,
            })
            .count();

        let max_count: usize = 9 - count;
        Self {
            source_id: item.id().to_string(),
            creeps: Vec::default(),
            max_count,
        }
    }
}

