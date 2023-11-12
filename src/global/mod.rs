use std::cell::{OnceCell, RefCell};

use log::warn;
use screeps::game;

use crate::source::{
    creesp_manager::{RoomScreepsItem, ScreepManager},
    memory_manager::{RoomMemory, RoomMemoryManager},
    source_manager::{RoomSourceItem, SourceManager},
};

thread_local! {
    pub static CELL :OnceCell<()> = OnceCell::new();
    // room_id
    pub static MEMORY_MANAGER:RefCell<RoomMemoryManager> = RefCell::new(RoomMemoryManager::default());

    pub static SCREEP_MANAGER: RefCell<ScreepManager> = RefCell::new(ScreepManager::new());
    pub static SOURCE_MANAGER: RefCell<SourceManager> = RefCell::new(SourceManager::new());
}

pub fn init_global() {
    CELL.with(|item| {
        item.get_or_init(|| {
            MEMORY_MANAGER.with(|memory_map| {
                let mut memory_map = memory_map.borrow_mut();
                for room in game::rooms().values() {
                    let mut room_memory = RoomMemory::new(room.name().to_string());
                    room_memory.init();
                    memory_map.add_room(room_memory);
                }
            });
            init_manager();
        });
    });
}

pub fn clean_memory() {
    for ele in screeps::game::creeps().values() {
        if let Some(live) = ele.ticks_to_live() {
            if live <= 1 {
                match ele.suicide() {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("{:?}", e);
                    }
                };
            }
        }
    }
}

pub fn init_manager() {
    MEMORY_MANAGER.with(|memory_map| {
        let memory_map = memory_map.borrow_mut();
        let room_creeps: Vec<RoomScreepsItem> = memory_map
            .room_item
            .values()
            .map(|item| item.creeps_info.clone())
            .collect();
        let room_source: Vec<RoomSourceItem> = memory_map
            .room_item
            .values()
            .map(|item| item.source_info.clone())
            .collect();

        SCREEP_MANAGER.with(|manager| {
            let mut manager = manager.borrow_mut();
            manager.init(room_creeps);
        });
        SOURCE_MANAGER.with(|manager| {
            let mut manager = manager.borrow_mut();
            manager.init(room_source);
        });
    });
}
