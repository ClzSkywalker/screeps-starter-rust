use serde::{Deserialize, Serialize};
use strum::EnumString;

pub mod harvester;
pub mod upgrade_controller;

#[derive(Debug, PartialEq,Serialize, Deserialize, EnumString,strum::Display)]
pub enum RoleEnum {
    Harvester,
    UpgradeController,
    Builder,
}
