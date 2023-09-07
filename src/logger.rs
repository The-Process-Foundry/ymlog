//! An instance of a Logger

// use std::fs::OpenOptions;
use std::cell::RefCell;

use serde_yaml::{Mapping, Value as YmlValue};

use crate::message::MessageType;
use crate::prelude::*;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Level {
  Trace,
  Debug,
  Info,
  Warn,
  Error,
}

/// A flag to tell what has been written at the current indent level
#[derive(Debug)]
enum LastBlockType {
  // Nothing has yet been written
  None,

  // A reset has been sent, so we need to prefix a '\n'
  Reset,

  // Wrote key message
  Message,

  // Wrote a message as a block (requires a "children:")
  BlockMessage,

  // An indent record was requested for the next block
  Indent,

  // A block was printed, so we need to use a different character to use it as a
  BlockIndent,

  /// Printed a key/value pair and should dedent before automatically should dedent when found
  KeyValue,
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
  /// Recursively use the block to build a YAML object
  ///
  /// This handles adding the children to the message (if appropriate) and updating the depth
  // FIXME: Children aren't handled properly with a scan. Need to think about how to define them
  // TODO: Test how nested children affect the depth
  fn build_value(block: &Block) -> (YmlValue, Vec<LastBlockType>) {
    // One or the other, both makes no sense
    match (&block.message, &block.children) {
      // Always fail if there is no message
      (MessageType::None, _) => {
        panic!("Logs must always have a base message set")
      }

      (MessageType::Value(YmlValue::Mapping(_)), Some(_)) => {
        panic!("Log message blocks either have children or a map, not both")
      }

      (MessageType::Value(value), Some(children)) => {
        // We will continue at the depth of the last child
        let mut last_depth = vec![];
        let seq = children.iter().fold(vec![], |mut acc, child| {
          let (kid, depth) = Tracker::build_value(child);
          last_depth = depth;
          acc.push(kid);
          acc
        });

        let mut mapping = Mapping::new();
        mapping.insert(value.clone(), YmlValue::Sequence(seq));
        (YmlValue::Mapping(mapping), last_depth)
      }

      (MessageType::Value(value), None) => (value.clone(), vec![LastBlockType::Message]),

      (MessageType::KeyValue(_, _), Some(_)) => {
        panic!("Key/Value log messages cannot have children")
      }

      (MessageType::KeyValue(key, value), None) => {
        let mut mapping = Mapping::new();
        mapping.insert(key.to_owned(), value.to_owned());

        (YmlValue::Mapping(mapping), vec![LastBlockType::KeyValue])
      }
    }
  }

  /// If it is a plain string, If it finds any \n in the message, it turns it into a block
  /// HACK: This is a lack in rust-yaml, which trickled into serde_yaml. Blocks are not detected,
  ///       and cannot be set manually. So I'm just going to handle the simple message.
  fn is_block(value: &YmlValue) -> bool {
    if let YmlValue::String(inner) = value {
      return inner.contains('\n');
    }
    false
  }

