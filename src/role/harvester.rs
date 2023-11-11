use log::*;
use screeps::{Creep, ErrorCode, HasPosition, ResourceType, Room, SharedCreepProperties};
use wasm_bindgen::JsValue;

use crate::{
    global::SOURCE_MANAGER,
    model::ctx::{CreepMemory, CreepStatus, StoreStatus},
    utils,
};

pub struct Harvester<'a> {
    pub creep: &'a Creep,
    pub room: Room,
    pub ctx: CreepMemory,
}

impl<'a> Harvester<'a> {
    // pub fn role(&self) -> RoleEnum {
    //     return self.ctx.role;
    // }

    pub fn new(creep: &'a Creep, ctx: CreepMemory) -> Harvester<'a> {
        let room = creep.room().expect("room not found");
        Harvester { creep, room, ctx }
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
                self.ctx.status = CreepStatus::Harversting;
            }
            StoreStatus::UnderFill => {
                // if self.ctx.status == CreepStatus::SourceNotfound {
                //     self.ctx.status = CreepStatus::Harversting;
                // }
            }
            StoreStatus::Full => {
                self.ctx.status = CreepStatus::Building;
            }
        };
    }

    pub fn run(&mut self) -> Result<(), ErrorCode> {
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
        match self.harveste() {
            Ok(_) => {}
            Err(e) => {
                warn!("{:?}", e);
                return Err(e);
            }
        };
        match self.consume() {
            Ok(_) => {}
            Err(e) => {
                warn!("{:?}", e);
                return Err(e);
            }
        };

        self.set_memory();

        Ok(())
    }

    // 有资源则收割资源
    pub fn harveste(&mut self) -> Result<(), ErrorCode> {
        match self.ctx.status {
            CreepStatus::Harversting | CreepStatus::SourceNotfound => {}
            _ => return Ok(()),
        }

        let source = SOURCE_MANAGER.with(|manager| {
            let mut manager = manager.borrow_mut();
            match manager.find_and_bind_source(self.room.name().to_string(), self.creep) {
                Some(r) => Some(r),
                None => {
                    self.ctx.status = CreepStatus::SourceNotfound;
                    None
                }
            }
        });
        let source = match source {
            Some(r) => r,
            None => {
                return Ok(());
            }
        };

        match source.resolve() {
            Some(s) => {
                // 资源在附近则收割资源
                if self.creep.pos().is_near_to(s.pos()) {
                    match self.creep.harvest(&s) {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("{:?}", e);
                            return Err(e);
                        }
                    };
                } else {
                    match utils::line::route_option(
                        self.creep,
                        &s,
                        utils::line::LineStatus::Harvesting,
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("{:?}", e);
                            return Err(e);
                        }
                    }
                    return Ok(());
                }
            }
            // 资源不存在
            None => {
                warn!("source not found");
                self.ctx.status = CreepStatus::SourceNotfound;
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn consume(&mut self) -> Result<(), ErrorCode> {
        if self.ctx.status != CreepStatus::Building {
            return Ok(());
        }

        // 建造建筑
        if let Some(site) = utils::find::find_site(self.creep, &self.room) {
            match self.creep.build(&site) {
                Ok(_) => return Ok(()),
                Err(e) => match e {
                    ErrorCode::NotInRange => {
                        match utils::line::route_option(
                            self.creep,
                            &site,
                            utils::line::LineStatus::Building,
                        ) {
                            Ok(_) => {
                                return Ok(());
                            }
                            Err(e) => {
                                warn!("{:?}", e);
                                return Err(e);
                            }
                        }
                    }
                    _ => {
                        warn!("{:?}", e);
                        return Err(e);
                    }
                },
            }
        };

        // 填充容器
        if let Some(store) = utils::find::find_store(self.creep, &self.room, true) {
            if let Some(transfer) = store.as_transferable() {
                // info!("transfer");
                match self.creep.transfer(transfer, ResourceType::Energy, None) {
                    Ok(_) => {
                        // info!("transfer2");
                        return Ok(());
                    }
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            match utils::line::route_option(
                                self.creep,
                                &store.as_structure(),
                                utils::line::LineStatus::Building,
                            ) {
                                Ok(_) => {
                                    // info!("transfer1");
                                    return Ok(());
                                }
                                Err(e) => {
                                    warn!("{:?}", e);
                                    return Err(e);
                                }
                            }
                        }
                        _ => {
                            warn!("{:?}", e);
                            return Err(e);
                        }
                    },
                }
            }

            // 查询控制器
            // if let Some(controller) = utils::find::find_controller(&self.room) {
            //     if let Some(controller) = controller.resolve() {
            //         match self.creep.upgrade_controller(&controller) {
            //             Ok(_) => return Ok(()),
            //             Err(e) => match e {
            //                 ErrorCode::NotInRange => {
            //                     match utils::line::route_option(
            //                         &mut self.creep,
            //                         &controller,
            //                         utils::line::LineStatus::Building,
            //                     ) {
            //                         Ok(_) => {
            //                             return Ok(());
            //                         }
            //                         Err(e) => {
            //                             warn!("{:?}", e);
            //                             return Err(e);
            //                         }
            //                     }
            //                 }
            //                 _ => {
            //                     warn!("{:?}", e);
            //                     return Err(e);
            //                 }
            //             },
            //         }
            //     }
            // };

            // match utils::line::route_option(
            //     &mut self.creep,
            //     &store.as_structure(),
            //     utils::line::LineStatus::Carry,
            // ) {
            //     Ok(_) => return Ok(()),
            //     Err(e) => {
            //         warn!("{:?}", e);
            //         return Err(e);
            //     }
            // }
        }

        Ok(())
    }

    pub fn set_memory(&self) {
        self.creep
            .set_memory(&JsValue::from_str(self.ctx.to_string().as_str()));
    }
}
