use log::*;

use super::{action::CreepAction, creep::CreepProp};

pub struct Harvester {
    pub creep: CreepProp,
}

impl CreepAction for Harvester {
    fn get_creep(&self) -> &CreepProp {
        &self.creep
    }

    fn get_creep_mut(&mut self) -> &mut CreepProp {
        &mut self.creep
    }
}

impl Harvester {
    pub fn new(creep: CreepProp) -> Harvester {
        Harvester { creep }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        if !self.check() {
            return Ok(());
        }

        self.set_status();

        self.say();

        if let Err(e) = self.harveste() {
            warn!("{:?}", e);
            return Err(e);
        };

        if let Err(e) = self.build() {
            warn!("{:?}", e);
            return Err(e);
        };

        if let Err(e) = self.store() {
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
