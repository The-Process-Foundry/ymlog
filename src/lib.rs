//! ymlog indented log file writer
//!

mod logger;
mod macros;
mod message;

pub use logger::{Level, YmLog};
pub use message::{Block, Chomp, Style};

pub mod prelude {
  pub use crate::{ymlog, ymlogger};

  pub use super::{Block, Chomp, Style};
  pub use super::{Level, YmLog};
}
