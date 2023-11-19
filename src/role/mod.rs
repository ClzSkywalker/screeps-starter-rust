use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use log::warn;
use serde::{Deserialize, Serialize};

use crate::model::ctx::{ActionStatus, CreepStatus, StoreStatus};

use self::{action::ICreepAction, creep::CreepProp};

pub mod action;
pub mod builder;
pub mod creep;
pub mod harvester;
pub mod porter;
pub mod repairer;
pub mod upgrader;

pub trait IRoleAction: ICreepAction {
    // 创建实例
    fn new(creep: CreepProp) -> Self;
    // 工作线
    fn work_line(&mut self) -> anyhow::Result<()>;
    // 执行
    fn run(&mut self) -> anyhow::Result<()> {
        if !self.check() {
            return Ok(());
        }

        self.set_status();

        if let Err(e) = self.work_line() {
            warn!("{:?}", e);
            return Err(e);
        }

        self.say();
        self.cancel_bind_structure();
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
    #[strum(serialize = "💘")]
    Repairer(RoleStatus),
}

impl Deref for RoleEnum {
    type Target = RoleStatus;

    fn deref(&self) -> &Self::Target {
        match self {
            RoleEnum::Harvester(s) => s,
            RoleEnum::Upgrader(s) => s,
            RoleEnum::Builder(s) => s,
            RoleEnum::Porter(s) => s,
            RoleEnum::Repairer(s) => s,
        }
    }
}

impl DerefMut for RoleEnum {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            RoleEnum::Harvester(s) => s,
            RoleEnum::Upgrader(s) => s,
            RoleEnum::Builder(s) => s,
            RoleEnum::Porter(s) => s,
            RoleEnum::Repairer(s) => s,
        }
    }
}

const HARVESTER: &str = "harvester";
const UPGRADER: &str = "upgrader";
const BUILDER: &str = "builder";
const PORTER: &str = "porter";
const REPAIRER: &str = "repairer";

impl From<String> for RoleEnum {
    fn from(value: String) -> Self {
        match value.as_str() {
            HARVESTER => RoleEnum::Harvester(RoleStatus::default()),
            UPGRADER => RoleEnum::Upgrader(RoleStatus::default()),
            BUILDER => RoleEnum::Builder(RoleStatus::default()),
            PORTER => RoleEnum::Porter(RoleStatus::default()),
            REPAIRER => RoleEnum::Repairer(RoleStatus::default()),
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
            RoleEnum::Repairer(_) => REPAIRER.to_string(),
        }
    }

    /// say文字：角色+行为
    pub fn get_say_test(&self) -> String {
        let status = match self {
            RoleEnum::Harvester(status) => status.clone(),
            RoleEnum::Upgrader(status) => status.clone(),
            RoleEnum::Builder(status) => status.clone(),
            RoleEnum::Porter(status) => status.clone(),
            RoleEnum::Repairer(status) => status.clone(),
        };
        if status.action_status != ActionStatus::NoWork {
            return String::new();
        }
        return self.to_string() + status.action_status.to_string().as_str();
    }

    /// 是否取消绑定
    pub fn is_cancel_bind(&self) -> bool {
        match self {
            RoleEnum::Harvester(s) => !matches!(s.action_status, ActionStatus::Harversting),
            RoleEnum::Upgrader(_) => true,
            RoleEnum::Builder(_) => true,
            RoleEnum::Porter(_) => true,
            RoleEnum::Repairer(s) => !matches!(s.action_status, ActionStatus::Repair),
        }
    }

    /// 设置下默认角色状态
    pub fn default(&self) -> Self {
        let status = RoleStatus::default();
        match self {
            RoleEnum::Harvester(_) => RoleEnum::Harvester(status),
            RoleEnum::Upgrader(_) => RoleEnum::Upgrader(status),
            RoleEnum::Builder(_) => RoleEnum::Builder(status),
            RoleEnum::Porter(_) => RoleEnum::Porter(status),
            RoleEnum::Repairer(_) => RoleEnum::Repairer(status),
        }
    }

