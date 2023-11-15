use std::fmt::Display;

use log::warn;
use serde::{Deserialize, Serialize};

use crate::model::ctx::{ActionStatus, CreepStatus, StoreStatus};

use self::{action::ICreepAction, creep::CreepProp};

pub mod action;
pub mod builder;
pub mod creep;
pub mod harvester;
pub mod porter;
pub mod upgrader;

pub trait IRoleAction: ICreepAction {
    // 创建实例
    fn new(creep: CreepProp) -> impl IRoleAction;
    // 工作线
    fn work_line(&mut self) -> anyhow::Result<()>;
    // 执行
    fn run(&mut self) -> anyhow::Result<()> {
        if !self.check() {
            return Ok(());
        }

        self.set_status();

        self.say();

        if let Err(e) = self.work_line() {
            warn!("{:?}", e);
            return Err(e);
        }
        self.set_memory();

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoleStatus {
    pub creep_status: CreepStatus,
    pub action_status: ActionStatus,
}

impl Display for RoleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.creep_status, self.action_status)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, strum::Display)]
pub enum RoleEnum {
    #[strum(serialize = "⛏️")]
    Harvester(RoleStatus),
    #[strum(serialize = "🔥")]
    Upgrader(RoleStatus),
    #[strum(serialize = "🚧")]
    Builder(RoleStatus),
    // 搬运工
    #[strum(serialize = "🐌")]
    Porter(RoleStatus),
}

const HARVESTER: &str = "harvester";
const UPGRADER: &str = "upgrade";
const BUILDER: &str = "builder";
const PORTER: &str = "porter";

impl From<String> for RoleEnum {
    fn from(value: String) -> Self {
        match value.as_str() {
            HARVESTER => RoleEnum::Harvester(RoleStatus::default()),
            UPGRADER => RoleEnum::Upgrader(RoleStatus::default()),
            BUILDER => RoleEnum::Builder(RoleStatus::default()),
            PORTER => RoleEnum::Porter(RoleStatus::default()),
            _ => RoleEnum::Harvester(RoleStatus::default()),
        }
    }
}

impl RoleEnum {
    pub fn get_role_name(&self) -> String {
        match self {
            RoleEnum::Harvester(_) => HARVESTER.to_string(),
            RoleEnum::Upgrader(_) => UPGRADER.to_string(),
            RoleEnum::Builder(_) => BUILDER.to_string(),
            RoleEnum::Porter(_) => PORTER.to_string(),
        }
    }

    pub fn get_say_test(&self) -> String {
        match self {
            RoleEnum::Harvester(status) => {
                self.to_string() + status.action_status.to_string().as_str()
            }
            RoleEnum::Upgrader(status) => {
                self.to_string() + status.action_status.to_string().as_str()
            }
            RoleEnum::Builder(status) => {
                self.to_string() + status.action_status.to_string().as_str()
            }
            RoleEnum::Porter(status) => {
                self.to_string() + status.action_status.to_string().as_str()
            }
        }
    }

    /// 设置下默认角色状态
    pub fn default(&self) -> Self {
        match self {
            RoleEnum::Harvester(_) => RoleEnum::Harvester(RoleStatus {
                creep_status: CreepStatus::SourceNotfound,
                action_status: ActionStatus::NoWork,
            }),
            RoleEnum::Upgrader(_) => RoleEnum::Upgrader(RoleStatus {
                creep_status: CreepStatus::SourceNotfound,
                action_status: ActionStatus::NoWork,
            }),
            RoleEnum::Builder(_) => RoleEnum::Builder(RoleStatus {
                creep_status: CreepStatus::SourceNotfound,
                action_status: ActionStatus::NoWork,
            }),
            RoleEnum::Porter(_) => RoleEnum::Porter(RoleStatus {
                creep_status: CreepStatus::SourceNotfound,
                action_status: ActionStatus::NoWork,
            }),
        }
    }

