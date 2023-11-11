use serde::{Deserialize, Serialize};
use strum::EnumString;

pub mod builder;
pub mod carrier;
pub mod harvester;
pub mod upgrader;

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    EnumString,
    strum::Display,
)]
pub enum RoleEnum {
    #[default]
    Harvester,
    Upgrader,
    Builder,
    // 搬运工
    Porter,
}
