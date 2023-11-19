use log::*;

use crate::utils::{
    errorx::ScreepError,
    find::{self, FindStoreOption},
};

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
    fn new(creep: CreepProp) -> Self {
        Self { creep }
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

    fn run(&mut self) -> anyhow::Result<()> {
        if !self.check() {
            return Ok(());
        }

        self.set_status();

        if let Err(e) = self.work_line() {
            warn!("{:?}", e);
            return Err(e);
        }

        self.say();
        self.cancel_bind_structure();
        self.set_memory();

        Ok(())
    }
}

