use screeps::{Creep, SharedCreepProperties};
use serde::{Deserialize, Serialize};

use crate::role::{creep::CreepProp, RoleEnum};

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

impl CreepStatus {
    /// 根据角色+能源存储状态=角色现在状态
    ///
    /// * `prop`:
    pub fn check(prop: &mut CreepProp) {
        match prop.ctx.role {
            RoleEnum::Harvester => {
                match prop.ctx.store_status {
                    StoreStatus::Empty => {
                        prop.ctx.status = CreepStatus::Harversting;
                    }
                    StoreStatus::Full => {
                        prop.ctx.status = CreepStatus::Building;
                    }
                    _ => {
                        if !matches!(
                            prop.ctx.status,
                            CreepStatus::Harversting | CreepStatus::Building
                        ) {
                            prop.ctx.status = CreepStatus::Harversting;
                        }
                    }
                };
            }
            RoleEnum::Upgrader => {
                match prop.ctx.store_status {
                    StoreStatus::Empty => {
                        prop.ctx.status = CreepStatus::CarryUp;
                    }
                    StoreStatus::Full => {
                        prop.ctx.status = CreepStatus::Building;
                    }
                    _ => {
                        if !matches!(
                            prop.ctx.status,
                            CreepStatus::CarryUp | CreepStatus::Building
                        ) {
                            prop.ctx.status = CreepStatus::CarryUp;
                        }
                    }
                };
            }
            RoleEnum::Builder => match prop.ctx.store_status {
                StoreStatus::Empty => {
                    prop.ctx.status = CreepStatus::CarryUp;
                }
                StoreStatus::Full => {
                    prop.ctx.status = CreepStatus::Building;
                }
                _ => {
                    if !matches!(
                        prop.ctx.status,
                        CreepStatus::CarryUp | CreepStatus::Building
                    ) {
                        prop.ctx.status = CreepStatus::CarryUp;
                    }
                }
            },
            RoleEnum::Porter => match prop.ctx.store_status {
                StoreStatus::Empty => {
                    prop.ctx.status = CreepStatus::CarryUp;
                }
                StoreStatus::Full => {
                    prop.ctx.status = CreepStatus::CarryDown;
                }
                _ => {
                    if !matches!(
                        prop.ctx.status,
                        CreepStatus::CarryUp | CreepStatus::CarryDown
                    ) {
                        prop.ctx.status = CreepStatus::CarryUp;
                    }
                }
            },
        }
    }
}

