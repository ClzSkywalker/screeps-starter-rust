use screeps::StructureTower;
use serde::{Deserialize, Serialize};

pub mod action;
pub mod tower;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, strum::Display)]
pub enum StructWorkStatus {
    #[default]
    NoWork,
    Work,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, strum::Display)]
pub enum StructActionStatus {
    // 不工作
    #[default]
    #[strum(serialize = "☹")]
    NoWork,
    // 攻击
    #[strum(serialize = "🤛")]
    Attack,
    // 修复
    #[strum(serialize = "💉")]
    Repair,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructStatus {
    pub struct_status: StructWorkStatus,
    pub struct_action: StructActionStatus,
}

#[derive(Debug, Clone, strum::Display)]
pub enum StructAction {
    #[strum(serialize = "🗼")]
    Tower(StructureTower),
}

