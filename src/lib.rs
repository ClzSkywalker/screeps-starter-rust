use std::cell::RefCell;
use std::collections::{hash_map::Entry, HashMap};

use log::*;
use screeps::Source;
use screeps::{
    constants::{Part, ResourceType},
    enums::StructureObject,
    find, game,
    objects::Creep,
    prelude::*,
};
use wasm_bindgen::prelude::*;

use crate::model::model::CreepTarget;
use crate::role::{builder, harvester};

mod logging;
mod model;
mod role;
mod structure;

// add wasm_bindgen to any function you would like to expose for call from js
#[wasm_bindgen]
pub fn setup() {
    logging::setup_logging(logging::Info);
}

// this is one way to persist data between ticks within Rust's memory, as opposed to
// keeping state in memory on game objects - but will be lost on global resets!
thread_local! {
    // 资源id-采矿id
    static SOURCE_MAP: RefCell<HashMap<String, [String;2]>> = RefCell::new(HashMap::new());
    // 采矿者
    static WORK_TARGETS: RefCell<HashMap<String, CreepTarget>> = RefCell::new(HashMap::new());
    // 升级者
    static UPGRADER_TARGETS: RefCell<HashMap<String, CreepTarget>> = RefCell::new(HashMap::new());
    // 运输者
    static MOVE_TARGETS: RefCell<HashMap<String, CreepTarget>> = RefCell::new(HashMap::new());
}

// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    debug!("loop starting! CPU: {}", game::cpu::get_used());
    // mutably borrow the creep_targets refcell, which is holding our creep target locks
    // in the wasm heap

    WORK_TARGETS.with(|creep_targets_refcell| {
        let mut creep_targets = creep_targets_refcell.borrow_mut();
        debug!("running creeps");
        for creep in game::creeps().values() {
            // if creep.name().contains(role::builder::Builder::role()) {}
            run_creep(&creep, &mut creep_targets);
        }
    });

    debug!("running spawns");
    let mut additional = 0;
    for spawn in game::spawns().values() {
        debug!("running spawn {}", String::from(spawn.name()));

        let body = [Part::Move, Part::Carry, Part::Work, Part::Work];
        if spawn.room().unwrap().energy_available() >= body.iter().map(|p| p.cost()).sum() {
            // create a unique name, spawn.
            let name_base = game::time();
            let name = format!("{}-{}", name_base, additional);
            match spawn.spawn_creep(&body, &name) {
                Ok(()) => additional += 1,
                Err(e) => warn!("couldn't spawn: {:?}", e),
            }
        }
    }
}

fn run_creep(creep: &Creep, creep_targets: &mut HashMap<String, CreepTarget>) {
    // 是否在孵化中
    if creep.spawning() {
        return;
    }
    let name = creep.name();
    debug!("running creep {}", name);

    let target = creep_targets.entry(name);
    match target {
        // 非空
        Entry::Occupied(entry) => {
            let creep_target = entry.get();
            match creep_target {
                // 升级建筑物
                CreepTarget::ControllerUpgrade(controller_id) => {
                    let ber = builder::Builder::new(creep, controller_id);
                    match ber.build() {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("err:{:?}", e);
                            entry.remove();
                        }
                    }
                }
                // 可收割的资源
                CreepTarget::Harvest(source_id) => {
                    let hman = harvester::Harverster::new(creep, source_id);
                    match hman.harveste() {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("err:{:?}", e);
                            entry.remove();
                        }
                    }
                }
                _ => {
                    entry.remove();
                }
            };
        }
        // 空资源
        Entry::Vacant(entry) => {
            // no target, let's find one depending on if we have energy
            let room = creep.room().expect("couldn't resolve creep room");
            if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
                for structure in room.find(find::STRUCTURES, None).iter() {
                    if creep.move_to(structure.clone()).is_ok() {
                        if let StructureObject::StructureController(controller) = structure {
                            entry.insert(CreepTarget::ControllerUpgrade(controller.id()));
                            return;
                        }
                        return;
                    }
                }
            } else {
                let mut source: Option<Source> = None;
                for ele in room.find(find::SOURCES_ACTIVE, None) {
                    if creep.move_to(ele.clone()).is_ok() {
                        source = Some(ele);
                        break;
                    };
                }
                // entry.insert(CreepTarget::Harvest(source.id()));
                if let Some(r) = source {
                    entry.insert(CreepTarget::Harvest(r.id()));
                };
            }
        }
    }
}
