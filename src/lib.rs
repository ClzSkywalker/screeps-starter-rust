use model::ctx::CreepMemory;

use role::{creep::CreepProp, RoleAction};
use screeps::{constants::Part, game, prelude::*};
use structure::StructureAction;
use wasm_bindgen::prelude::*;

mod global;
mod logging;
mod manager;
mod model;
mod role;
mod structure;
mod utils;

// add wasm_bindgen to any function you would like to expose for call from js
#[wasm_bindgen]
pub fn setup() {
    logging::setup_logging(logging::Info);
}

// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    global::global_init(true);
    global::global_check();

    for creep in game::creeps().values() {
        if creep.spawning() {
            continue;
        }

        let creep_memory: CreepMemory = global::SCREEP_MANAGER.with(|manager| {
            let mut manager = manager.borrow_mut();
            manager.get_or_init_memory(&creep)
        });

        RoleAction::new(CreepProp::new(creep, creep_memory)).run();
    }

    let mut additional = 0;
    global::SCREEP_MANAGER.with(|manager| {
        let manager = manager.borrow();
        for spawn in game::spawns().values() {
            let room = spawn.room().unwrap();
            let ext_count = utils::find::get_extension_count(&room).len();
            let mut body = vec![Part::Move, Part::Carry, Part::Work, Part::Work];
            if ext_count >= 10 {
                body.append(&mut vec![
                    Part::Carry,
                    Part::Carry,
                    Part::Move,
                    Part::Move,
                    Part::Carry,
                    Part::Work,
                    Part::Work,
                ]);
            } else if ext_count >= 6 {
                body.append(&mut vec![Part::Carry, Part::Move, Part::Carry, Part::Work]);
            } else if ext_count >= 4 {
                body.append(&mut vec![Part::Move, Part::Carry, Part::Work]);
            } else if ext_count > 2 {
                body.append(&mut vec![Part::Move, Part::Carry]);
            }
            let consum = body.iter().map(|p| p.cost()).sum();
            // 能量不足
            if spawn.room().unwrap().energy_available() < consum {
                continue;
            }

            // 不允许生产
            let spawing = manager.can_spawing(spawn.room().unwrap().name().to_string());
            if !spawing {
                // log::info!("can_span:{}", spawing);
                continue;
            }
            // log::info!(
            //     "energy:{},custom:{}",
            //     spawn.room().unwrap().energy_available(),
            //     body.iter().map(|p| p.cost()).sum()
            // );
            if spawn.room().unwrap().energy_available() >= body.iter().map(|p| p.cost()).sum() {
                // create a unique name, spawn.
                let name_base = game::time();
                let name = format!("{}-{}", name_base, additional);
                match spawn.spawn_creep(&body, &name) {
                    Ok(()) => {
                        additional += 1;
                    }
                    Err(e) => log::warn!("couldn't spawn: {:?}", e),
                }
            }
        }
    });

    for structure in game::structures().values() {
        if let screeps::StructureObject::StructureTower(tower) = structure {
            StructureAction::from(tower).run();
        }
    }

    global::save_memory();
}

