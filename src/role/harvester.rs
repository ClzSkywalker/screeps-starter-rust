use screeps::{game, look, HasPosition, StructureType};

use crate::{
    global,
    utils::{self, errorx::ScreepError, find},
};

use super::{action::ICreepAction, creep::CreepProp, IRoleAction};

pub struct Harvester {
    pub creep: CreepProp,
}

impl ICreepAction for Harvester {
    fn get_creep(&self) -> &CreepProp {
        &self.creep
    }

    fn get_creep_mut(&mut self) -> &mut CreepProp {
        &mut self.creep
    }
}

impl IRoleAction for Harvester {
    fn new(creep: CreepProp) -> impl IRoleAction {
        Harvester { creep }
    }

    fn work_line(&mut self) -> anyhow::Result<()> {
        match self.harveste() {
            Ok(r) => {
                if r.is_some() {
                    return Ok(());
                }
            }
            Err(e) => {
                log::warn!("{:?}", e);
                return Err(e);
            }
        }

        match self.carry_up() {
            Ok(r) => {
                if r.is_some() {
                    return Ok(());
                }
            }

            Err(e) => {
                log::warn!("{:?}", e);
                return Err(e);
            }
        }

        match self.carry_down(Some(find::FindStoreOption::harvester_store())) {
            Ok(r) => {
                if r.is_some() {
                    return Ok(());
                }
            }
            Err(e) => {
                log::warn!("{:?}", e);
                return Err(e);
            }
        }

        // 如果 controller 等级小于2，则运输资源给控制器升级
        if self.creep.room.controller().unwrap().level() < 2 {
            match self.upgrade() {
                Ok(r) => {
                    if r.is_some() {
                        return Ok(());
                    }
                }
                Err(e) => {
                    log::warn!("{:?}", e);
                    return Err(e);
                }
            }
        }

        // 如果当前creep的数目小于最大挖掘人数，则运输资源到spawn
        let creep_count = game::creeps().keys().count();
        let create_creep = global::SOURCE_MANAGER.with(|manager| {
            let manager = manager.borrow();
            if let Some(r) = manager.room_item.get(&self.creep.room.name().to_string()) {
                return creep_count < r.max_count;
            }
            false
        });

        if create_creep {
            let count = utils::find::get_area_range(
                &self.creep.room,
                look::STRUCTURES,
                self.creep.creep.pos(),
                25,
            )
            .iter()
            .filter(|item| match &item.look_result {
                look::LookResult::Structure(a) => {
                    if a.clone().structure_type() == StructureType::Spawn {
                        return true;
                    }
                    false
                }
                _ => false,
            })
            .count();
            if count > 0 {
                match self.carry_down(Some(find::FindStoreOption::carry_down())) {
                    Ok(r) => {
                        if r.is_some() {
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        log::warn!("{:?}", e);
                        return Err(e);
                    }
                }
            }
        }

        match self.build() {
            Ok(r) => {
                if r.is_some() {
                    return Ok(());
                }
            }
            Err(e) => {
                log::warn!("{:?}", e);
                return Err(e);
            }
        }

        match self.upgrade() {
            Ok(r) => {
                if r.is_some() {
                    return Ok(());
                }
            }
            Err(e) => {
                log::warn!("{:?}", e);
                return Err(e);
            }
        }

        log::info!(
            "{}",
            ScreepError::RoleCanNotWork(self.creep.ctx.role.to_string())
        );
        Ok(())
    }
}

