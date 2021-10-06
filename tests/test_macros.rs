//! Test the various macros
//!
//!
#![feature(trace_macros)]

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

use ymlog::prelude::*;

// Make a buffer for test result inspection
mod common {
  use std::cell::RefCell;
  use std::io::{IoSlice, Result, Write};
  use std::sync::Arc;

  /// A basic write buffer that we can keep a reference to to examine the contents later
  #[derive(Clone)]
  pub struct TestWriter(Arc<RefCell<Vec<u8>>>);

  impl TestWriter {
    pub fn new(buffer: &Arc<RefCell<Vec<u8>>>) -> TestWriter {
      TestWriter(Arc::clone(buffer))
    }
  }

  unsafe impl Send for TestWriter {}
  unsafe impl Sync for TestWriter {}

  impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
      Arc::make_mut(&mut self.0).get_mut().write(buf)
    }

    fn flush(&mut self) -> Result<()> {
      Arc::make_mut(&mut self.0).get_mut().flush()
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<usize> {
      Arc::make_mut(&mut self.0).get_mut().write_vectored(bufs)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
      Arc::make_mut(&mut self.0).get_mut().write_all(buf)
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> Result<()> {
      Arc::make_mut(&mut self.0).get_mut().write_fmt(fmt)
    }
  }
}

lazy_static::lazy_static! {
  // TODO: It looks as if slog_scopes does this by magic. Look into it
  pub(crate) static ref LOG: Mutex<YmLog> = Mutex::new(YmLog::new());
}

#[test]
fn test_recursion() {
  let buffer = Arc::new(RefCell::new(Vec::new()));
  let writer = common::TestWriter::new(&Arc::clone(&buffer));
  crate::LOG.lock().unwrap().set_output(writer);

  // Write a log statement
  fn recurse(remains: &mut Vec<i32>) {
    let value = remains.pop();
    if value.is_none() {
      return;
    }

    match value.as_ref().unwrap() % 5 {
      0 => ymlog!("R0, value is {:?}", value),
      1 => ymlog!("_+" => "R1, value is {:?}", value),
      2 => ymlog!("_" => "R2, Indented After value is{:?}", value),
      3 => ymlog!("+++_" => "R3, Indented After value is{:?}", value),
      // 3 =>  {
      // trace_macros!(true);
      // ymlog!("+_-" => {
      // msg => "R3, Bump Test, value is {:?}", value
      // }),
      // trace_macros!(false);
      // }
      4 => ymlog!("--_" => "Back to the the root level, value is {:?}", value),
      _ => unreachable!("Mod never gets above 5"),
    }
  }

  println!("The Log Contents now:\n{:?}", *buffer);
  let mut values: Vec<i32> = (1..100).collect();
  recurse(&mut values);
  /*

  ymlog! {
    "Hi, I'm an Info level message written at the current indentation level. {}",
    "The standard display formatter is implied by this"
  };

  */
  panic!("I'm done here")
}
