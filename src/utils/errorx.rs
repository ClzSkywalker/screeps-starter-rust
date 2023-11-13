use thiserror::Error;
#[derive(Error, Debug)]
pub enum ScreepError {
    #[error("room notfound: {0}")]
    RoomNotfound(String),
    #[error("role can not: {0}")]
    RoleCanNotWork(String),
    // #[error("structure notfound: {0}")]
    // StructureNotfound(String),
    #[error("screep 内部错误")]
    ScreepInner,
}