    /// 状态不对或者切换角色时重置状态
    pub fn reset_status(&mut self, store_status: StoreStatus) {
        match self {
            RoleEnum::Harvester(status) => {
                if !matches!(
                    status.creep_status,
                    CreepStatus::Harversting | CreepStatus::Building | CreepStatus::SourceNotfound
                ) {
                    status.creep_status = CreepStatus::Harversting
                } else {
                    match store_status {
                        StoreStatus::Full => status.creep_status = CreepStatus::Building,
                        StoreStatus::Empty => status.creep_status = CreepStatus::Harversting,
                        _ => {}
                    }
                }
            }
            RoleEnum::Upgrader(status) => {
                if !matches!(
                    status.creep_status,
                    CreepStatus::Building | CreepStatus::CarryUp | CreepStatus::SourceNotfound
                ) {
                    status.creep_status = CreepStatus::CarryUp
                } else {
                    match store_status {
                        StoreStatus::Full => status.creep_status = CreepStatus::Building,
                        StoreStatus::Empty => status.creep_status = CreepStatus::CarryUp,
                        _ => {}
                    }
                }
            }
            RoleEnum::Builder(status) => {
                if !matches!(
                    status.creep_status,
                    CreepStatus::Building | CreepStatus::CarryUp | CreepStatus::SourceNotfound
                ) {
                    status.creep_status = CreepStatus::CarryUp
                } else {
                    match store_status {
                        StoreStatus::Full => status.creep_status = CreepStatus::Building,
                        StoreStatus::Empty => status.creep_status = CreepStatus::CarryUp,
                        _ => {}
                    }
                }
            }
            RoleEnum::Porter(status) => {
                if !matches!(
                    status.creep_status,
                    CreepStatus::CarryUp | CreepStatus::CarryDown | CreepStatus::SourceNotfound
                ) {
                    status.creep_status = CreepStatus::CarryUp
                } else {
                    match store_status {
                        StoreStatus::Full => status.creep_status = CreepStatus::CarryDown,
                        StoreStatus::Empty => status.creep_status = CreepStatus::CarryUp,
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn check(&self, action: ActionStatus) -> bool {
        match self {
            RoleEnum::Harvester(status) => match action {
                ActionStatus::Harversting => {
                    matches!(status.creep_status, CreepStatus::Harversting)
                }
                ActionStatus::Upgrade | ActionStatus::Building | ActionStatus::CarryDown => {
                    matches!(status.creep_status, CreepStatus::Building)
                }
                _ => false,
            },
            RoleEnum::Upgrader(status) => match action {
                ActionStatus::CarryUp => {
                    matches!(status.creep_status, CreepStatus::CarryUp)
                }
                ActionStatus::Building | ActionStatus::Upgrade => {
                    matches!(status.creep_status, CreepStatus::Building)
                }
                _ => false,
            },
            RoleEnum::Builder(status) => match action {
                ActionStatus::CarryUp => {
                    matches!(status.creep_status, CreepStatus::CarryUp)
                }
                ActionStatus::Building | ActionStatus::Upgrade => {
                    matches!(status.creep_status, CreepStatus::Building)
                }
                _ => false,
            },
            RoleEnum::Porter(status) => match action {
                ActionStatus::CarryUp | ActionStatus::PickUp => {
                    matches!(status.creep_status, CreepStatus::CarryUp)
                }
                ActionStatus::CarryDown | ActionStatus::Building | ActionStatus::Upgrade => {
                    matches!(status.creep_status, CreepStatus::CarryDown)
                }
                _ => false,
            },
        }
    }

    // pub fn change_creep_status(&mut self, creep_status: CreepStatus) {
    //     match self {
    //         RoleEnum::Harvester(status) => status.creep_status = creep_status,
    //         RoleEnum::Upgrader(status) => status.creep_status = creep_status,
    //         RoleEnum::Builder(status) => status.creep_status = creep_status,
    //         RoleEnum::Porter(status) => status.creep_status = creep_status,
    //     }
    // }

    pub fn change_action(&mut self, action: ActionStatus) {
        match self {
            RoleEnum::Harvester(status) => status.action_status = action,
            RoleEnum::Upgrader(status) => status.action_status = action,
            RoleEnum::Builder(status) => status.action_status = action,
            RoleEnum::Porter(status) => status.action_status = action,
        }
    }
}

// 角色行为
#[derive(Debug, Clone)]
pub enum RoleAction {
    Harvester(CreepProp),
    Upgrader(CreepProp),
    Builder(CreepProp),
    Porter(CreepProp),
}

impl RoleAction {
    pub fn new(prop: CreepProp) -> Self {
        match prop.ctx.role {
            RoleEnum::Harvester(_) => RoleAction::Harvester(prop),
            RoleEnum::Upgrader(_) => RoleAction::Upgrader(prop),
            RoleEnum::Builder(_) => RoleAction::Builder(prop),
            RoleEnum::Porter(_) => RoleAction::Porter(prop),
        }
    }

    pub fn run(&self) {
        match self {
            RoleAction::Harvester(prop) => {
                let mut role = harvester::Harvester::new(prop.clone());
                match role.run() {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("{:?}", e);
                    }
                };
            }
            RoleAction::Upgrader(prop) => {
                let mut role = upgrader::Upgrader::new(prop.clone());
                match role.run() {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("{:?}", e);
                    }
                };
            }
            RoleAction::Builder(prop) => {
                let mut role = builder::Builder::new(prop.clone());
                match role.run() {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("{:?}", e);
                    }
                };
            }
            RoleAction::Porter(prop) => {
                let mut role = porter::Porter::new(prop.clone());
                match role.run() {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("{:?}", e);
                    }
                };
            }
        };
    }
}

