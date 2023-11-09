use log::*;
use screeps::{
    Creep, ErrorCode, HasPosition, MoveToOptions, ObjectId, PolyStyle, ResourceType,
    SharedCreepProperties, Source,
};

use crate::model::model::HarversterStatus;

use super::RoleEnum;

pub struct Harverster<'a> {
    pub creep: &'a Creep,
    pub source: &'a ObjectId<Source>,
    pub status: HarversterStatus,
}

impl<'a> Harverster<'a> {
    pub fn role() -> RoleEnum {
        return RoleEnum::Harvester;
    }

    pub fn new(creep: &'a Creep, source: &'a ObjectId<Source>) -> Harverster<'a> {
        Harverster {
            creep,
            source,
            status: HarversterStatus::Default,
        }
    }

    pub fn check(&self) -> bool {
        if self.creep.fatigue() > 0 {
            return false;
        }
        true
    }

    // 是否该移除该任务
    pub fn is_remove_task(&self) -> bool {
        match self.status {
            HarversterStatus::Full
            | HarversterStatus::SourceEmpty
            | HarversterStatus::SourceNotfound => {
                return true;
            }
            _ => return false,
        }
    }

    // 有资源则收割资源，没有则移除资源点
    pub fn harveste(&mut self) -> Result<(), ErrorCode> {
        if !self.check() {
            return Ok(());
        }

        if self
            .creep
            .store()
            .get_free_capacity(Some(ResourceType::Energy))
            == 0
        {
            self.status = HarversterStatus::Full;
            return Ok(());
        }
        match self.source.resolve() {
            Some(s) => {
                if s.energy() == 0 {
                    self.status = HarversterStatus::SourceEmpty;
                    return Err(ErrorCode::NotEnough);
                }
                // 资源在附近则收割资源
                if self.creep.pos().is_near_to(s.pos()) {
                    match self.creep.harvest(&s) {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("{:?}", e);
                            return Err(e);
                        }
                    };
                } else {
                    // 移动到资源附近
                    match self.creep.move_to(&s) {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("{:?}", e);
                            return Err(e);
                        }
                    }
                    // 样式设计
                    match self.creep.move_to_with_options(
                        &s,
                        Some(
                            MoveToOptions::new().visualize_path_style(
                                PolyStyle::default()
                                    .line_style(screeps::LineDrawStyle::Solid)
                                    .stroke("#07a125"),
                            ),
                        ),
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("{:?}", e);
                            return Err(e);
                        }
                    }
                    return Ok(());
                }
            }
            // 资源不存在
            None => {
                self.status = HarversterStatus::SourceNotfound;
                return Ok(());
            }
        }
        Ok(())
    }
}
