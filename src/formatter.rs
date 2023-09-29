//! Handle custom styling of YAML
//!
//! Because I don't have time to write a new serializer, I'm going to hack in some functionality
//! missing from rust-yaml. I'm likely going to reuse this when I try to write my own YAML parser.

use serde_yaml::{Result as YmlResult, Value as YmlValue};

/// Options used in converting a YAML Value into a string
///
/// This inserts itself as a middle-man to serde_yaml so we can customize the formatting
///
/// TODO: Merge this with the tracker for the new serializer
#[derive(Debug, Default)]
pub struct YamlFormatter {
  /// Similar to a buffer, this can be used when streaming to tell how to prefix the current line
  ///
  /// The first item is the indent used for the last item. If it hasn't changed, we can continue a
  /// line. The second tells the syntax and typed used for the last write. Blocks are handled
  /// differently from flow
  last_write: (u8, LastWriteItem),

  /// The character to use as an indent
  indent: Indent,

  /// How multiline scalars should be printed
  multiline_style: Style,

  /// Disables some options that don't work when using this in a streaming write
  ///
  /// to_flow doesn't make sense since the next write may contain another value of the same indent
  /// same with adding a newline
  _is_stream: bool,

  /// Add the document end ('\n...') after printing the value if the indent was 0/None
  finalize_document: bool,

  /// Add a final newline to the string if it doesn't already have one.
  ///
  /// This is an option that we want turned off if we're streaming, as we may want to continue the
  /// preceding value
  _trailing_newline: bool,

  /// The width of the readable page (default is 120 characters)
  ///
  /// This is used to determine how to fold strings
  /// TODO: Make this a more robust filter, such as using min or max items
  wrap_at: Option<usize>,
}

impl YamlFormatter {
  /// Set the indent
  pub fn set_indent(&mut self, indent: Indent) {
    self.indent = indent;
  }

  /// Set the style
  pub fn set_style(&mut self, style: Style) {
    self.multiline_style = style;
  }

  /// Convert a yaml value into a string
  ///
  /// This is being designed for streaming.
  pub fn stringify(&mut self, value: YmlValue, indent: Option<u8>) -> YmlResult<String> {
    // An empty container for the string
    let mut result = String::new();
    let depth = indent.unwrap_or(0);

    // Get the initial indent and add one to the result
    let indent_str = self.indent.make(indent);
    result.push_str(&indent_str);

    match value {
      YmlValue::Mapping(_mapping) => {
        unimplemented!("'stringify mapping' still needs to be implemented")
      }
      YmlValue::Sequence(_seq) => {
        unimplemented!("'stringify sequence' still needs to be implemented")
      }
      YmlValue::Null => {
        self.last_write = (indent.unwrap_or(0), LastWriteItem::Flow(ItemType::Scalar));
        Ok(String::new())
      }
      YmlValue::Number(value) => {
        self.last_write = (indent.unwrap_or(0), LastWriteItem::Flow(ItemType::Scalar));
        serde_yaml::to_string(&value)
      }
      YmlValue::Bool(value) => {
        self.last_write = (indent.unwrap_or(0), LastWriteItem::Flow(ItemType::Scalar));
        serde_yaml::to_string(&value)
      }
      YmlValue::String(value) => {
        self.last_write = (indent.unwrap_or(0), LastWriteItem::None);
        self.stringify_string(value, depth as usize)
      }
    }
    .map(|value| {
      result.push_str(&value);
    })?;

    // How to finish the document if the document ended is 0
    if indent.unwrap_or(0) == 0 && self.finalize_document {
      result.push_str("...\n")
    };

    Ok(result)
  }

  /// Write a String scalar
  fn stringify_string(&mut self, value: String, depth: usize) -> YmlResult<String> {
    let mut result = String::new();

    let style = match self.multiline_style {
      Style::Guess => Style::guess_style(&value),
      _ => self.multiline_style.clone(),
    };

    let last_write = &mut self.last_write.1;
    match style {
      Style::Guess => unreachable!("The style is already guessed"),
      Style::Folded(chomp) => {
        *last_write = LastWriteItem::Block(ItemType::Scalar);
        result.push_str(&Style::fold_string(
          value,
          depth,
          &chomp,
          &self.indent,
          self.wrap_at.unwrap_or(120),
        )?);
      }
      Style::Literal(chomp) => {
        *last_write = LastWriteItem::Block(ItemType::Scalar);
        result.push_str(&Style::literal_string(
          value,
          depth,
          &chomp,
          &self.indent,
          self.wrap_at.unwrap_or(120),
        )?);
      }
      _ => unimplemented!("'The Scalars' still needs to be implemented"),
    }
    Ok(result)
  }
}

