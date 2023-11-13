use log::*;

use crate::utils::{errorx::ScreepError, find};

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
                warn!("{:?}", e);
                return Err(e);
            }
        }

        // 应当改变角色的
        match self.carry_up() {
            Ok(r) => {
                if r.is_some() {
                    return Ok(());
                }
            }
            Err(e) => {
                warn!("{:?}", e);
                return Err(e);
            }
        }

        match self.store(Some(find::FindStoreOption::harvester_build())) {
            Ok(r) => {
                if r.is_some() {
                    return Ok(());
                }
            }
            Err(e) => {
                warn!("{:?}", e);
                return Err(e);
            }
        }

        match self.build() {
            Ok(r) => {
                if r.is_some() {
                    return Ok(());
                }
            }
            Err(e) => {
                warn!("{:?}", e);
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
                warn!("{:?}", e);
                return Err(e);
            }
        }
        info!(
            "{}",
            ScreepError::RoleCanNotWork(self.creep.ctx.role.to_string())
        );
        Ok(())
    }
}

