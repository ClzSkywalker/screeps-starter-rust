use std::{collections::HashMap, str::FromStr};

use log::warn;
use screeps::{game, Creep, RoomName, SharedCreepProperties};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::{model::ctx::CreepMemory, role::RoleEnum, utils};

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
            return Some(r.add_screep(creep));
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
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RoomScreepsItem {
    pub room_id: String,
    pub harvester: usize,
    pub upgrader: usize,
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
            harvester: Default::default(), //2
            upgrader: Default::default(),  //1
            creep_map: HashMap::default(),
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
                    match serde_json::from_str(r.as_str()) {
                        Ok(r) => r,
                        Err(e) => {
                            warn!("{:?},json:{}", e, r.as_str());
                            self.add_screep(creep);
                        }
                    };
                }
                None => {
                    self.add_screep(creep);
                }
            };
        }
        Ok(())
    }

    // 添加creep
    pub fn add_screep(&mut self, creep: Creep) -> CreepMemory {
        let role = self.next_role();
        let mut c = CreepMemory::new(&creep);
        c.role = role;
        creep.set_memory(&JsValue::from_str(c.to_string().as_str()));
        self.add_count(&creep, role);
        c
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
    }

    pub fn next_role(&self) -> RoleEnum {
        if self.harvester < 2 {
            return RoleEnum::Harvester;
        }
        if self.harvester * 3 > self.upgrader {
            RoleEnum::Upgrader
        } else {
            RoleEnum::Harvester
        }
    }

    fn add_count(&mut self, creep: &Creep, role: RoleEnum) {
        match role {
            RoleEnum::Harvester => self.harvester += 1,
            RoleEnum::Upgrader => self.upgrader += 1,
            _ => {}
        }
        self.creep_map.insert(creep.name().to_string(), role);
    }
}
