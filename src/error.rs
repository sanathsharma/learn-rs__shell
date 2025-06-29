use std::{io, result};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("IO Error: {0}")]
  IoError(#[from] io::Error),

  #[error("UTF8 Error: {0}")]
  Utf8Error(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = result::Result<T, Error>;
