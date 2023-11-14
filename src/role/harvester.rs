use screeps::{look, HasPosition, StructureType};

use crate::utils::{self, errorx::ScreepError, find};

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

        // 如果 controller 等级小于2，周围10格内有spawn则运输能量到spawn
        if self.creep.room.controller().unwrap().level() < 2 {
            let count = utils::find::get_area_range(
                &self.creep.room,
                look::STRUCTURES,
                self.creep.creep.pos(),
                10,
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
                match self.store(Some(find::FindStoreOption::carry_down())) {
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

        // 把能量存储进一定范围内的容器
        let count = utils::find::get_area_range(
            &self.creep.room,
            look::STRUCTURES,
            self.creep.creep.pos(),
            10,
        )
        .iter()
        .filter(|item| match &item.look_result {
            look::LookResult::Structure(a) => {
                matches!(a.structure_type(), StructureType::Container)
            }
            _ => false,
        })
        .count();
        if count > 0 {
            match self.store(Some(find::FindStoreOption::harvester_build())) {
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

