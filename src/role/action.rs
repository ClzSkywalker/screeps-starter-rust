use log::*;
use screeps::{ErrorCode, ResourceType, SharedCreepProperties};
use wasm_bindgen::JsValue;

use crate::{
    global::SOURCE_MANAGER,
    model::ctx::{CreepStatus, StoreStatus},
    utils::{self, errorx::ScreepError},
};

use super::creep::CreepProp;

pub trait CreepAction {
    fn get_creep(&self) -> &CreepProp;
    fn get_creep_mut(&mut self) -> &mut CreepProp;

    fn say(&self) {
        let prop = self.get_creep();
        if let Some(e) = prop
            .creep
            .say(prop.ctx.status.to_string().as_str(), false)
            .err()
        {
            warn!("{:?}", e);
        };
    }

    fn check(&self) -> bool {
        let prop = self.get_creep();
        if prop.creep.fatigue() > 0 {
            return false;
        }
        true
    }

    fn set_memory(&self) {
        let prop = self.get_creep();
        prop.creep
            .set_memory(&JsValue::from_str(prop.ctx.to_string().as_str()));
    }

    fn set_status(&mut self) {
        let prop = self.get_creep_mut();
        match prop.ctx.role {
            super::RoleEnum::Harvester => {
                prop.ctx.store_status = StoreStatus::new(&prop.creep);
                match prop.ctx.store_status {
                    StoreStatus::Empty => {
                        prop.ctx.status = CreepStatus::Harversting;
                    }

                    StoreStatus::Full => {
                        prop.ctx.status = CreepStatus::Building;
                    }
                    _ => {}
                };
            }
            super::RoleEnum::Upgrader => todo!(),
            super::RoleEnum::Builder => todo!(),
            super::RoleEnum::Porter => todo!(),
        }
    }

    // 收割检测
    fn harveste_check(&self) -> bool {
        let prop = self.get_creep();
        matches!(
            prop.ctx.status,
            CreepStatus::Harversting | CreepStatus::SourceNotfound
        )
    }

    // 收割
    fn harveste(&mut self) -> anyhow::Result<()> {
        if !self.harveste_check() {
            return Ok(());
        }
        let prop = self.get_creep_mut();
        let source = SOURCE_MANAGER.with(|manager| {
            let mut manager = manager.borrow_mut();
            match manager.find_and_bind_source(prop.room.name().to_string(), &prop.creep) {
                Some(r) => Some(r),
                None => {
                    prop.ctx.status = CreepStatus::SourceNotfound;
                    None
                }
            }
        });
        let source = match source {
            Some(r) => r,
            None => {
                return Err(ScreepError::StructureNotfound("source".to_string()).into());
            }
        };

        match source.resolve() {
            Some(site) => match prop.creep.harvest(&site) {
                Ok(_) => {}
                Err(e) => match e {
                    ErrorCode::NotInRange => {
                        match utils::line::route_option(
                            &prop.creep,
                            &site,
                            utils::line::LineStatus::Harvesting,
                        ) {
                            Ok(_) => {
                                return Ok(());
                            }
                            Err(e) => {
                                warn!("{:?}", e);
                                return Err(ScreepError::ScreepInner.into());
                            }
                        }
                    }
                    _ => {
                        warn!("{:?}", e);
                        return Err(ScreepError::ScreepInner.into());
                    }
                },
            },
            None => {
                return Err(ScreepError::RoleCanNotWork(source.to_string()).into());
            }
        }
        Err(ScreepError::RoleCanNotWork(prop.ctx.role.to_string()).into())
    }

    // 收割检测
    fn build_check(&self) -> bool {
        let prop = self.get_creep();
        matches!(prop.ctx.status, CreepStatus::Building)
    }

    // 建造
    fn build(&mut self) -> anyhow::Result<()> {
        if !self.build_check() {
            return Ok(());
        }
        let prop = self.get_creep_mut();
        if let Some(site) = utils::find::find_site(&prop.creep, &prop.room) {
            match prop.creep.build(&site) {
                Ok(_) => return Ok(()),
                Err(e) => match e {
                    ErrorCode::NotInRange => {
                        match utils::line::route_option(
                            &prop.creep,
                            &site,
                            utils::line::LineStatus::Building,
                        ) {
                            Ok(_) => {
                                return Ok(());
                            }
                            Err(e) => {
                                warn!("{:?}", e);
                                return Err(ScreepError::ScreepInner.into());
                            }
                        }
                    }
                    _ => {
                        warn!("{:?}", e);
                        return Err(ScreepError::ScreepInner.into());
                    }
                },
            }
        };
        Err(ScreepError::RoleCanNotWork(prop.ctx.role.to_string()).into())
    }

