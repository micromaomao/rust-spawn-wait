//! This crate allows you to spawn and manage a set of processes each associated
//! with a key, and wait on all or part of them simultaneously.
//!
//! This crate also:
//!
//! * Handles catching ctrl+C
//! * Allows you to signal all spawned processes to terminate, for example in case
//!   any one of them fails.

mod processset;
pub use processset::{ProcessSet, WaitAnyResult};
mod errors;
pub use errors::Error;
mod signal;
pub use signal::SignalHandler;
