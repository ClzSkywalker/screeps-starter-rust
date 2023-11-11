use screeps::{Creep, SharedCreepProperties};
use serde::{Deserialize, Serialize};

use crate::role::RoleEnum;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreepMemory {
    pub name: String,
    pub role: RoleEnum,
    pub status: CreepStatus,
    pub store_status: StoreStatus,
}

impl CreepMemory {
    pub fn new(creep: &Creep) -> Self {
        Self {
            name: creep.name().to_string(),
            role: Default::default(),
            status: Default::default(),
            store_status: StoreStatus::new(creep),
        }
    }
}

impl ToString for CreepMemory {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
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
        StoreStatus::Empty
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, strum::Display)]
pub enum CreepStatus {
    // 收割中
    #[default]
    #[strum(serialize = "🔄harvest")]
    Harversting,
    // 建造中🚧 build
    #[strum(serialize = "🚧build")]
    Building,
    // 资源未找到
    #[strum(serialize = "notfound")]
    SourceNotfound,
    // 到容器中寻找能量
    #[strum(serialize = "♋carryUp")]
    CarryUp,
    // 把能量放下
    #[strum(serialize = "♒carryDown")]
    CarryDown,
    // 升级
    Upgrade,
}
