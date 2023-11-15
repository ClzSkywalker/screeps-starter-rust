use log::*;
use screeps::{ErrorCode, ResourceType, SharedCreepProperties};
use wasm_bindgen::JsValue;

use crate::{
    global::SOURCE_MANAGER,
    model::ctx::{ActionStatus, StoreStatus},
    utils::{self, errorx::ScreepError, find::FindStoreOption},
};

use super::creep::CreepProp;

pub trait ICreepAction {
    fn get_creep(&self) -> &CreepProp;
    fn get_creep_mut(&mut self) -> &mut CreepProp;

    fn say(&self) {
        let prop = self.get_creep();
        if let Some(e) = prop
            .creep
            .say(prop.ctx.role.get_say_test().as_str(), false)
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

    /// 将信息放置creep内存中
    fn set_memory(&self) {
        let prop = self.get_creep();
        prop.creep
            .set_memory(&JsValue::from_str(prop.ctx.to_string().as_str()));
    }

    fn set_status(&mut self) {
        let prop = self.get_creep_mut();
        prop.ctx.store_status = StoreStatus::new(&prop.creep);
        prop.ctx.role.reset_status(prop.ctx.store_status.clone());
    }

    // 收割能源
    fn harveste(&mut self) -> anyhow::Result<Option<()>> {
        let prop = self.get_creep_mut();
        if !prop.ctx.role.check(ActionStatus::Harversting) {
            return Ok(None);
        }
        let source = SOURCE_MANAGER.with(|manager| {
            let mut manager = manager.borrow_mut();
            manager.find_and_bind_source(prop.room.name().to_string(), &prop.creep)
        });
        let source = match source {
            Some(r) => r,
            None => return Ok(None),
        };

        if let Some(site) = source.resolve() {
            match prop.creep.harvest(&site) {
                Ok(_) => {
                    prop.ctx.role.change_action(ActionStatus::Harversting);
                    return Ok(Some(()));
                }
                Err(e) => match e {
                    ErrorCode::NotInRange => {
                        prop.ctx.role.change_action(ActionStatus::Harversting);
                        match utils::line::route_option(
                            &prop.creep,
                            &site,
                            utils::line::LineStatus::Harvesting,
                        ) {
                            Ok(_) => return Ok(Some(())),
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
        Ok(None)
    }

    /// 捡起掉落的资源
    fn pickup(&mut self) -> anyhow::Result<Option<()>> {
        let prop = self.get_creep_mut();
        if !prop.ctx.role.check(ActionStatus::PickUp) {
            return Ok(None);
        }
        if let Some(site) = utils::find::find_tombstone(&prop.creep, &prop.room) {
            match prop.creep.withdraw(&site, ResourceType::Energy, None) {
                Ok(_) => {
                    prop.ctx.role.change_action(ActionStatus::PickUp);
                    return Ok(Some(()));
                }
                Err(e) => match e {
                    ErrorCode::NotInRange => {
                        prop.ctx.role.change_action(ActionStatus::PickUp);
                        match utils::line::route_option(
                            &prop.creep,
                            &site,
                            utils::line::LineStatus::Harvesting,
                        ) {
                            Ok(_) => {
                                return Ok(Some(()));
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

        if let Some(site) = utils::find::find_drop_resource(&prop.creep, &prop.room) {
            match prop.creep.pickup(&site) {
                Ok(_) => {
                    prop.ctx.role.change_action(ActionStatus::PickUp);
                    return Ok(Some(()));
                }
                Err(e) => match e {
                    ErrorCode::NotInRange => {
                        prop.ctx.role.change_action(ActionStatus::PickUp);
                        match utils::line::route_option(
                            &prop.creep,
                            &site,
                            utils::line::LineStatus::Harvesting,
                        ) {
                            Ok(_) => {
                                return Ok(Some(()));
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
        Ok(None)
    }

    // 建造待建造的建筑
    fn build(&mut self) -> anyhow::Result<Option<()>> {
        let prop = self.get_creep_mut();
        if !prop.ctx.role.check(ActionStatus::Building) {
            return Ok(None);
        }
        if let Some(site) = utils::find::find_site(&prop.creep, &prop.room) {
            match prop.creep.build(&site) {
                Ok(_) => {
                    prop.ctx.role.change_action(ActionStatus::Building);
                    return Ok(Some(()));
                }
                Err(e) => match e {
                    ErrorCode::NotInRange => {
                        prop.ctx.role.change_action(ActionStatus::Building);
                        match utils::line::route_option(
                            &prop.creep,
                            &site,
                            utils::line::LineStatus::Building,
                        ) {
                            Ok(_) => {
                                return Ok(Some(()));
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
        Ok(None)
    }

    /// 升级控制器
    fn upgrade(&mut self) -> anyhow::Result<Option<()>> {
        let prop = self.get_creep_mut();
        if !prop.ctx.role.check(ActionStatus::Upgrade) {
            return Ok(None);
        }

        if let Some(site) = utils::find::find_controller(&prop.room) {
            match site.resolve() {
                Some(controller) => match prop.creep.upgrade_controller(&controller) {
                    Ok(_) => {
                        prop.ctx.role.change_action(ActionStatus::Upgrade);
                        return Ok(Some(()));
                    }
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            prop.ctx.role.change_action(ActionStatus::Upgrade);
                            match utils::line::route_option(
                                &prop.creep,
                                &controller,
                                utils::line::LineStatus::Building,
                            ) {
                                Ok(_) => {
                                    return Ok(Some(()));
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
        Ok(None)
    }

    /// 将资源存储进容器
    fn carry_down(&mut self, option: Option<FindStoreOption>) -> anyhow::Result<Option<()>> {
        let prop = self.get_creep_mut();
        if !prop.ctx.role.check(ActionStatus::CarryDown) {
            return Ok(None);
        }
        if let Some(store) = utils::find::find_store(&prop.creep, &prop.room, option) {
            if let Some(transfer) = store.as_transferable() {
                match prop.creep.transfer(transfer, ResourceType::Energy, None) {
                    Ok(_) => {
                        prop.ctx.role.change_action(ActionStatus::CarryDown);
                        return Ok(Some(()));
                    }
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            prop.ctx.role.change_action(ActionStatus::CarryDown);
                            match utils::line::route_option(
                                &prop.creep,
                                &store.as_structure(),
                                utils::line::LineStatus::Building,
                            ) {
                                Ok(_) => {
                                    return Ok(Some(()));
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
        Ok(None)
    }

    /// 从存储点取能量
    fn carry_up(&mut self) -> anyhow::Result<Option<()>> {
        let prop = self.get_creep_mut();
        if !prop.ctx.role.check(ActionStatus::CarryUp) {
            return Ok(None);
        }

        let prop = self.get_creep_mut();
        if let Some(structure) =
            utils::find::find_store(&prop.creep, &prop.room, Some(FindStoreOption::carry_up()))
        {
            if let Some(store) = structure.as_withdrawable() {
                match prop.creep.withdraw(
                    store,
                    ResourceType::Energy,
                    Some(
                        prop.creep
                            .store()
                            .get_free_capacity(None)
                            as u32,
                    ),
                ) {
                    Ok(_) => {
                        prop.ctx.role.change_action(ActionStatus::CarryUp);
                        return Ok(Some(()));
                    }
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            prop.ctx.role.change_action(ActionStatus::CarryUp);
                            match utils::line::route_option(
                                &prop.creep,
                                &structure.as_structure(),
                                utils::line::LineStatus::Carry,
                            ) {
                                Ok(_) => return Ok(Some(())),
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
        Ok(None)
    }
}
