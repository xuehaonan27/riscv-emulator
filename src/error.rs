#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error when performing I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error when loading ELF: {0}")]
    LoadElf(#[from] goblin::error::Error),
    #[error("Invalid ELF format: {0}")]
    InvalidElf(String),
    #[error("Error when parsing input to REDB: {0}")]
    DbgParse(String),
    #[error("Unknown register name: {0}")]
    InvalidRegName(String),
    #[error("Error when fetch instruction: {0}")]
    Fetch(String),
    #[error("Error when decoding: {0}")]
    Decode(String),
    #[error("Error when executing: {0}")]
    Execute(String),
    #[error("{0}")]
    Exception(#[from] Exception),
}

/// CPU raised exceptions
#[derive(Debug, thiserror::Error)]
pub enum Exception {
    #[error("DividedByZero")]
    DividedByZero,
    #[error("IllegalInstruction")]
    IllegalInstruction,
}

pub type Result<T> = std::result::Result<T, Error>;
