use std::{collections::HashMap, str::FromStr};

use log::{error, warn};
use screeps::{game, RoomName};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::utils::{self, errorx::ScreepError};

use super::{creesp_manager::RoomScreepsItem, source_manager::RoomSourceItem};

#[derive(Debug, Default)]
pub struct RoomMemoryManager {
    pub room_item: HashMap<String, RoomMemory>,
}

impl RoomMemoryManager {
    pub fn add_room(&mut self, room: RoomMemory) {
        self.room_item.insert(room.room_id.clone(), room);
    }

    pub fn set_memory(&self) {
        for (_, memory) in self.room_item.iter() {
            memory.set_memory()
        }
    }

    pub fn check(&mut self) {
        for ele in self.room_item.values_mut() {
            ele.check();
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RoomMemory {
    pub room_id: String,
    pub creeps_info: RoomScreepsItem,
    pub source_info: RoomSourceItem,
}

impl RoomMemory {
    pub fn new(id: String) -> Self {
        Self {
            room_id: id,
            creeps_info: RoomScreepsItem::default(),
            source_info: RoomSourceItem::default(),
        }
    }

    // 如果房间memory存在加载，不存在则初始化
    pub fn init(&mut self) {
        let room = match RoomName::from_str(&self.room_id) {
            Ok(r) => r,
            Err(e) => {
                warn!("{:?}", e);
                return;
            }
        };
        let room = match game::rooms().get(room) {
            Some(r) => r,
            None => {
                warn!("{}", ScreepError::RoomNotfound(self.room_id.clone()));
                return;
            }
        };
        let mut room_screep = RoomScreepsItem::new(self.room_id.clone());
        let mut room_source = RoomSourceItem::new(self.room_id.clone());
        match room.memory().as_string() {
            Some(memory_str) => match serde_json::from_str(memory_str.as_str()) {
                Ok(r) => {
                    let tmp: RoomMemory = r;
                    room_screep = tmp.creeps_info;
                    room_source = tmp.source_info;
                }
                Err(e) => {
                    warn!("{:?},json:{}", e, memory_str.as_str());
                    let _ = room_screep.init();
                    let _ = room_source.init();
                }
            },
            None => {
                let _ = room_screep.init();
                let _ = room_source.init();
            }
        };
        self.creeps_info = room_screep;
        self.source_info = room_source;
    }

    pub fn check(&mut self) {
        self.creeps_info.check();
    }

    pub fn set_memory(&self) {
        if let Ok(room) = utils::find::find_room(self.room_id.clone()) {
            match serde_json::to_string(self) {
                Ok(r) => {
                    room.set_memory(&JsValue::from_str(r.as_str()));
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
    }
}
