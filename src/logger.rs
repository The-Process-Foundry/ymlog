//! An instance of a Logger

use std::fs::OpenOptions;

use serde_yaml::{Error as YmlError, Value as YmlValue};
use slog::{o, Drain};

use crate::prelude::*;

/// A flag to tell what has been written at the current indent level
enum LastBlockType {
  // Nothing has yet been written
  None,
  // Made the last message a key with no value, but with a trailing ":\n"
  Parent,
  // Wrote a complete message item at the current level
  Sibling,
}

impl Default for LastBlockType {
  fn default() -> LastBlockType {
    LastBlockType::None
  }
}

/// This handles tracking items that need to be remembered in order to create valid YAML
///
/// The tracker follows a generator pattern, where it uses the depth to figure out the indentation
/// of new records, and the proper way to concatenate each item to the previous one.
#[derive(Default)]
struct Tracker {
  /// A list the last item
  depth: Vec<LastBlockType>,
}

impl Tracker {
  pub fn new() -> Self {
    Default::default()
  }

  /// Use the current tracker state and convert the block into a string
  /// FIXME: This should return an error
  pub fn serialize(&self, block: &mut Block) -> String {
    // -> Result<String, YmlError> {
    unimplemented!("'Tracker::serialize' still needs to be implemented")
  }

  /// Add a new indentation from the last block written and return the prefix needed
  ///
  /// To indent a message, the last item needs to be turned into a key using a ":". Each parent node
  /// only indents once, so additional attempts to indent are ignored.
  pub fn indent(&mut self) -> Option<String> {
    match &self.depth.last() {
      // These both indicate that the indent doesn'tchange because there is already a valid parent
      None | Some(LastBlockType::None) => Some("".to_string()),
      // Adding an indent after a plain message requires turing it into a key
      // TODO: TEST_CASE is to see how long messages operate as keys
      Some(LastBlockType::Sibling) => {
        self.depth.push(LastBlockType::None);
        Some(":\n".to_string())
      }
      //
      Some(LastBlockType::Parent) => Some("".to_string()),
    }
  }

  /// Remove a level of indentation.
  pub fn dedent(&mut self) {
    let _ = self.depth.pop();
  }

  /// Make a new root document
  ///
  /// TODO: Test what happens when a trailing ':' is left
  pub fn reset(&mut self) {
    self.depth.clear();
  }
}

/// Contains the state tracker and a pointer to the output write stream
#[derive(Default)]
pub struct YmLog {
  tracker: Tracker,
  logger: Option<slog::Logger>,
}

impl YmLog {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn set_output<W>(&mut self, writable: W)
  where
    W: std::io::Write + Send + Sync + 'static,
  {
    // let file = OpenOptions::new()
    //     .create(true)
    //     .write(true)
    //     .truncate(true)
    //     .open(log_path)
    //     .unwrap();

    let decorator = slog_term::PlainDecorator::new(writable);
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
  }

  /// Borrow the logger and write the string to it
  fn write(&mut self, level: Level, value: String) {
    //-> Result<(), std::io::Error> {
    match &self.logger {
      None => panic!("The logger wasn't initialized"),
      Some(logger) => match level {
        Level::Critical => slog::crit!(logger, "{}", value),
        Level::Error => slog::error!(logger, "{}", value),
        Level::Warning => slog::warn!(logger, "{}", value),
        Level::Info => slog::info!(logger, "{}", value),
        Level::Debug => slog::debug!(logger, "{}", value),
        Level::Trace => slog::trace!(logger, "{}", value),
      },
    }
  }

  /// Convert and write the block to the log
  pub fn log(&mut self, block: &mut Block, actions: Option<&str>) {
    // println!("Building a block: {:#?}", block.message);

    let mut indent_prefix = None;
    let mut has_printed = false;
    let acts = actions.unwrap_or("");

    println!("Processing actions: {:#?}", actions);
    for c in acts.chars() {
      match c {
        // Indentation options
        '+' => {
          if let Some(val) = self.tracker.indent() {
            indent_prefix = Some(val)
          }
        }
        '-' => self.tracker.dedent(),
        'r' => self.tracker.reset(),

        // Write the block
        '_' => {
          self.write(
            block.log_level.unwrap_or(Level::Info),
            self.tracker.serialize(block),
          );
          has_printed = true;
        }

        // Change the log level of the message
        'T' => block.set_log_level(Level::Trace),
        'D' => block.set_log_level(Level::Debug),
        'I' => block.set_log_level(Level::Info),
        'W' => block.set_log_level(Level::Warning),
        'E' => block.set_log_level(Level::Error),
        'C' => block.set_log_level(Level::Critical),

        _ => panic!("invalid character {} found in logging statement", c),
      }
    }

    // Convert the
    unimplemented!("'YmLog::write' still needs to be implemented")
  }
}
