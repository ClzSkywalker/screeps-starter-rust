use screeps::{game, look, HasPosition, StructureType};

use crate::{
    global,
    manager::source_manager::{StructureInfo, StructureInfoEnum},
    utils::{
        self,
        errorx::ScreepError,
        find::{self, FindStoreOption},
    },
};

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
    fn new(creep: CreepProp) -> Self {
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
                log::warn!("{:?}", e);
                return Err(e);
            }
        }

        match self.withdraw(Some(FindStoreOption::builder_up())) {
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

        // 如果 controller 等级小于2，则运输资源给控制器升级
        if self.creep.room.controller().unwrap().level() < 2 {
            match self.upgrade() {
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
        }

        // 如果当前creep的数目小于最大挖掘人数，则运输资源到spawn/ext
        let creep_count = game::creeps().keys().count();
        let create_creep = global::SOURCE_MANAGER.with(|manager| {
            let manager = manager.borrow();
            if let Some(r) = manager.room_item.get(&self.creep.room.name().to_string()) {
                // 需要一个额外的Porter帮忙生产
                return creep_count
                    < *r.source_max_map
                        .get(&StructureInfoEnum::Source(StructureInfo::default()).to_string())
                        .unwrap_or(&usize::default())
                        + 1;
            }
            false
        });

        if create_creep {
            let count = utils::find::get_area_range(
                &self.creep.room,
                look::STRUCTURES,
                self.creep.creep.pos(),
                25,
            )
            .iter()
            .filter(|item| match &item.look_result {
                look::LookResult::Structure(a) => matches!(
                    a.clone().structure_type(),
                    StructureType::Spawn | StructureType::Extension
                ),
                _ => false,
            })
            .count();
            if count > 0 {
                match self.transfer(Some(find::FindStoreOption::spawn_ext_down())) {
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
            }
        }

        match self.transfer(Some(find::FindStoreOption::harvester_store())) {
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

        match self.build() {
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

        match self.upgrade() {
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

        log::info!(
            "{}",
            ScreepError::RoleCanNotWork(self.creep.ctx.role.to_string())
        );
        Ok(())
    }
}

