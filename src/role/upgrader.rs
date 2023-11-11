use screeps::{Creep, ErrorCode, ResourceType, Room, SharedCreepProperties};

use log::*;
use wasm_bindgen::JsValue;

use crate::{
    model::ctx::{CreepMemory, CreepStatus, StoreStatus},
    utils::{self, errorx::ScreepError},
};

pub struct Upgrader<'a> {
    pub creep: &'a Creep,
    pub room: Room,
    pub ctx: CreepMemory,
}

impl<'a> Upgrader<'a> {
    // pub fn role() -> RoleEnum {
    //     return RoleEnum::Upgrader;
    // }
    pub fn new(creep: &'a Creep, ctx: CreepMemory) -> Upgrader<'a> {
        let room = creep.room().expect("room not found");
        Upgrader { creep, room, ctx }
    }

    pub fn check(&self) -> bool {
        if self.creep.fatigue() > 0 {
            return false;
        }
        true
    }

    pub fn set_status(&mut self) {
        self.ctx.store_status = StoreStatus::new(self.creep);
        match self.ctx.store_status {
            StoreStatus::Empty => {
                self.ctx.status = CreepStatus::CarryUp;
            }
            StoreStatus::UnderFill => if self.ctx.status == CreepStatus::SourceNotfound {},
            StoreStatus::Full => {
                self.ctx.status = CreepStatus::Upgrade;
            }
        };
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        if !self.check() {
            return Ok(());
        }
        self.set_status();
        match self.creep.say(self.ctx.status.to_string().as_str(), false) {
            Ok(_) => {}
            Err(e) => {
                warn!("{:?}", e);
            }
        };
        match self.carry() {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        };
        match self.upgrade() {
            Ok(_) => {}
            Err(e) => {
                warn!("{:?}", e);
                return Err(e);
            }
        };

        self.set_memory();

        Ok(())
    }

    pub fn carry(&mut self) -> anyhow::Result<()> {
        match self.ctx.status {
            CreepStatus::CarryUp | CreepStatus::SourceNotfound => {}
            _ => return Ok(()),
        }

        if let Some(structure) = utils::find::find_store(self.creep, &self.room, false) {
            info!("t2");
            if let Some(store) = structure.as_withdrawable() {
                info!("t3");
                match self.creep.withdraw(store, ResourceType::Energy, None) {
                    Ok(_) => {}
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            match utils::line::route_option(
                                self.creep,
                                &structure.as_structure(),
                                utils::line::LineStatus::Carry,
                            ) {
                                Ok(_) => return Ok(()),
                                Err(e) => {
                                    warn!("{:?}", e);
                                    return Err(ScreepError::Unknown.into());
                                }
                            }
                        }
                        _ => {
                            error!("{:?}", e);
                            return Err(ScreepError::Unknown.into());
                        }
                    },
                };
            };
        };

        Ok(())
    }

    pub fn upgrade(&mut self) -> anyhow::Result<()> {
        if self.ctx.status != CreepStatus::Upgrade {
            return Ok(());
        }
        if let Some(structure) = utils::find::find_controller(&self.room) {
            if let Some(structure) = structure.resolve() {
                match self.creep.upgrade_controller(&structure) {
                    Ok(_) => return Ok(()),
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            match utils::line::route_option(
                                self.creep,
                                &structure,
                                utils::line::LineStatus::Carry,
                            ) {
                                Ok(_) => return Ok(()),
                                Err(e) => {
                                    warn!("{:?}", e);
                                    return Err(ScreepError::Unknown.into());
                                }
                            }
                        }
                        _ => {
                            error!("{:?}", e);
                            return Err(ScreepError::Unknown.into());
                        }
                    },
                };
            }
        }
        Ok(())
    }

    pub fn set_memory(&self) {
        self.creep
            .set_memory(&JsValue::from_str(self.ctx.to_string().as_str()));
    }
}
