//! Macro rules for controlling the logger

/// Initialize a ymlogger
#[macro_export]
macro_rules! ymlogger {
  () => {};
}

// Macro to quickly build a block
#[macro_export]
macro_rules! ymlog {
  // ---  Main processors
  // A bare message string
  ( $($msg:expr),+ ) => {
    let mut block = crate::logger::Block::new();
    ymlog!(@msg block $($msg),+);
    let _ = crate::LOG.lock().unwrap().print(None, &mut block);
  };

  // A block definition with no preceding options
  ( $block:tt ) => {
    let mut block = crate::logger::Block::new();
    ymlog!(@params block $block);
    let _ = crate::LOG.lock().unwrap().print(None, &mut block);
  };

  // Starts with a string of action tokens
  ($tokens:tt => $($rest:expr),+) => {
    let mut block = crate::logger::Block::new();
    ymlog!(@msg block $($rest),+);
    let _ = crate::LOG.lock().unwrap().print(Some($tokens), &mut block).map_err(|x| panic!("Got an error printing the log: {}", x));
  };

  // Starts with a string of action tokens
  ($tokens:tt => $params:tt) => {
    let mut block = crate::logger::Block::new();
    ymlog!(@params block $params);
    let _ = crate::LOG.lock().unwrap().print(Some($tokens), &mut block);
  };



  // ---  Helper parsers
  // Get params from a block
  (@params $block:ident $($type:literal => $params:tt),+) => {


  };

  // Set the params from the block
  (@param $block:ident stamp) => {};
  (@param $block:ident level => $level:ident) => {};
  (@param $block:ident msg => $($msg:expr),+) => {

  };

  // Format a string message
  (@msg $block:ident $msg:expr) => { $block.set_message($msg); };
  (@msg $block:ident $($msg:expr),+) => { $block.set_message(format!($($msg),+)); };


}