  /// Add the proper indentation around the block".to_string()
  ///
  /// It also adds a "__Cut Here__" so when stringified, we can remove the plain indents that do not
  /// need to be added
  fn indent_string(&mut self, value: YmlValue) -> String {
    // println!("\n\nIndenting value: {:?}", value);
    // println!("At depth: {:?}", self.depth);

    let is_block = Tracker::is_block(&value);
    if is_block {
      if let Some(last) = self.depth.last_mut() {
        *last = LastBlockType::BlockMessage;
      }
    }

    match self.depth.len() {
      0 => unreachable!("Should never be able to get here with a zero depth"),

      // Print a root level message (new document)
      1 => match is_block {
        true => {
          if let YmlValue::String(inner) = value {
            return format!("|+ {}", inner);
          };
          unreachable!("It's a block, so it's always a string")
        }
        false => serde_yaml::to_string(&value).unwrap(),
      },

      // Pad out the value so the message and children have the proper indentation
      _ => {
        let mut tmp = serde_yaml::Mapping::new();
        tmp.insert(
          YmlValue::String("__Cut Here__".into()),
          YmlValue::Sequence(vec![value]),
        );
        let mut padded = YmlValue::Mapping(tmp);

        // Pad out the value with indentations
        for _ in 1..(self.depth.len() - 1) {
          let mut tmp = serde_yaml::Mapping::new();
          tmp.insert(YmlValue::String("".into()), padded);
          padded = YmlValue::Mapping(tmp);
        }

        // println!("The padded message is: {:?}", padded);

        // Find the placeholder and get rid of it
        match serde_yaml::to_string(&padded)
          .unwrap()
          .split_once("__Cut Here__:\n")
        {
          None => panic!(
            "\n\n--> Could not find '__Cut Here__:' in the serialized message block:\n{:#?}",
            serde_yaml::to_string(&padded).unwrap()
          ),
          Some((_, message)) => match is_block {
            true => {
              let i_size = self.depth.len() * 2;

              // Split after the initial indent
              let (indent, end) = message.split_at(i_size);

              // Add an indent after each carriage return
              let block_indent = " ".repeat(i_size);

              let sliced = &end[1..end.len() - 2].replace("\\n", &format!("\n{}", block_indent));
              format!("{}|-\n{}{}\n", indent, block_indent, sliced)
            }
            false => message.to_string(),
          },
        }
      }
    }
  }

  /// Convert it to a writable string, updating the Tracker state
  pub fn serialize(&mut self, block: &mut Block) -> String {
    // Convert the block into a pure YmlValue and its depth
    let (value, _new_depth) = Tracker::build_value(block);

    // Convert the value to a string with proper indentation
    let indented = match self.depth.last() {
      // First message in the document is done plain
      None => {
        self.depth.push(LastBlockType::Message);
        format!("\n{}", serde_yaml::to_string(&value).unwrap())
      }

      // Same as None, but has written the document tag. It appends a newline, so the next document
      // tag doesn't get mashed up on the previous line
      Some(LastBlockType::None) => {
        if let Some(last) = self.depth.last_mut() {
          *last = LastBlockType::Message;
        }
        format!("\n{}", serde_yaml::to_string(&value).unwrap())
      }

      // After an explicit reset, we need to add a newline
      Some(LastBlockType::Reset) => {
        if let Some(last) = self.depth.last_mut() {
          *last = LastBlockType::Message;
        }
        format!("\n{}", serde_yaml::to_string(&value).unwrap())
      }

      // The last item was in a sequence (this is the plain record)
      Some(LastBlockType::Message) => {
        format!("\n{}", self.indent_string(value))
      }

      // The last item was a block. This only affects indents after
      Some(LastBlockType::BlockMessage) => {
        format!("\n{}", self.indent_string(value))
      }

      // An indent was requested for this item
      Some(LastBlockType::Indent) => {
        // Tell the tracker we've taken care of the indent
        if let Some(last) = self.depth.last_mut() {
          *last = LastBlockType::Message;
        }
        format!(":\n{}", self.indent_string(value))
      }

      // An indent was requested for this item
      // HACK: To make this work as a stream, we have to add a phony key
      Some(LastBlockType::BlockIndent) => {
        // Tell the tracker we've taken care of the indent
        if let Some(last) = self.depth.last_mut() {
          *last = LastBlockType::BlockMessage;
        }

        // This adds another item to the sequence and the phony key
        format!(
          "\n{}- \"\" :\n{}",
          "  ".repeat(self.depth.len() - 2),
          self.indent_string(value)
        )
      }

      _ => unimplemented!("'KeyValue' still needs to be implemented"),
    }
    .trim_end()
    .to_string();

    // Update the depth, if needed

    // And return the value
    indented
  }

  /// Add a new indentation from the last block written and return the prefix needed
  ///
  /// To indent a message, the last item needs to be turned into a key using a ":". Each parent node
  /// only indents once, so additional attempts to indent are ignored.
  pub fn indent(&mut self) {
    match &self.depth.last() {
      Some(LastBlockType::Message) => self.depth.push(LastBlockType::Indent),
      Some(LastBlockType::BlockMessage) => self.depth.push(LastBlockType::BlockIndent),
      _ => (),
    };
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
    self.depth.push(LastBlockType::Reset);
  }
}

