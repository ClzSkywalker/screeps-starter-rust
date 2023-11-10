use std::{cell::RefCell, collections::HashMap};

use log::*;
use model::model::{CreepMemory, StoreStatus};

use screeps::{constants::Part, game, prelude::*};
use wasm_bindgen::prelude::*;

mod logging;
mod model;
mod role;
mod structure;
mod utils;

// add wasm_bindgen to any function you would like to expose for call from js
#[wasm_bindgen]
pub fn setup() {
    logging::setup_logging(logging::Info);
}

// this is one way to persist data between ticks within Rust's memory, as opposed to
// keeping state in memory on game objects - but will be lost on global resets!
// thread_local! {
//     // role count
//     static CREEP_ROLE_MAP:RefCell<HashMap<String, i32>>= RefCell::new(HashMap::new());
//     // 资源id-采矿id
//     static CREEP_STATUS: RefCell<HashMap<String, CreepMemory>>= RefCell::new(HashMap::new());
//     // 采矿者
//     static WORK_TARGETS: RefCell<HashMap<String, CreepTarget>> = RefCell::new(HashMap::new());
//     // 升级者
//     static UPGRADER_TARGETS: RefCell<HashMap<String, CreepTarget>> = RefCell::new(HashMap::new());
// }

thread_local! {
    // key-sourceId value-creep id
    static WORK_TARGETS: RefCell<HashMap<String, Vec<String>>> = RefCell::new(HashMap::new());
}

// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    let mut creep_count = 0;
    for creep in game::creeps().values() {
        if creep.spawning() {
            continue;
        }
        let r = match creep.memory().as_string() {
            Some(r) => {
                let c: CreepMemory = match serde_json::from_str(&r) {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("{:?}", e);
                        CreepMemory::default()
                    }
                };
                c
            }
            None => {
                let mut c = CreepMemory::default();
                c.store_status = StoreStatus::new(&creep);
                c
            }
        };
        let mut har = role::harvester::Harvester::new(&creep, r.clone());
        match har.run() {
            Ok(_) => {}
            Err(e) => {
                warn!("{:?}", e);
            }
        };
        creep.set_memory(&JsValue::from_str(&har.ctx.to_string().as_str()));
        creep_count += 1;
    }
    let mut additional = 0;
    for spawn in game::spawns().values() {
        if creep_count > 0 {
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
}

// fn run_creep(
//     creep: &Creep,
//     creep_status: &HashMap<String, CreepMemory>,
//     creep_targets: &mut HashMap<String, CreepTarget>,
// ) {
//     // 是否在孵化中
//     if creep.spawning() {
//         return;
//     }
//     let name = creep.name();
//     let status = match creep_status.get(&name) {
//         Some(r) => r,
//         None => return,
//     };

//     let target = creep_targets.entry(name);
//     match target {
//         // 非空
//         Entry::Occupied(entry) => {
//             let creep_target = entry.get();
//             match creep_target {
//                 // 升级建筑物
//                 CreepTarget::ControllerUpgrade(controller_id) => {
//                     let ber = upgrade_controller::Builder::new(creep, controller_id);
//                     match ber.build() {
//                         Ok(_) => {}
//                         Err(_) => {
//                             entry.remove();
//                         }
//                     }
//                 }
//                 // 可收割的资源
//                 CreepTarget::Harvest(source_id) => {
//                     let mut hman = harvester::Harverster::new(creep, source_id);
//                     match hman.harveste() {
//                         Ok(_) => {
//                             if hman.set_status() {
//                                 entry.remove();
//                             }
//                         }
//                         Err(_) => {
//                             entry.remove();
//                         }
//                     };
//                 }
//                 CreepTarget::ConstructionSiteBuild(id) => {
//                     let mut hman = builder::Builder::new(creep, id);
//                     match hman.build() {
//                         Ok(_) => {
//                             if hman.is_remove_task() {
//                                 entry.remove();
//                             }
//                         }
//                         Err(_) => {
//                             entry.remove();
//                         }
//                     };
//                 }
//                 _ => {
//                     entry.remove();
//                 }
//             };
//         }
//         // 空资源
//         Entry::Vacant(entry) => {
//             // 疲劳值大于0不移动
//             if creep.fatigue() > 0 {
//                 return;
//             }
//             let room = creep.room().expect("couldn't resolve creep room");
//             match status.role {
//                 RoleEnum::Harvester => {
//                     let mut source: Option<Source> = None;
//                     for ele in room.find(find::SOURCES_ACTIVE, None) {
//                         if creep.move_to(ele.clone()).is_ok() {
//                             source = Some(ele);
//                             break;
//                         };
//                     }
//                     // entry.insert(CreepTarget::Harvest(source.id()));
//                     if let Some(r) = source {
//                         entry.insert(CreepTarget::Harvest(r.id()));
//                     };
//                 }
//                 RoleEnum::UpgradeController => {
//                     if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
//                         for structure in room.find(find::STRUCTURES, None).iter() {
//                             if creep.move_to(structure.clone()).is_ok() {
//                                 if let StructureObject::StructureController(controller) = structure
//                                 {
//                                     entry.insert(CreepTarget::ControllerUpgrade(controller.id()));
//                                     return;
//                                 }
//                             }
//                         }
//                     }
//                 }
//                 RoleEnum::Builder => {
//                     if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
//                         for structure in room.find(find::CONSTRUCTION_SITES, None).iter() {
//                             if creep.move_to(structure.clone()).is_ok() {
//                                 let id = match structure.try_id() {
//                                     Some(r) => r,
//                                     None => {
//                                         return;
//                                     }
//                                 };
//                                 entry.insert(CreepTarget::ConstructionSiteBuild(id));
//                                 break;
//                             }
//                         }
//                     }
//                 }
//             };

//             // no target, let's find one depending on if we have energy
//             // 需要升级的控制器
//         }
//     }
// }
