use global::{MEMORY_MANAGER, SCREEP_MANAGER, SOURCE_MANAGER};
use log::*;
use model::ctx::CreepMemory;

use role::RoleEnum;
use screeps::{constants::Part, game, prelude::*};
use wasm_bindgen::prelude::*;

mod global;
mod logging;
mod model;
mod role;
mod source;
mod utils;

// add wasm_bindgen to any function you would like to expose for call from js
#[wasm_bindgen]
pub fn setup() {
    logging::setup_logging(logging::Info);
}

// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    global::init_global();
    MEMORY_MANAGER.with(|manager| {
        let mut manager = manager.borrow_mut();
        manager.check();
    });
    SCREEP_MANAGER.with(|manager| {
        let mut manager = manager.borrow_mut();
        manager.check();
    });
    SOURCE_MANAGER.with(|manager| {
        let mut manager = manager.borrow_mut();
        manager.check();
    });

    for creep in game::creeps().values() {
        if creep.spawning() {
            continue;
        }

        let creep_memory: CreepMemory = SCREEP_MANAGER.with(|manager| {
            let mut manager = manager.borrow_mut();
            match creep.memory().as_string() {
                Some(r) => {
                    let c: CreepMemory = match serde_json::from_str(&r) {
                        Ok(r) => r,
                        Err(e) => {
                            warn!("{:?}", e);
                            match manager.add_screep(creep.clone()) {
                                Some(r) => r,
                                None => CreepMemory::new(&creep),
                            }
                        }
                    };
                    c
                }
                None => match manager.add_screep(creep.clone()) {
                    Some(r) => r,
                    None => CreepMemory::new(&creep),
                },
            }
        });

        match creep_memory.role {
            RoleEnum::Harvester => {
                let mut har = role::harvester::Harvester::new(&creep, creep_memory.clone());
                match har.run() {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("{:?}", e);
                    }
                };
            }
            RoleEnum::Upgrader => {
                let mut har = role::upgrader::Upgrader::new(&creep, creep_memory.clone());
                match har.run() {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("{:?}", e);
                    }
                };
            }
            _ => {}
        }
    }
    let mut additional = 0;
    for spawn in game::spawns().values() {
        let mut spawing = false;
        SCREEP_MANAGER.with(|manager| {
            let manager = manager.borrow();
            spawing = manager.can_spawing(spawn.room().unwrap().name().to_string());
        });
        if spawing {
            continue;
        }
        let body = [Part::Move, Part::Move, Part::Carry, Part::Work];
        if spawn.room().unwrap().energy_available() >= body.iter().map(|p| p.cost()).sum() {
            // create a unique name, spawn.
            let name_base = game::time();
            let name = format!("{}-{}", name_base, additional);
            match spawn.spawn_creep(&body, &name) {
                Ok(()) => {
                    additional += 1;
                }
                Err(e) => warn!("couldn't spawn: {:?}", e),
            }
        }
    }

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
                            continue;
                        }
                    };
                    let source_m = match source_manager.get_memory(memory.room_id.clone()) {
                        Some(r) => r,
                        None => {
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