/// The type and size of indenting to use
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Indent {
  /// Use the number of spaces listed for each level. Default is 2
  Space(u8),
  Tab,
}

impl std::fmt::Display for Indent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Indent::Space(count) => " ".repeat(*count as usize),
        Indent::Tab => "\t".to_string(),
      }
    )
  }
}

impl Default for Indent {
  fn default() -> Indent {
    Indent::Space(2)
  }
}

impl Indent {
  /// Make a string containing count many indents
  pub fn make(&self, count: Option<u8>) -> String {
    self.to_string().repeat(count.unwrap_or(0).into())
  }
}

/// How YAML should handle formatting multiline strings
///
/// FIXME: The spec for YAML is rather confusing, so this will need to be totally reworked

#[derive(Debug, Clone)]
pub enum Style {
  /// This will guess the best style based on the contents of the message (Heaviest calculation)
  Guess,

  /// Block Style: Folded replaces all individual newlines with a single space '>'
  Folded(Chomp),
  /// Block Style: Leave all newlines as is: '|'
  Literal(Chomp),

  /// Flow Style:
  Plain,
  /// Flow Style: encode everything within single quotes
  Single,
  /// Flow Style: encode everything within single quotes
  Double,
}

impl Default for Style {
  fn default() -> Self {
    Style::Guess
  }
}

impl Style {
  /// When writing, this replaces a whitespace character with a wrap
  ///
  pub fn fold_string(
    value: String,
    depth: usize,
    chomp: &Chomp,
    indent: &Indent,
    wrap_at: usize,
  ) -> YmlResult<String> {
    // How much to indent a block
    let indent = indent.to_string().repeat(depth + 1);

    // Count of characters towards the wrap in the current line. Indent counts
    let mut line_len = indent.len() as usize;
    let mut word = String::new();
    let mut word_is_ws = false;

    let mut result = format!(" >{}\n", chomp);
    for c in value.chars() {
      match c {
        ' ' => {
          // If we've hit the wrap limit, convert the space into a newline
          if wrap_at <= word.len() + line_len {
            if word_is_ws {
              // If its all whitespace, save it for the next line
              result.push('\n');
              word = format!("{}{}", indent, word);
              line_len = indent.len() - 1;
            } else {
              // Otherwise the word fits so we print it to the next line
              result.push_str(&format!("{}\n{}", word, indent));
              word = indent.clone();
              line_len = indent.len() - 1;
              word_is_ws = false;
            }
          } else {
            // Found the end of a word, so we print it
            result.push_str(&word);
            line_len += word.len();
            word.clear();
          }
        }
        '\n' => {
          result.push_str(&format!("\n{}", indent));
          line_len = indent.len() - 1;
        }
        _ => {
          word.push(c);
        }
      }
    }

    Ok(result)
  }

  /// Print a block with a Literal syntax (preserves newlines)
  pub fn literal_string(
    _value: String,
    _depth: usize,
    chomp: &Chomp,
    _indent: &Indent,
    _wrap_at: usize,
  ) -> YmlResult<String> {
    let result = format!(" |{}\n", chomp);

    Ok(result)
  }

  pub fn guess_style(value: &str) -> Style {
    match value.contains('\n') {
      true => Style::Literal(Default::default()),
      false => Style::Double,
    }
  }
}

/// Whether to remove any trailing newlines
#[derive(Debug, Clone)]
pub enum Chomp {
  Clip,
  Strip,
  Keep,
}

impl Default for Chomp {
  fn default() -> Self {
    Chomp::Clip
  }
}

impl std::fmt::Display for Chomp {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Chomp::Clip => "",
        Chomp::Strip => "-",
        Chomp::Keep => "+",
      }
    )
  }
}

/// A description of the
#[derive(Debug)]
pub enum LastWriteItem {
  /// The formatter is brand new and hasn't written anything yet
  None,

  /// A CR/LF was printed.
  ///
  /// This is not unusual after some tokens, such as creating a new Sequence with no values. It
  /// contains a lookback to help indicate what the next valid value can be.
  _NewLine(Box<LastWriteItem>),

  /// Formatted blocks that can be treated as a single unit.
  Flow(ItemType),
  Block(ItemType),

  // ---  Tokens
  /// Printed the end of a document, so the next item needs to be prefixed with a new document
  _EndDocument,

  /// The queston mark operator, indicating the next item will be be a scalar key,
  _Question,

  /// The colon operator preceding a Mapping value
  _Colon,
}

impl Default for LastWriteItem {
  fn default() -> Self {
    LastWriteItem::None
  }
}

#[derive(Debug)]
pub enum ItemType {
  /// Last printed a scalar
  Scalar,

  /// Item started with a "-"
  _SequenceItem,

  /// The key of a Mapping pair
  _MappingKey,

  // The value of a Mapping pair
  _MappingValue,
}
