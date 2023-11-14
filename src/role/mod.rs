use log::warn;
use serde::{Deserialize, Serialize};
use strum::EnumString;

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
            RoleEnum::Harvester => RoleAction::Harvester(prop),
            RoleEnum::Upgrader => RoleAction::Upgrader(prop),
            RoleEnum::Builder => RoleAction::Builder(prop),
            RoleEnum::Porter => RoleAction::Porter(prop),
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