/// Contains the state tracker and a pointer to the output write stream
pub struct YmLog<T>
where
  T: std::io::Write + Send + Sync + 'static,
{
  // The state caused by data being written to the logger
  tracker: Tracker,
  // Minimum level to be written to the logger
  log_level: Level,
  // The output buffer of the log
  logger: Option<RefCell<T>>,
}

impl<T> Default for YmLog<T>
where
  T: std::io::Write + Send + Sync + 'static,
{
  fn default() -> YmLog<T> {
    YmLog {
      tracker: Default::default(),
      log_level: Level::Warn,
      logger: None,
    }
  }
}

impl<T> YmLog<T>
where
  T: std::io::Write + Send + Sync + 'static,
{
  pub fn new() -> Self {
    Default::default()
  }

  pub fn set_output(&mut self, writable: T) {
    // let file = OpenOptions::new()
    //     .create(true)
    //     .write(true)
    //     .truncate(true)
    //     .open(log_path)
    //     .unwrap();

    self.logger = Some(RefCell::new(writable));
  }

  /// Change the level threshhold for writing a message to the log
  pub fn set_level(&mut self, level: Level) {
    self.log_level = level;
  }

  /// Borrow the logger and write the string to it
  fn write(&mut self, block: &mut Block) {
    //-> Result<(), std::io::Error> {

    let level = block.log_level.as_ref().unwrap_or(&Level::Info);
    if self.log_level > *level {
      return;
    };

    if let Some(logger) = &self.logger {
      let value = self.tracker.serialize(block);
      let _ = logger.borrow_mut().write_all(value.as_bytes());
    }
  }

  fn split_block(&mut self, block: &mut Block) {
    // Fail if message doesn't have a colon
    let msg = match &block.message {
      MessageType::Value(YmlValue::String(msg)) => msg,
      MessageType::Value(_) => panic!("Only string messages can be split"),
      MessageType::KeyValue(key, _) => {
        panic!("Tried to re-split a logging block with key {:?}", key)
      }
      MessageType::None => panic!("Cannot split message that wasn't set"),
    };

    let (key, value) = match msg.split_once(':') {
      Some(x) => x,
      None => panic!("Could not find a ':' to split at\nmsg => {:?}", msg),
    };

    block.message = MessageType::KeyValue(
      YmlValue::String(key.to_string()),
      YmlValue::String(value.to_string()),
    );
  }

  /// Convert and write the block to the log
  pub fn log(&mut self, block: &mut Block, actions: Option<&str>) {
    // println!("Building a block: {:#?}", block.message);
    // Skip working on

    // Make sure we know the logger is correct
    assert!(self.logger.is_some(), "The logger wasn't initialized");

    let mut has_printed = false;
    let acts = actions.unwrap_or("");

    // println!("Processing actions: {:#?}", actions);
    for c in acts.chars() {
      match c {
        // Indentation options
        '+' => self.tracker.indent(),
        '-' => self.tracker.dedent(),
        'r' => self.tracker.reset(),

        // TODO: Add this feature
        // Split the message at the first colon, making the left a key and the right a block
        'k' => self.split_block(block),

        // Formatting options for the message
        // 'b' => block.set_style(Style::Literal(Chomp::Clip)),

        // Write the block
        '_' => {
          self.write(block);
          has_printed = true;
        }

        // Change the log level of the message
        'T' => block.set_log_level(Level::Trace),
        'D' => block.set_log_level(Level::Debug),
        'I' => block.set_log_level(Level::Info),
        'W' => block.set_log_level(Level::Warn),
        'E' => block.set_log_level(Level::Error),

        _ => panic!("invalid character {} found in logging statement", c),
      }
    }

    if !has_printed {
      self.write(block);
    }
  }
}
