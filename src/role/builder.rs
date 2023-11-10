// use log::warn;
// use screeps::{
//     ConstructionSite, Creep, ErrorCode, MoveToOptions, ObjectId, PolyStyle, ResourceType,
//     SharedCreepProperties,
// };

// use crate::model::model::{CreepSourceStatus, CreepStatus};

// use super::RoleEnum;

// pub struct Builder<'a> {
//     pub creep: &'a Creep,
//     pub source: &'a ObjectId<ConstructionSite>,
//     pub status: CreepSourceStatus,
// }

// impl<'a> Builder<'a> {
//     pub fn role() -> RoleEnum {
//         return RoleEnum::Builder;
//     }

//     pub fn new(creep: &'a Creep, source: &'a ObjectId<ConstructionSite>) -> Builder<'a> {
//         Builder {
//             creep,
//             source,
//             status: CreepSourceStatus::Harversting,
//         }
//     }

//     pub fn check(&self) -> bool {
//         if self.creep.fatigue() > 0 {
//             return false;
//         }
//         true
//     }

//     // 是否该移除该任务
//     pub fn is_remove_task(&self) -> bool {
//         match self.status {
//             CreepSourceStatus::Full
//             | CreepSourceStatus::Empty
//             | CreepSourceStatus::SourceNotfound => {
//                 return true;
//             }
//             _ => return false,
//         }
//     }

//     // 有资源则收割资源，没有则移除资源点
//     pub fn build(&mut self) -> Result<(), ErrorCode> {
//         if !self.check() {
//             return Ok(());
//         }

//         if self
//             .creep
//             .store()
//             .get_free_capacity(Some(ResourceType::Energy))
//             == 0
//         {
//             self.status = CreepSourceStatus::Full;
//             return Ok(());
//         }
//         match self.source.resolve() {
//             Some(s) => {
//                 match self.creep.move_to(s.clone()) {
//                     Ok(_) => {}
//                     Err(e) => {
//                         warn!("{:?}", e);
//                         return Err(e);
//                     }
//                 };

//                 // 样式设计
//                 match self.creep.move_to_with_options(
//                     &s,
//                     Some(
//                         MoveToOptions::new().visualize_path_style(
//                             PolyStyle::default()
//                                 .line_style(screeps::LineDrawStyle::Solid)
//                                 .stroke("#f9cf59"),
//                         ),
//                     ),
//                 ) {
//                     Ok(_) => {}
//                     Err(e) => {
//                         warn!("{:?}", e);
//                         return Err(e);
//                     }
//                 }
//             }
//             // 资源不存在
//             None => {
//                 self.status = CreepSourceStatus::SourceNotfound;
//                 return Ok(());
//             }
//         }
//         Ok(())
//     }
// }
