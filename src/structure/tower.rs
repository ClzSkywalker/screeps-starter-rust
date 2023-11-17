use screeps::{StructureObject, StructureTower};

use super::{action::IStructureAction, IStructureActionRun};

pub struct Tower {
    pub site: StructureTower,
}

impl Tower {
    pub fn new(site: StructureTower) -> Self {
        Self { site }
    }
}

impl IStructureActionRun for Tower {
    fn work_line(&mut self) -> anyhow::Result<()> {
        match self.attack() {
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
        Ok(())
    }
}

impl IStructureAction for Tower {
    fn get_site(&self) -> screeps::StructureObject {
        StructureObject::from(self.site.clone())
    }
}
