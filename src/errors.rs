use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    // todo: Add errors here when there is such a need
    #[error("unknown game error")]
    Unknown,
}