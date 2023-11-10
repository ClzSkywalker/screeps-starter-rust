use screeps::Creep;
use serde::{Deserialize, Serialize};

use crate::role::RoleEnum;

#[derive(Debug, Default)]
pub struct ManagerRoleCount {
    pub harvester: i32,
    pub upgrade: i32,
    pub builder: i32,
    pub porter: i32,
}

// impl ManagerRoleCount {
//     pub fn get_role(&self) -> RoleEnum {
//         let mut tmp_role = RoleEnum::Harvester;
//         let mut tmp_count = self.harvester;
//         if tmp_count > self.upgrade {
//             tmp_role = RoleEnum::UpgradeController;
//             tmp_count = self.upgrade;
//         }
//         if tmp_count > self.builder {
//             tmp_role = RoleEnum::Builder;
//             tmp_count = self.builder;
//         }
//         return tmp_role;
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreepMemory {
    pub name: String,
    pub role: RoleEnum,
    pub status: CreepSourceStatus,
    pub store_status: StoreStatus,
}

impl Default for CreepMemory {
    fn default() -> Self {
        Self {
            name: Default::default(),
            role: Default::default(),
            status: Default::default(),
            store_status: Default::default(),
        }
    }
}

impl ToString for CreepMemory {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub enum CreepStatus {
    Building,
    Upgrading,
    Harverst,
}

impl Default for CreepStatus {
    fn default() -> Self {
        Self::Harverst
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StoreStatus {
    UnderFill,
    Full,
    Empty,
}

impl Default for StoreStatus {
    fn default() -> Self {
        Self::Empty
    }
}

impl StoreStatus {
    pub fn new(creep: &Creep) -> StoreStatus {
        if creep.store().get_free_capacity(None) == 0 {
            return StoreStatus::Full;
        }
        if creep.store().get_used_capacity(None) > 0 {
            return StoreStatus::UnderFill;
        }
        return StoreStatus::Empty;
    }
}

// this enum will represent a creep's lock on a specific target object, storing a js reference
// to the object id so that we can grab a fresh reference to the object each successive tick,
// since screeps game objects become 'stale' and shouldn't be used beyond the tick they were fetched
// #[derive(Clone)]
// pub enum CreepTarget {
//     // 可升级的控制器
//     ControllerUpgrade(ObjectId<StructureController>),
//     ConstructionSiteBuild(ObjectId<ConstructionSite>),
//     ExtensionBuild(ObjectId<StructureExtension>),
//     // 可收割的资源
//     Harvest(ObjectId<Source>),
// }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, strum::Display)]
pub enum CreepSourceStatus {
    // 收割中
    #[strum(serialize = "🔄harvest")]
    Harversting,
    // 建造中🚧 build
    #[strum(serialize = "🚧build")]
    Building,
    // 资源未找到
    #[strum(serialize = "Notfound")]
    SourceNotfound,
    #[strum(serialize = "♋carry")]
    CarryUp,
    #[strum(serialize = "♒carry")]
    CarryDown
}
impl Default for CreepSourceStatus {
    fn default() -> Self {
        CreepSourceStatus::Harversting
    }
}
