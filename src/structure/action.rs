use screeps::{find, prelude::*, Position, StructureObject};

use crate::utils::errorx::ScreepError;

pub trait IStructureAction {
    fn get_creep(&self) -> &StructureObject;
    fn get_creep_mut(&mut self) -> &mut StructureObject;

    fn check(&self) -> bool {
        true
    }

    fn set_status(&mut self) {}

    fn attack(&mut self) -> anyhow::Result<Option<()>> {
        let site = self.get_creep_mut();
        if let StructureObject::StructureTower(structure) = site {
            let x = Position::x(structure.pos()).u8();
            let y = Position::y(structure.pos()).u8();
            if let Some(target) = structure
                .room()
                .unwrap()
                .get_position_at(x, y)
                .find_closest_by_range(find::HOSTILE_CREEPS)
            {
                match structure.attack(&target) {
                    Ok(_) => return Ok(Some(())),
                    Err(e) => {
                        log::error!("err:{:?}", e);
                        return Err(ScreepError::ScreepInner.into());
                    }
                }
            }
        };
        Ok(None)
    }

    // todo
    // fn heal(&mut self) -> anyhow::Result<Option<()>> {
    //     let site = self.get_creep_mut();
    //     if let StructureObject::StructureTower(structure) = site {
    //         if let Some(target) = structure
    //             .room()
    //             .unwrap()
    //             .get_position_at(1, 1)
    //             .find_closest_by_range(find::MY_CREEPS)
    //         {
    //             match structure.heal(&target) {
    //                 Ok(_) => return Ok(Some(())),
    //                 Err(e) => {
    //                     log::error!("err:{:?}", e);
    //                     return Err(ScreepError::ScreepInner.into());
    //                 }
    //             }
    //         }
    //     };
    //     Ok(None)
    // }
}
