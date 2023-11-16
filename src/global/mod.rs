use std::cell::{OnceCell, RefCell};

use screeps::game;

use crate::manager::{
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

/// 初始化缓存
///
/// * `clear_memory`: 是否清理缓存重新计算
pub fn global_init(clear_memory: bool) {
    CELL.with(|item| {
        item.get_or_init(|| {
            MEMORY_MANAGER.with(|memory_map| {
                let mut memory_map = memory_map.borrow_mut();
                for room in game::rooms().values() {
                    let mut room_memory = RoomMemory::new(room.name().to_string());
                    room_memory.init(clear_memory);
                    memory_map.add_room(room_memory);
                }

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
        });
    });
}

/// 检测全局对象数据是否正常
pub fn global_check() {
    SCREEP_MANAGER.with(|manager| {
        let mut manager = manager.borrow_mut();
        manager.check();
    });
    SOURCE_MANAGER.with(|manager| {
        let mut manager = manager.borrow_mut();
        manager.check();
    });
}

/// 将数据保存到room的memory中
pub fn save_memory() {
    SCREEP_MANAGER.with(|manager| {
        let screep_manager = manager.borrow();
        SOURCE_MANAGER.with(|manager| {
            let source_manager = manager.borrow();
            MEMORY_MANAGER.with(|manager| {
                let mut memory_manager = manager.borrow_mut();
                for memory in memory_manager.room_item.values_mut() {
                    let screep_m = match screep_manager.get_memory(memory.room_id.clone()) {
                        Some(r) => r,
                        None => {
                            log::warn!("screep empty");
                            continue;
                        }
                    };
                    let source_m = match source_manager.get_memory(memory.room_id.clone()) {
                        Some(r) => r,
                        None => {
                            log::warn!("source empty");
                            continue;
                        }
                    };
                    memory.creeps_info = screep_m;
                    memory.source_info = source_m;
                }
                memory_manager.set_memory();
            });
        });
    });
}

