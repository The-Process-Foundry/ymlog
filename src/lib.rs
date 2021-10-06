//! ymlog indented log file writer
//!

mod logger;
mod macros;
mod message;

pub use logger::YmLog;
pub use message::Block;

pub mod prelude {
  // Re-export the level from slog
  pub use slog::Level;

  pub use crate::{ymlog, ymlogger};

  pub use super::Block;
  pub use super::YmLog;
}
