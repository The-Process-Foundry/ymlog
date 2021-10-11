//! Test the various macros
//!
//!
#![feature(trace_macros)]

use std::sync::Arc;
use std::sync::Mutex;

use ymlog::prelude::*;

// Make a buffer for test result inspection
mod common {
  use std::io::{IoSlice, Result, Write};
  use std::sync::Arc;
  use std::sync::Mutex;

  /// A basic write buffer that we can keep a reference to to examine the contents later
  #[derive(Clone)]
  pub struct TestWriter(Arc<Mutex<Vec<u8>>>);

  impl TestWriter {
    pub fn new(buffer: &Arc<Mutex<Vec<u8>>>) -> TestWriter {
      TestWriter(Arc::clone(buffer))
    }
  }

  unsafe impl Send for TestWriter {}
  unsafe impl Sync for TestWriter {}

  impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
      self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> Result<()> {
      self.0.lock().unwrap().flush()
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<usize> {
      self.0.lock().unwrap().write_vectored(bufs)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
      self.0.lock().unwrap().write_all(buf)
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> Result<()> {
      self.0.lock().unwrap().write_fmt(fmt)
    }
  }
}

lazy_static::lazy_static! {
  // TODO: It looks as if slog_scopes does this by magic. Look into it
  pub(crate) static ref LOG: Mutex<YmLog<common::TestWriter>> = Mutex::new(YmLog::new());
}

#[test]
/// Just making sure the basics work. All the functional edge cases will be tested elsewhere
fn sanity_check() {
  let buffer = Arc::new(Mutex::new(Vec::<u8>::new()));
  let writer = common::TestWriter::new(&Arc::clone(&buffer));
  crate::LOG.lock().unwrap().set_output(writer);

  fn is_eq(expected: &str, buffer: &Arc<Mutex<Vec<u8>>>) {
    assert_eq!(
      expected,
      std::str::from_utf8(&buffer.lock().unwrap()).unwrap()
    );
  }

  // Empty buffer
  let mut expected = String::new();
  is_eq(&expected, &buffer);

  ymlog!("Root Message");
  expected.push_str("---\nRoot Message");
  is_eq(&expected, &buffer);

  // trace_macros!(true);
  ymlog!("_" => "Another Root Message (not a sequence)");
  // trace_macros!(false);
  expected.push_str("\n---\nAnother Root Message (not a sequence)");
  is_eq(&expected, &buffer);

  ymlog!("+_" => "Add an indented record");
  expected.push_str(":\n  - Add an indented record");
  is_eq(&expected, &buffer);

  ymlog!("_" => "Second at depth 1");
  expected.push_str("\n  - Second at depth 1");
  is_eq(&expected, &buffer);

  ymlog!("_+" => "Third and adds an indent at the end");
  expected.push_str("\n  - Third and adds an indent at the end");
  is_eq(&expected, &buffer);

  ymlog!("_" => "First at depth 2");
  expected.push_str(":\n    - First at depth 2");
  is_eq(&expected, &buffer);

  ymlog!("+_+" => "First at depth 3 with a trailing indent");
  expected.push_str(":\n      - First at depth 3 with a trailing indent");
  is_eq(&expected, &buffer);

  ymlog!("_" => "First at depth 4");
  expected.push_str(":\n        - First at depth 4");
  is_eq(&expected, &buffer);

  ymlog!("--_-" => "Dedent twice to depth 2");
  expected.push_str("\n    - Dedent twice to depth 2");
  is_eq(&expected, &buffer);

  ymlog!("_" => "And the trailing dedent brings us back to depth 1");
  expected.push_str("\n  - And the trailing dedent brings us back to depth 1");
  is_eq(&expected, &buffer);

  ymlog!("+-_" => "Neutral indent commands");
  expected.push_str("\n  - Neutral indent commands");
  is_eq(&expected, &buffer);

  ymlog!("+_" => "Adding a Block Indent\nWith extra text");
  expected.push_str(":\n    - |-\n      Adding a Block Indent\n      With extra text");
  is_eq(&expected, &buffer);

  ymlog!("+_" => "Adding another Block Indent\nAfter a block");
  expected.push_str(
    "\n    - \"\" :\n      - |-\n        Adding another Block Indent\n        After a block",
  );
  is_eq(&expected, &buffer);

  ymlog!("_" => "Add a simple item");
  expected.push_str("\n      - Add a simple item");
  is_eq(&expected, &buffer);

  // println!(
  //   "\n\nThe final buffer: '''{}'''\n",
  //   std::str::from_utf8(&buffer.lock().unwrap()).unwrap()
  // );
}
