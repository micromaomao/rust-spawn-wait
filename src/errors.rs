use std::{fmt::Debug, io};

use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
  #[error("Failed to spawn process: {0}")]
  UnableToSpawnProcess(#[source] io::Error),
  #[error("Wait failed")]
  WaitFailed(#[source] io::Error),
}
