use serde::{Deserialize, Serialize};
use strum::EnumString;

pub mod builder;
pub mod harvester;
pub mod carrier;
pub mod upgrade_controller;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumString, strum::Display)]
pub enum RoleEnum {
    Harvester,
    UpgradeController,
    Builder,
    // 搬运工
    Porter,
}

impl Default for RoleEnum {
    fn default() -> Self {
        RoleEnum::Harvester
    }
}
