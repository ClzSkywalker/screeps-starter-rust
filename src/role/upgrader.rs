use log::*;

use crate::utils::{errorx::ScreepError, find::FindStoreOption};

use super::{action::ICreepAction, creep::CreepProp, IRoleAction};

pub struct Upgrader {
    pub creep: CreepProp,
}

impl ICreepAction for Upgrader {
    fn get_creep(&self) -> &CreepProp {
        &self.creep
    }

    fn get_creep_mut(&mut self) -> &mut CreepProp {
        &mut self.creep
    }
}

impl IRoleAction for Upgrader {
    fn new(creep: CreepProp) -> Self {
        Upgrader { creep }
    }

    fn work_line(&mut self) -> anyhow::Result<()> {
        match self.withdraw(Some(FindStoreOption::builder_up())) {
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

        info!(
            "{}",
            ScreepError::RoleCanNotWork(self.creep.ctx.role.get_role_name())
        );
        Ok(())
    }
}

