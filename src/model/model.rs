use screeps::{Creep, ObjectId, Source, StructureController, StructureExtension};
use serde::{Deserialize, Serialize};

use crate::role::RoleEnum;

#[derive(Debug, Default)]
pub struct ManagerRoleCount {
    pub harvester: i32,
    pub upgrade: i32,
    pub builder: i32,
}

impl ManagerRoleCount {
    pub fn get_role(&self) -> RoleEnum {
        let mut tmp_role = RoleEnum::Harvester;
        let mut tmp_count = self.harvester;
        if tmp_count > self.upgrade {
            tmp_role = RoleEnum::UpgradeController;
            tmp_count = self.upgrade;
        }
        if tmp_count > self.builder {
            tmp_role = RoleEnum::Builder;
            tmp_count = self.builder;
        }
        return tmp_role;
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreepMemory {
    pub name: String,
    pub role: RoleEnum,
    pub status: CreepStatus,
    pub store_status: StoreStatus,
}

#[derive(Serialize, Deserialize)]
pub enum CreepStatus {
    Default,
    Building,
    Upgrading,
    Harverst,
}

#[derive(Serialize, Deserialize)]
pub enum StoreStatus {
    UnderFill,
    Full,
    Empty,
}

impl StoreStatus {
    pub fn new(creep: &Creep) -> StoreStatus {
        if creep.store().get_free_capacity(None) == 0 {
            return StoreStatus::Full;
        }
        if creep.store().get_used_capacity(None) > 0 {
            return StoreStatus::UnderFill;
        }
        return StoreStatus::Full;
    }
}

// this enum will represent a creep's lock on a specific target object, storing a js reference
// to the object id so that we can grab a fresh reference to the object each successive tick,
// since screeps game objects become 'stale' and shouldn't be used beyond the tick they were fetched
#[derive(Clone)]
pub enum CreepTarget {
    // 可升级的控制器
    ControllerUpgrade(ObjectId<StructureController>),
    ExtensionBuild(ObjectId<StructureExtension>),
    // 可收割的资源
    Harvest(ObjectId<Source>),
}

pub enum HarversterStatus {
    Default,
    // 满载
    Full,
    // 资源已空
    SourceEmpty,
    // 资源未找到
    SourceNotfound,
}
impl Default for HarversterStatus {
    fn default() -> Self {
        HarversterStatus::Default
    }
}

// pub enum SourceStatus {
//     Empty,
// }