    /// 状态不对或者切换角色时重置状态
    pub fn reset_status(&mut self, store_status: StoreStatus) {
        match self {
            RoleEnum::Harvester(status) => {
                status.action_status = ActionStatus::NoWork;
                if !matches!(
                    status.creep_status,
                    CreepStatus::UseEnergy | CreepStatus::FindEnergy | CreepStatus::SourceNotfound
                ) {
                    status.creep_status = CreepStatus::FindEnergy
                } else {
                    match store_status {
                        StoreStatus::Full => status.creep_status = CreepStatus::UseEnergy,
                        StoreStatus::Empty => status.creep_status = CreepStatus::FindEnergy,
                        _ => {}
                    }
                }
            }
            RoleEnum::Upgrader(status) => {
                status.action_status = ActionStatus::NoWork;
                if !matches!(
                    status.creep_status,
                    CreepStatus::UseEnergy | CreepStatus::FindEnergy | CreepStatus::SourceNotfound
                ) {
                    status.creep_status = CreepStatus::FindEnergy
                } else {
                    match store_status {
                        StoreStatus::Full => status.creep_status = CreepStatus::UseEnergy,
                        StoreStatus::Empty => status.creep_status = CreepStatus::FindEnergy,
                        _ => {}
                    }
                }
            }
            RoleEnum::Builder(status) => {
                status.action_status = ActionStatus::NoWork;
                if !matches!(
                    status.creep_status,
                    CreepStatus::UseEnergy | CreepStatus::FindEnergy | CreepStatus::SourceNotfound
                ) {
                    status.creep_status = CreepStatus::FindEnergy
                } else {
                    match store_status {
                        StoreStatus::Full => status.creep_status = CreepStatus::UseEnergy,
                        StoreStatus::Empty => status.creep_status = CreepStatus::FindEnergy,
                        _ => {}
                    }
                }
            }
            RoleEnum::Porter(status) => {
                status.action_status = ActionStatus::NoWork;
                if !matches!(
                    status.creep_status,
                    CreepStatus::UseEnergy | CreepStatus::FindEnergy | CreepStatus::SourceNotfound
                ) {
                    status.creep_status = CreepStatus::FindEnergy
                } else {
                    match store_status {
                        StoreStatus::Full => status.creep_status = CreepStatus::UseEnergy,
                        StoreStatus::Empty => status.creep_status = CreepStatus::FindEnergy,
                        _ => {}
                    }
                }
            }
            RoleEnum::Repairer(status) => {
                status.action_status = ActionStatus::NoWork;
                if !matches!(
                    status.creep_status,
                    CreepStatus::UseEnergy | CreepStatus::FindEnergy | CreepStatus::SourceNotfound
                ) {
                    status.creep_status = CreepStatus::FindEnergy
                } else {
                    match store_status {
                        StoreStatus::Full => status.creep_status = CreepStatus::UseEnergy,
                        StoreStatus::Empty => status.creep_status = CreepStatus::FindEnergy,
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn check(&self, action: ActionStatus) -> bool {
        match self {
            RoleEnum::Harvester(status) => match action {
                ActionStatus::Harversting | ActionStatus::CarryUp => {
                    matches!(
                        status.creep_status,
                        CreepStatus::FindEnergy | CreepStatus::SourceNotfound
                    )
                }
                ActionStatus::Upgrade | ActionStatus::Building | ActionStatus::CarryDown => {
                    matches!(status.creep_status, CreepStatus::UseEnergy)
                }
                _ => false,
            },
            RoleEnum::Upgrader(status) => match action {
                ActionStatus::CarryUp => {
                    matches!(
                        status.creep_status,
                        CreepStatus::FindEnergy | CreepStatus::SourceNotfound
                    )
                }
                ActionStatus::Building | ActionStatus::Upgrade => {
                    matches!(status.creep_status, CreepStatus::UseEnergy)
                }
                _ => false,
            },
            RoleEnum::Builder(status) => match action {
                ActionStatus::CarryUp => {
                    matches!(
                        status.creep_status,
                        CreepStatus::FindEnergy | CreepStatus::SourceNotfound
                    )
                }
                ActionStatus::Building | ActionStatus::Upgrade | ActionStatus::CarryDown => {
                    matches!(status.creep_status, CreepStatus::UseEnergy)
                }
                _ => false,
            },
            RoleEnum::Porter(status) => match action {
                ActionStatus::CarryUp | ActionStatus::PickUp | ActionStatus::Harversting => {
                    matches!(
                        status.creep_status,
                        CreepStatus::FindEnergy | CreepStatus::SourceNotfound
                    )
                }
                ActionStatus::CarryDown | ActionStatus::Building | ActionStatus::Upgrade => {
                    matches!(status.creep_status, CreepStatus::UseEnergy)
                }
                _ => false,
            },
            RoleEnum::Repairer(status) => match action {
                ActionStatus::CarryUp => {
                    matches!(
                        status.creep_status,
                        CreepStatus::FindEnergy | CreepStatus::SourceNotfound
                    )
                }
                ActionStatus::Repair
                | ActionStatus::CarryDown
                | ActionStatus::Building
                | ActionStatus::Upgrade => {
                    matches!(status.creep_status, CreepStatus::UseEnergy)
                }
                _ => false,
            },
        }
    }

    pub fn change_action(&mut self, action: ActionStatus) {
        match self {
            RoleEnum::Harvester(status) => status.action_status = action,
            RoleEnum::Upgrader(status) => status.action_status = action,
            RoleEnum::Builder(status) => status.action_status = action,
            RoleEnum::Porter(status) => status.action_status = action,
            RoleEnum::Repairer(status) => status.action_status = action,
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
    Repairer(CreepProp),
}

impl RoleAction {
    pub fn new(prop: CreepProp) -> Self {
        match prop.ctx.role {
            RoleEnum::Harvester(_) => RoleAction::Harvester(prop),
            RoleEnum::Upgrader(_) => RoleAction::Upgrader(prop),
            RoleEnum::Builder(_) => RoleAction::Builder(prop),
            RoleEnum::Porter(_) => RoleAction::Porter(prop),
            RoleEnum::Repairer(_) => RoleAction::Repairer(prop),
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
            RoleAction::Repairer(prop) => {
                let mut role = repairer::Repairer::new(prop.clone());
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

