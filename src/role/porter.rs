use log::*;

use crate::utils::{errorx::ScreepError, find};

use super::{action::ICreepAction, creep::CreepProp, IRoleAction};

/// 搬运工
pub struct Porter {
    pub creep: CreepProp,
}

impl ICreepAction for Porter {
    fn get_creep(&self) -> &CreepProp {
        &self.creep
    }

    fn get_creep_mut(&mut self) -> &mut CreepProp {
        &mut self.creep
    }
}

impl IRoleAction for Porter {
    fn new(creep: CreepProp) -> Porter {
        Porter { creep }
    }

    fn work_line(&mut self) -> anyhow::Result<()> {
        match self.pickup() {
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

        match self.transfer(Some(find::FindStoreOption::porter_down())) {
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
