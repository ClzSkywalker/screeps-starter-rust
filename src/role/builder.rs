use screeps::{
    ConstructionSite, Creep, ErrorCode, MoveToOptions, ObjectId, PolyStyle, SharedCreepProperties,
    StructureController,
};

use log::*;

const ROLE_BUILDER: &str = "Builder";

pub struct Builder<'a> {
    pub creep: &'a Creep,
    pub structure: &'a ObjectId<StructureController>,
}

impl<'a> Builder<'a> {
    pub fn role() -> &'a str {
        return ROLE_BUILDER;
    }

    pub fn new(creep: &'a Creep, structure: &'a ObjectId<StructureController>) -> Builder<'a> {
        Builder { creep, structure }
    }

    pub fn build(&self) -> Result<(), ErrorCode> {
        match self.structure.resolve() {
            Some(controller) => match self.creep.upgrade_controller(&controller) {
                Ok(_) => Ok(()),
                Err(e) => match e {
                    ErrorCode::NotInRange => {
                        match self.creep.move_to(&controller) {
                            Ok(_) => {}
                            Err(e) => return Err(e),
                        };
                        // 样式设计
                        self.creep
                            .move_to_with_options(
                                &controller,
                                Some(
                                    MoveToOptions::new().visualize_path_style(
                                        PolyStyle::default()
                                            .line_style(screeps::LineDrawStyle::Solid)
                                            .stroke("#fad05a"),
                                    ),
                                ),
                            )
                            .expect("creep desciption");
                        Ok(())
                    }
                    _ => {
                        warn!("couldn't upgrade: {:?}", e);
                        Err(e)
                    }
                },
            },
            None => Err(ErrorCode::InvalidArgs),
        }
    }
}
