// use screeps::{
//     Creep, ErrorCode, MoveToOptions, ObjectId, PolyStyle, SharedCreepProperties,
//     StructureController,
// };

// use log::*;

// use super::RoleEnum;


// pub struct Builder<'a> {
//     pub creep: &'a Creep,
//     pub structure: &'a ObjectId<StructureController>,
// }

// impl<'a> Builder<'a> {
//     pub fn role() -> RoleEnum {
//         return RoleEnum::Builder;
//     }
//     pub fn new(creep: &'a Creep, structure: &'a ObjectId<StructureController>) -> Builder<'a> {
//         Builder { creep, structure }
//     }

//     pub fn check(&self) -> bool {
//         if self.creep.fatigue() > 0 {
//             return false;
//         }
//         true
//     }

//     pub fn build(&self) -> Result<(), ErrorCode> {
//         if !self.check() {
//             return Ok(());
//         }
//         match self.structure.resolve() {
//             Some(controller) => match self.creep.upgrade_controller(&controller) {
//                 Ok(_) => Ok(()),
//                 Err(e) => match e {
//                     ErrorCode::NotInRange => {
//                         match self.creep.move_to(&controller) {
//                             Ok(_) => {}
//                             Err(e) => {
//                                 warn!("{:?}", e);
//                                 return Err(e);
//                             }
//                         };
//                         // 样式设计
//                         match self.creep.move_to_with_options(
//                             &controller,
//                             Some(
//                                 MoveToOptions::new().visualize_path_style(
//                                     PolyStyle::default()
//                                         .line_style(screeps::LineDrawStyle::Solid)
//                                         .stroke("#fad05a"),
//                                 ),
//                             ),
//                         ) {
//                             Ok(_) => {}
//                             Err(e) => {
//                                 warn!("{:?}", e);
//                                 return Err(e);
//                             }
//                         }
//                         Ok(())
//                     }
//                     _ => {
//                         warn!("couldn't upgrade: {:?}", e);
//                         Err(e)
//                     }
//                 },
//             },
//             None => {
//                 warn!("constroller not found");
//                 Err(ErrorCode::InvalidArgs)
//             }
//         }
//     }
// }
