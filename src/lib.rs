//! ymlog indented log file writer
//!

mod logger;
mod macros;
mod message;

pub use logger::{Level, YmLog};
pub use message::Block;

pub mod prelude {
  pub use crate::{ymlog, ymlogger};

  pub use super::Block;
  pub use super::{Level, YmLog};
}
