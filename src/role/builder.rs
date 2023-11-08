use screeps::{
    Creep, ErrorCode, MoveToOptions, ObjectId, PolyStyle, SharedCreepProperties,
    StructureController,
};

use log::*;

pub struct Builder<'a> {
    pub creep: &'a Creep,
    pub structure: &'a ObjectId<StructureController>,
}

impl<'a> Builder<'a> {
    pub fn new(creep: &'a Creep, structure: &'a ObjectId<StructureController>) -> Builder<'a> {
        return Builder {
            creep: creep,
            structure: structure,
        };
    }

    pub fn build(&self) -> Result<(), ErrorCode> {
        match self.structure.resolve() {
            Some(controller) => match self.creep.upgrade_controller(&controller) {
                Ok(_) => return Ok(()),
                Err(e) => match e {
                    ErrorCode::NotInRange => {
                        self.creep.move_to(&controller).expect("can not move");
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
                        return Ok(());
                    }
                    _ => {
                        warn!("couldn't upgrade: {:?}", e);
                        return Err(e);
                    }
                },
            },
            None => return Err(ErrorCode::InvalidArgs),
        }
    }
}
