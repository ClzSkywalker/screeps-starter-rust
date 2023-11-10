// use log::*;
// use screeps::{prelude::*, Creep, ErrorCode, HasPosition, Room};

// use crate::{
//     model::model::{CreepMemory, CreepSourceStatus, CreepStatus, StoreStatus},
//     utils,
// };

// use super::harvester::Harvester;

// pub struct Carrier<'a> {
//     pub creep: &'a Creep,
//     pub room: Room,
//     pub ctx: CreepMemory,
// }

// impl<'a> Carrier<'a> {
//     pub fn new(creep: &'a Creep, ctx: CreepMemory) -> Harvester<'a> {
//         let room = creep.room().expect("room not found");
//         Harvester { creep, room, ctx }
//     }

//     pub fn check(&self) -> bool {
//         if self.creep.fatigue() > 0 {
//             return false;
//         }
//         true
//     }

//     pub fn set_status(&mut self) {
//         self.ctx.store_status = StoreStatus::new(self.creep);
//         match self.ctx.store_status {
//             StoreStatus::Empty => {
//                 self.ctx.status = CreepSourceStatus::CarryUp;
//             }
//             StoreStatus::UnderFill => {}
//             StoreStatus::Full => {
//                 self.ctx.status = CreepSourceStatus::CarryDown;
//             }
//         };
//     }

//     pub fn run(&mut self) -> Result<(), ErrorCode> {
//         if !self.check() {
//             return Ok(());
//         }
//         self.set_status();
//         match self.creep.say(self.ctx.status.to_string().as_str(), false) {
//             Ok(_) => {}
//             Err(e) => {
//                 warn!("{:?}", e);
//                 return Err(e);
//             }
//         };
//         Ok(())
//     }

//     pub fn carry_up(&self) -> Result<(), ErrorCode> {
//         if self.ctx.status != CreepSourceStatus::CarryUp {
//             return Ok(());
//         }

//         Ok(())
//     }
// }
