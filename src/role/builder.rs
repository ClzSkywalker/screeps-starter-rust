use log::*;

use crate::utils::{errorx::ScreepError, find};

use super::{action::ICreepAction, creep::CreepProp, IRoleAction};

pub struct Builder {
    pub creep: CreepProp,
}

impl ICreepAction for Builder {
    fn get_creep(&self) -> &CreepProp {
        &self.creep
    }

    fn get_creep_mut(&mut self) -> &mut CreepProp {
        &mut self.creep
    }
}

impl IRoleAction for Builder {
    fn new(creep: CreepProp) -> impl IRoleAction {
        Builder { creep }
    }

    fn work_line(&mut self) -> anyhow::Result<()> {
        match self.withdraw() {
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

        match self.transfer(Some(find::FindStoreOption::carry_down())) {
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

