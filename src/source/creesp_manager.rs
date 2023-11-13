use std::{collections::HashMap, str::FromStr};

use log::warn;
use screeps::{game, Creep, RoomName, SharedCreepProperties};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::{
    global::SOURCE_MANAGER,
    model::ctx::CreepMemory,
    role::RoleEnum,
    utils::{self, errorx::ScreepError},
};

// creep 管理器
#[derive(Debug, Default)]
pub struct ScreepManager {
    pub room_item: HashMap<String, RoomScreepsItem>,
}

impl ScreepManager {
    pub fn new() -> Self {
        ScreepManager {
            room_item: HashMap::new(),
        }
    }

    pub fn init(&mut self, room_creeps: Vec<RoomScreepsItem>) {
        for ele in room_creeps {
            self.room_item.insert(ele.room_id.clone(), ele);
        }
    }

    pub fn can_spawing(&self, room_id: String) -> bool {
        if let Some(item) = self.room_item.get(&room_id) {
            return item.can_spawing();
        }
        false
    }

    // pub fn add_room(&mut self, item: RoomScreepsItem) {
    //     self.room_item.insert(item.room_id.clone(), item);
    // }

    pub fn add_screep(&mut self, creep: Creep) -> Option<CreepMemory> {
        if let Some(r) = self
            .room_item
            .get_mut(&creep.room().unwrap().name().to_string())
        {
            if let Ok(r) = r.add_screep(creep) {
                return Some(r);
            }
        }
        None
    }

    pub fn check(&mut self) {
        for ele in self.room_item.values_mut() {
            ele.check();
        }
    }

    pub fn get_memory(&self, room_id: String) -> Option<RoomScreepsItem> {
        for (rid, value) in self.room_item.iter() {
            if rid != &room_id {
                continue;
            }
            return Some(value.clone());
        }
        None
    }

    // 内存中存在则返回，否则进行创建初始化
    pub fn get_or_init_memory(&mut self, creep: &Creep) -> CreepMemory {
        match creep.memory().as_string() {
            Some(r) => {
                let c: CreepMemory = match serde_json::from_str(&r) {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("{:?}", e);
                        match self.add_screep(creep.clone()) {
                            Some(r) => r,
                            None => CreepMemory::new(creep),
                        }
                    }
                };
                c
            }
            None => match self.add_screep(creep.clone()) {
                Some(r) => r,
                None => CreepMemory::new(creep),
            },
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RoomScreepsItem {
    pub room_id: String,
    pub harvester: usize,
    pub upgrader: usize,
    pub builder: usize,
    // 管理id
    pub creep_map: HashMap<String, RoleEnum>,
}

impl RoomScreepsItem {
    pub fn can_spawing(&self) -> bool {
        // game::creeps().keys().count()
        true
    }

    pub fn new(id: String) -> RoomScreepsItem {
        Self {
            room_id: id,
            ..Default::default()
        }
    }

    // 读取每个creep memory初始化creep item manager
    pub fn init(&mut self) -> anyhow::Result<()> {
        let room = match RoomName::from_str(&self.room_id) {
            Ok(r) => r,
            Err(e) => {
                warn!("{:?}", e);
                return Err(e.into());
            }
        };

        for creep in game::creeps().values() {
            if creep.room().unwrap().name() != room {
                continue;
            }
            match creep.memory().as_string() {
                Some(r) => {
                    if let Err(e) = serde_json::from_str::<CreepMemory>(r.as_str()) {
                        warn!("{:?},json:{}", e, r.as_str());
                        if let Err(e) = self.add_screep(creep) {
                            log::warn!("{:?}", e);
                        }
                    };
                }
                None => {
                    if let Err(e) = self.add_screep(creep) {
                        log::warn!("{:?}", e);
                    }
                }
            };
        }
        Ok(())
    }

    // 添加creep
    pub fn add_screep(&mut self, creep: Creep) -> anyhow::Result<CreepMemory> {
        let role = self.next_role()?;
        let mut c = CreepMemory::new(&creep);
        c.role = role;
        creep.set_memory(&JsValue::from_str(c.to_string().as_str()));
        self.add_count(&creep, role);
        Ok(c)
    }

    // 检测creep是否还存在
    pub fn check(&mut self) {
        self.creep_map.retain(|x, _| utils::check_creep(x.clone()));
        self.harvester = self
            .creep_map
            .values()
            .filter(|x| **x == RoleEnum::Harvester)
            .count();
        self.upgrader = self
            .creep_map
            .values()
            .filter(|x| **x == RoleEnum::Upgrader)
            .count();
        self.builder = self
            .creep_map
            .values()
            .filter(|x| **x == RoleEnum::Builder)
            .count();
    }

    pub fn next_role(&self) -> anyhow::Result<RoleEnum> {
        let room_source_info = match SOURCE_MANAGER.with(|manager| {
            let manager = manager.borrow();
            match manager.get_memory(self.room_id.clone()) {
                Some(r) => Ok(r),
                None => {
                    log::warn!("{}", ScreepError::RoomNotfound(self.room_id.clone()));
                    Err(ScreepError::RoomNotfound(self.room_id.clone()))
                }
            }
        }) {
            Ok(r) => r,
            Err(e) => return Err(e.into()),
        };

        if self.harvester < room_source_info.max_count {
            return Ok(RoleEnum::Harvester);
        }
        if self.builder == 0 {
            return Ok(RoleEnum::Builder);
        }
        if self.upgrader == 0 {
            return Ok(RoleEnum::Upgrader);
        }
        if self.upgrader < self.harvester * 2 {
            Ok(RoleEnum::Upgrader)
        } else if self.builder < self.harvester * 2 {
            Ok(RoleEnum::Builder)
        } else {
            Ok(RoleEnum::Harvester)
        }
    }

    fn add_count(&mut self, creep: &Creep, role: RoleEnum) {
        match role {
            RoleEnum::Harvester => self.harvester += 1,
            RoleEnum::Upgrader => self.upgrader += 1,
            RoleEnum::Builder => self.builder += 1,
            _ => {}
        }
        self.creep_map.insert(creep.name().to_string(), role);
    }
}