    fn upgrade_check(&self) -> bool {
        let prop = self.get_creep();
        matches!(prop.ctx.status, CreepStatus::Building)
    }

    fn upgrade(&mut self) -> anyhow::Result<()> {
        if !self.upgrade_check() {
            return Ok(());
        }
        let prop = self.get_creep_mut();
        if let Some(site) = utils::find::find_controller(&prop.room) {
            match site.resolve() {
                Some(controller) => match prop.creep.upgrade_controller(&controller) {
                    Ok(_) => return Ok(()),
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            match utils::line::route_option(
                                &prop.creep,
                                &controller,
                                utils::line::LineStatus::Building,
                            ) {
                                Ok(_) => {
                                    return Ok(());
                                }
                                Err(e) => {
                                    warn!("{:?}", e);
                                    return Err(ScreepError::ScreepInner.into());
                                }
                            }
                        }
                        _ => {
                            warn!("{:?}", e);
                            return Err(ScreepError::ScreepInner.into());
                        }
                    },
                },
                None => {
                    warn!("{}", ScreepError::RoomNotfound(site.to_string()));
                    return Err(ScreepError::RoomNotfound(site.to_string()).into());
                }
            }
        };
        Err(ScreepError::RoleCanNotWork(prop.ctx.role.to_string()).into())
    }

    fn store_check(&self) -> bool {
        let prop = self.get_creep();
        matches!(prop.ctx.status, CreepStatus::Building)
    }

    fn store(&mut self) -> anyhow::Result<()> {
        if !self.store_check() {
            return Ok(());
        }
        let prop = self.get_creep_mut();
        if let Some(store) = utils::find::find_store(&prop.creep, &prop.room, true) {
            if let Some(transfer) = store.as_transferable() {
                // info!("transfer");
                match prop.creep.transfer(transfer, ResourceType::Energy, None) {
                    Ok(_) => {
                        // info!("transfer2");
                        return Ok(());
                    }
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            match utils::line::route_option(
                                &prop.creep,
                                &store.as_structure(),
                                utils::line::LineStatus::Building,
                            ) {
                                Ok(_) => {
                                    // info!("transfer1");
                                    return Ok(());
                                }
                                Err(e) => {
                                    warn!("{:?}", e);
                                    return Err(ScreepError::ScreepInner.into());
                                }
                            }
                        }
                        _ => {
                            warn!("{:?}", e);
                            return Err(ScreepError::ScreepInner.into());
                        }
                    },
                }
            }
        }
        Err(ScreepError::RoleCanNotWork(prop.ctx.role.to_string()).into())
    }

    fn carry_up_check(&self) -> bool {
        let prop = self.get_creep();
        matches!(
            prop.ctx.status,
            CreepStatus::CarryUp | CreepStatus::SourceNotfound
        )
    }

    fn carry_up(&mut self) -> anyhow::Result<()> {
        if !self.carry_up_check() {
            return Ok(());
        }
        let prop = self.get_creep_mut();
        if let Some(structure) = utils::find::find_store(&prop.creep, &prop.room, false) {
            info!("t2");
            if let Some(store) = structure.as_withdrawable() {
                info!("t3");
                match prop.creep.withdraw(store, ResourceType::Energy, None) {
                    Ok(_) => {}
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            match utils::line::route_option(
                                &prop.creep,
                                &structure.as_structure(),
                                utils::line::LineStatus::Carry,
                            ) {
                                Ok(_) => return Ok(()),
                                Err(e) => {
                                    warn!("{:?}", e);
                                    return Err(ScreepError::ScreepInner.into());
                                }
                            }
                        }
                        _ => {
                            error!("{:?}", e);
                            return Err(ScreepError::ScreepInner.into());
                        }
                    },
                };
            };
        };

        Ok(())
    }
}
