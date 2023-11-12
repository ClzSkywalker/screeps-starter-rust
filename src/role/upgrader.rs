use log::*;

use crate::utils::errorx::ScreepError;

use super::{action::CreepAction, creep::CreepProp};

pub struct Upgrader {
    pub creep: CreepProp,
}

impl CreepAction for Upgrader {
    fn get_creep(&self) -> &CreepProp {
        &self.creep
    }

    fn get_creep_mut(&mut self) -> &mut CreepProp {
        &mut self.creep
    }
}

impl Upgrader {
    // pub fn role() -> RoleEnum {
    //     return RoleEnum::Upgrader;
    // }
    pub fn new(creep: CreepProp) -> Upgrader {
        Upgrader { creep }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        if !self.check() {
            return Ok(());
        }

        self.set_status();

        self.say();

        if let Err(e) = self.work_line() {
            warn!("{:?}", e);
            return Err(e);
        }
        self.set_memory();

        Ok(())
    }

    fn work_line(&mut self) -> anyhow::Result<()> {
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
            ScreepError::RoleCanNotWork(self.creep.ctx.role.to_string())
        );
        Ok(())
    }
}
