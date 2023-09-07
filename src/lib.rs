//! ymlog indented log file writer
//!

mod formatter;
mod logger;
mod macros;
mod message;

pub use formatter::{Chomp, Style, YamlFormatter};
pub use logger::{Level, YmLog};
pub use message::Block;

pub mod prelude {
  pub use crate::{ymlog, ymlogger};

  pub use super::{Block, Chomp, Level, Style, YamlFormatter, YmLog};
}
