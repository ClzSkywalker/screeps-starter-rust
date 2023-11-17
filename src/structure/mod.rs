use screeps::StructureTower;
use serde::{Deserialize, Serialize};

use self::action::IStructureAction;

pub mod action;
pub mod tower;

pub trait IStructureActionRun: IStructureAction {
    // 工作线
    fn work_line(&mut self) -> anyhow::Result<()>;
    // 执行
    fn run(&mut self) -> anyhow::Result<()> {
        if !self.check() {
            return Ok(());
        }

        self.set_status();

        if let Err(e) = self.work_line() {
            log::warn!("{:?}", e);
            return Err(e);
        }

        Ok(())
    }
}

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

// #[derive(Debug, Clone, Default, Serialize, Deserialize)]
// pub struct StructStatus {
//     pub object_id: String,
//     pub struct_status: StructWorkStatus,
//     pub struct_action: StructActionStatus,
// }

#[derive(Debug, Clone, strum::Display)]
pub enum StructureAction {
    #[strum(serialize = "🗼")]
    Tower(StructureTower),
}

impl StructureAction {
    pub fn run(&self) {
        match self {
            Self::Tower(s) => {
                let mut site = tower::Tower::new(s.clone());
                match site.run() {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("{:?}", e);
                    }
                };
            }
        }
    }
}

impl From<StructureTower> for StructureAction {
    fn from(value: StructureTower) -> Self {
        Self::Tower(value)
    }
}

