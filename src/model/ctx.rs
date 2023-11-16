use screeps::{Creep, SharedCreepProperties};
use serde::{Deserialize, Serialize};

use crate::role::RoleEnum;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreepMemory {
    pub name: String,
    pub role: RoleEnum,
    pub store_status: StoreStatus,
}

impl CreepMemory {
    pub fn new(creep: &Creep) -> Self {
        serde_json::from_str(
            creep
                .memory()
                .as_string()
                .unwrap_or("".to_string())
                .as_str(),
        )
        .unwrap_or(Self {
            name: creep.name().to_string(),
            role: RoleEnum::Harvester(crate::role::RoleStatus {
                creep_status: CreepStatus::default(),
                action_status: ActionStatus::default(),
            })
            .default(),
            store_status: StoreStatus::new(creep),
        })
    }
}

impl ToString for CreepMemory {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, strum::Display)]
pub enum ActionStatus {
    // 不工作
    #[default]
    #[strum(serialize = "☹")]
    NoWork,
    // 收割中
    #[strum(serialize = "⛏️")]
    Harversting,
    // 建造中🚧 build
    #[strum(serialize = "🚧")]
    Building,
    // 到容器中寻找能量
    #[strum(serialize = "🐛")]
    CarryUp,
    // 捡起能量
    #[strum(serialize = "🍂")]
    PickUp,
    // 把能量放下
    #[strum(serialize = "🐌")]
    CarryDown,
    // 升级
    #[strum(serialize = "🔥")]
    Upgrade,
    // 修复
    #[strum(serialize = "💉")]
    Repair,
    // 塔攻击
    // #[strum(serialize = "🧨")]
    // TowerAttack,
    // // 塔修复
    // #[strum(serialize = "💉")]
    // TowerRepair,
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
    // 资源未找到
    #[default]
    #[strum(serialize = "☹")]
    SourceNotfound,
    // 使用能量
    #[strum(serialize = "⛏️")]
    UseEnergy,
    // 寻找能量
    #[strum(serialize = "🚧")]
    FindEnergy,
}

