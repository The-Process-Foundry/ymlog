//! Building blocks of the log

use chrono::{DateTime, Utc};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use serde_yaml::Error as YmlError;

use crate::prelude::*;

/// A block is a message formatting container
///
/// Serialization is customized based on the blocks filled in
#[derive(Default)]
pub struct Block {
  /// The local time the message was generated
  pub(crate) timestamp: Option<DateTime<Utc>>,

  /// The level of the message
  pub(crate) log_level: Option<Level>,

  /// Searchable strings in the output log
  pub(crate) tags: Option<Vec<String>>,

  /// The content of the message
  pub(crate) message: Option<serde_yaml::Value>,

  /// Any indented child blocks
  ///
  /// This is really only used for deserializing. A user is never allowed to directly add children
  /// because keeping the indentation level straight becomes too heavy.
  pub(crate) children: Option<Vec<Block>>,
}

impl Serialize for Block {
  /// Customize the format of the output based on the given fields
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    if self.message.is_none() {
      return Err(serde::ser::Error::custom(
        "Tried to serialize a ymlog block without setting a message",
      ));
    };

    // If any of these are included, we loop. Otherwise, make a simple string of the level and message
    let count = vec![
      self.timestamp.is_some(),
      self.tags.is_some(),
      self.children.is_some(),
      self.log_level.is_some(),
    ]
    .into_iter()
    .filter(|x| x.to_owned())
    .count();
    match count == 0 {
      true => self.message.as_ref().unwrap().serialize(serializer),
      false => {
        let mut state = serializer.serialize_struct("Block", count)?;
        if self.timestamp.is_some() {
          state.serialize_field("timestamp", &self.timestamp.unwrap())?
        };
        if self.log_level.is_some() {
          state.serialize_field(
            "log_level",
            match &self.log_level {
              Some(Level::Trace) => "Trace",
              Some(Level::Debug) => "Debug",
              Some(Level::Info) => "Info",
              Some(Level::Warn) => "Warn",
              Some(Level::Error) => "Error",
              None => unreachable!("Already checked for None"),
            },
          )?
        };
        state.serialize_field("message", self.message.as_ref().unwrap())?;
        if self.children.is_some() {
          state.serialize_field("children", &self.timestamp.unwrap())?
        };
        state.end()
      }
    }
  }
}

impl Block {
  pub fn new() -> Block {
    Default::default()
  }

  /// Change the message to the Display message of the object passed in
  pub fn set_message(&mut self, message: impl Serialize) -> Result<(), YmlError> {
    println!("Setting the message");
    self.message = Some(serde_yaml::to_value(message)?);
    Ok(())
  }

  /// Set the tags of the current block
  pub fn set_tags(&mut self, tags: Vec<impl std::fmt::Display>) {
    self.tags = Some(tags.iter().map(|tag| tag.to_string()).collect());
  }

  /// Add child blocks that have been aggregated in code
  pub fn set_children(&mut self, children: Vec<Block>) {
    self.children = Some(children);
  }

  /// Updates the level. If left unset, it defaults to debug.
  pub fn set_log_level(&mut self, level: Level) {
    self.log_level = Some(level);
  }

  /// Set the timestamp to the current time
  pub fn stamp(&mut self) {
    self.timestamp = Some(Utc::now());
  }
}
