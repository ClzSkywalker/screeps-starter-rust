use screeps::{Creep, Room};

use crate::model::ctx::CreepMemory;

#[derive(Debug, Clone)]
pub struct CreepProp {
    pub creep: Creep,
    pub room: Room,
    pub ctx: CreepMemory,
}

impl CreepProp {
    pub fn new(creep: Creep, ctx: CreepMemory) -> Self {
        let room = creep.room().expect("room not found");
        Self { creep, room, ctx }
    }
}
