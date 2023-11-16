use std::{collections::HashMap, str::FromStr};

use screeps::{
    find, game, Creep, RoomName, SharedCreepProperties, StructureProperties, StructureType,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::{
    global::{self, SOURCE_MANAGER},
    model::ctx::CreepMemory,
    role::{RoleEnum, RoleStatus},
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

    /// 检测数据是否正常
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
                        log::warn!("{:?}", e);
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
    pub porter: usize,
    pub repairer: usize,
    // 管理id key-creep_id value-role_name
    pub creep_map: HashMap<String, String>,
}

impl RoomScreepsItem {
    pub fn can_spawing(&self) -> bool {
        let mut max_count = 0;
        global::SOURCE_MANAGER.with(|manager| {
            let manager = manager.borrow();
            if let Some(m) = manager.get_memory(self.room_id.clone()) {
                max_count = m.max_count;
            }
        });
        let count2 = self.harvester + self.upgrader + self.builder + self.porter + self.repairer;
        if let Ok(room) = utils::find::find_room(self.room_id.clone()) {
            if let Some(c) = utils::find::find_controller(&room) {
                if c.level() <= 2 && self.harvester < max_count {
                    return true;
                } else if c.level() <= 2 {
                    let count = room
                        .find(find::STRUCTURES, None)
                        .iter()
                        .filter(|item| item.structure_type() == StructureType::Container)
                        .count();
                    return 0 < count && count2 < max_count * 5;
                }
            }
        }
        count2 < max_count * 5
    }

    pub fn new(id: String) -> RoomScreepsItem {
        Self {
            room_id: id,
            ..Default::default()
        }
    }

    // 读取每个creep memory初始化，只读取memory能够解析的creep
    pub fn init(&mut self) -> anyhow::Result<()> {
        let room = match RoomName::from_str(&self.room_id) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("{:?}", e);
                return Err(e.into());
            }
        };

        for creep in game::creeps().values() {
            if creep.room().unwrap().name() != room {
                continue;
            }
            if creep.memory().is_null() {
                continue;
            }
            if let Some(r) = creep.memory().as_string() {
                if let Ok(e) = serde_json::from_str::<CreepMemory>(r.as_str()) {
                    self.bind_screep(e);
                };
            }
        }
        Ok(())
    }

    pub fn bind_screep(&mut self, memory: CreepMemory) {
        self.creep_map
            .insert(memory.name, memory.role.get_role_name());
        self.add_count(&memory.role);
    }

    // 添加creep
    pub fn add_screep(&mut self, creep: Creep) -> anyhow::Result<CreepMemory> {
        let role = self.next_role()?;
        let mut c = CreepMemory::new(&creep);

        c.role = role;
        creep.set_memory(&JsValue::from_str(c.to_string().as_str()));
        self.creep_map.insert(creep.name(), c.role.get_role_name());
        self.add_count(&c.role);
        Ok(c)
    }

    // 检测creep是否还存在
    pub fn check(&mut self) {
        self.harvester = 0;
        self.upgrader = 0;
        self.builder = 0;
        self.porter = 0;
        self.repairer = 0;
        self.creep_map.retain(|x, _| utils::check_creep(x.clone()));
        for ele in self.creep_map.values() {
            match RoleEnum::from(ele.clone()) {
                RoleEnum::Harvester(_) => self.harvester += 1,
                RoleEnum::Upgrader(_) => self.upgrader += 1,
                RoleEnum::Builder(_) => self.builder += 1,
                RoleEnum::Porter(_) => self.porter += 1,
                RoleEnum::Repairer(_) => self.repairer += 1,
            }
        }
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
            return Ok(RoleEnum::Harvester(RoleStatus::default()).default());
        }

        if self.porter == 0 {
            return Ok(RoleEnum::Porter(RoleStatus::default()).default());
        }
        if self.upgrader == 0 {
            return Ok(RoleEnum::Upgrader(RoleStatus::default()).default());
        }
        if self.builder == 0 {
            return Ok(RoleEnum::Builder(RoleStatus::default()).default());
        }

        if self.repairer == 0 {
            return Ok(RoleEnum::Repairer(RoleStatus::default()).default());
        }

        if self.porter < self.harvester {
            Ok(RoleEnum::Porter(RoleStatus::default()).default())
        } else if self.builder < 2 {
            Ok(RoleEnum::Builder(RoleStatus::default()).default())
        } else if self.upgrader < 2 {
            Ok(RoleEnum::Upgrader(RoleStatus::default()).default())
        } else {
            Ok(RoleEnum::Repairer(RoleStatus::default()).default())
        }
        // else if self.repairer < 4 {
        //     Ok(RoleEnum::Repairer(RoleStatus::default()).default())
        // }
    }

    fn add_count(&mut self, role: &RoleEnum) {
        match role {
            RoleEnum::Harvester(_) => self.harvester += 1,
            RoleEnum::Upgrader(_) => self.upgrader += 1,
            RoleEnum::Builder(_) => self.builder += 1,
            RoleEnum::Porter(_) => self.porter += 1,
            RoleEnum::Repairer(_) => self.repairer += 1,
        }
    }
}

