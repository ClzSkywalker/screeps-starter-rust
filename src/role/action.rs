use screeps::{ErrorCode, ResourceType, SharedCreepProperties, StructureType};
use wasm_bindgen::JsValue;

use crate::{
    global::SOURCE_MANAGER,
    model::ctx::{ActionStatus, CreepStatus, StoreStatus},
    utils::{self, errorx::ScreepError, find::FindStoreOption},
};

use super::creep::CreepProp;

pub trait ICreepAction {
    fn get_creep(&self) -> &CreepProp;
    fn get_creep_mut(&mut self) -> &mut CreepProp;

    fn say(&self) {
        let prop = self.get_creep();
        let text = prop.ctx.role.get_say_test();
        if text.is_empty() {
            return;
        }
        if let Some(e) = prop.creep.say(text.as_str(), false).err() {
            log::warn!("{:?}", e);
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

    /// 取消绑定
    fn cancel_bind_structure(&self) {
        let prop = self.get_creep();
        if prop.ctx.role.creep_status != CreepStatus::UseEnergy {
            return;
        }
        SOURCE_MANAGER.with(|manager| {
            let mut manager = manager.borrow_mut();
            let room_id = prop.room.name().to_string();
            let creep_id = prop.creep.name().to_string();
            if prop.ctx.role.is_cancel_bind() {
                manager.cancel_bind(room_id, creep_id);
            }
        });
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
                                log::warn!("{:?}", e);
                                return Err(ScreepError::ScreepInner.into());
                            }
                        }
                    }
                    _ => {
                        log::warn!("{:?}", e);
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
                                log::warn!("{:?}", e);
                                return Err(ScreepError::ScreepInner.into());
                            }
                        }
                    }
                    _ => {
                        log::warn!("{:?}", e);
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
                                log::warn!("{:?}", e);
                                return Err(ScreepError::ScreepInner.into());
                            }
                        }
                    }
                    _ => {
                        log::warn!("{:?}", e);
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
                                log::warn!("{:?}", e);
                                return Err(ScreepError::ScreepInner.into());
                            }
                        }
                    }
                    _ => {
                        log::warn!("{:?}", e);
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

        if let Some(controller) = utils::find::find_controller(&prop.room) {
            match prop.creep.upgrade_controller(&controller) {
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
                                log::warn!("{:?}", e);
                                return Err(ScreepError::ScreepInner.into());
                            }
                        }
                    }
                    _ => {
                        log::warn!("{:?}", e);
                        return Err(ScreepError::ScreepInner.into());
                    }
                },
            }
        };
        Ok(None)
    }

    /// 将资源存储进容器
    fn transfer(&mut self, option: Option<FindStoreOption>) -> anyhow::Result<Option<()>> {
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
                                    log::warn!("{:?}", e);
                                    return Err(ScreepError::ScreepInner.into());
                                }
                            }
                        }
                        _ => {
                            log::warn!("{:?}:{:?}", e, store.as_structure());
                            return Err(ScreepError::ScreepInner.into());
                        }
                    },
                }
            }
        }
        Ok(None)
    }

    /// 从存储点取能量
    fn withdraw(&mut self, option: Option<FindStoreOption>) -> anyhow::Result<Option<()>> {
        let prop = self.get_creep_mut();
        if !prop.ctx.role.check(ActionStatus::CarryUp) {
            return Ok(None);
        }

        let prop = self.get_creep_mut();
        if let Some(structure) = utils::find::find_store(&prop.creep, &prop.room, option) {
            if let Some(store) = structure.as_withdrawable() {
                match prop.creep.withdraw(store, ResourceType::Energy, None) {
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
                                    log::warn!("{:?}", e);
                                    return Err(ScreepError::ScreepInner.into());
                                }
                            }
                        }
                        _ => {
                            log::error!("{:?}", e);
                            return Err(ScreepError::ScreepInner.into());
                        }
                    },
                };
            };
        };
        Ok(None)
    }

    /// 使用携带的能量修复受损建筑。需要 WORK 和 CARRY 身体部件。目标必须位于以 creep 为中心的 7*7 正方形区域内。
    fn repair_rampart(&mut self) -> anyhow::Result<Option<()>> {
        let prop = self.get_creep_mut();
        if !prop.ctx.role.check(ActionStatus::Repair) {
            return Ok(None);
        }

        let site = SOURCE_MANAGER.with(|manager| {
            let mut manager = manager.borrow_mut();
            manager.find_and_bind_rampart(prop.room.name().to_string(), &prop.creep)
        });

        let site = match site {
            Some(r) => r,
            None => return Ok(None),
        };

        match prop.creep.repair(&site) {
            Ok(_) => {
                prop.ctx.role.change_action(ActionStatus::Repair);
                Ok(Some(()))
            }
            Err(e) => match e {
                ErrorCode::NotInRange => {
                    prop.ctx.role.change_action(ActionStatus::Repair);
                    match utils::line::route_option(
                        &prop.creep,
                        &site,
                        utils::line::LineStatus::Carry,
                    ) {
                        Ok(_) => Ok(Some(())),
                        Err(e) => {
                            log::warn!("{:?}", e);
                            Err(ScreepError::ScreepInner.into())
                        }
                    }
                }
                _ => {
                    log::error!("{:?}", e);
                    Err(ScreepError::ScreepInner.into())
                }
            },
        }
    }

    fn repair(&mut self) -> anyhow::Result<Option<()>> {
        let prop = self.get_creep_mut();
        if !prop.ctx.role.check(ActionStatus::Repair) {
            return Ok(None);
        }
        let structure_list = utils::find::find_need_repair(&prop.room);
        let structure_list = utils::find::priority_structure(
            structure_list,
            vec![
                StructureType::Tower,
                StructureType::Storage,
                StructureType::Container,
                StructureType::Extension,
                StructureType::Wall,
                StructureType::Road,
            ],
        );
        match utils::find::get_near_site(&prop.creep, &structure_list) {
            Some(site) => match prop.creep.repair(site.as_structure()) {
                Ok(_) => {
                    prop.ctx.role.change_action(ActionStatus::Repair);
                    Ok(Some(()))
                }
                Err(e) => match e {
                    ErrorCode::NotInRange => {
                        prop.ctx.role.change_action(ActionStatus::Repair);
                        match utils::line::route_option(
                            &prop.creep,
                            &site.as_structure(),
                            utils::line::LineStatus::Carry,
                        ) {
                            Ok(_) => Ok(Some(())),
                            Err(e) => {
                                log::warn!("{:?}", e);
                                Err(ScreepError::ScreepInner.into())
                            }
                        }
                    }
                    _ => {
                        log::error!("{:?}", e);
                        Err(ScreepError::ScreepInner.into())
                    }
                },
            },
            None => Ok(None),
        }
    }
}

