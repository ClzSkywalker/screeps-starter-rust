use thiserror::Error;
#[derive(Error, Debug)]
pub enum ScreepError {
    #[error("room notfound: {0}")]
    RoomNotfound(String),
    #[error("unknown data store error")]
    Unknown,
}
