use screeps::{
    Creep, ErrorCode, HasPosition, MoveToOptions, ObjectId, PolyStyle, ResourceType,
    SharedCreepProperties, Source,
};

use log::*;

pub struct Harverster<'a> {
    pub creep: &'a Creep,
    pub source: &'a ObjectId<Source>,
}

impl<'a> Harverster<'a> {
    pub fn new(creep: &'a Creep, source: &'a ObjectId<Source>) -> Harverster<'a> {
        return Harverster {
            creep: creep,
            source: source,
        };
    }

    // 有资源则收割资源，没有则移除资源点
    pub fn harveste(&self) -> Result<(), ErrorCode> {
        if self
            .creep
            .store()
            .get_free_capacity(Some(ResourceType::Energy))
            == 0
        {
            return Err(ErrorCode::InvalidArgs);
        }
        match self.source.resolve() {
            Some(s) => {
                // 资源在附近则收割资源
                if self.creep.pos().is_near_to(s.pos()) {
                    match self.creep.harvest(&s) {
                        Ok(_) => {}
                        Err(e) => return Err(e),
                    };
                } else {
                    // 移动到资源附近
                    self.creep.move_to(&s).expect("can not move");
                    // 样式设计
                    self.creep.move_to_with_options(
                        &s,
                        Some(
                            MoveToOptions::new().visualize_path_style(
                                PolyStyle::default()
                                    .line_style(screeps::LineDrawStyle::Solid)
                                    .stroke("#07a125"),
                            ),
                        ),
                    ).expect("creep desciption");
                    return Ok(());
                }
            }
            // 资源不存在
            None => return Err(ErrorCode::InvalidArgs),
        }
        return Ok(());
    }
}
