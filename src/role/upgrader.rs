use log::*;

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

        self.set_status();

        if let Err(e) = self.carry_up() {
            warn!("{:?}", e);
            return Err(e);
        };

        if let Err(e) = self.upgrade() {
            warn!("{:?}", e);
            return Err(e);
        };

        self.set_memory();

        Ok(())
    }
}
