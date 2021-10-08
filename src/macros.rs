//! Macro rules for controlling the logger

/// Initialize a ymlogger
#[macro_export]
macro_rules! ymlogger {
  ($output:expr) => {{
    let mut logger = YmLog::new();
    logger.set_output($output);
    logger
  };};
}

/// Format and append a message to the log
///
///
#[macro_export]
macro_rules! ymlog {

  // --- Block Parameters

  (@msg $block:ident $msg:expr) => { let _ = $block.set_message($msg); };
  (@msg $block:ident $($msg:expr),+) => { let _ = $block.set_message(format!($($msg),+)); };

  (@block $block:ident $($key:expr => $($value:expr),+),+) => {
    $(ymlog!(@$key, $block, $($values),+));+
  };

  // --- Send the message
  (@send $block:ident $acts:ident) => {
    crate::LOG.lock().unwrap().log(&mut $block, $acts);
  };

  // --- Entry points

  // A bare message string
  ( $($msg:expr),+ ) => {{
    let mut block = ymlog::Block::new();
    ymlog!(@msg block $($msg),+);
    ymlog!(@send block None)
  }};

  // Block Only
  ( $params:block ) => {{
    let mut block = ymlog::Block::new();
    ymlog!(@block $block $params)
    ymlog!(@send block None)
  }};

  // Actions with a full Block
  ( $actions:expr => {$block_def:tt} ) => {{
    let acts = Some($actions);
    let mut block = ymlog::Block::new();
    ymlog!(@params block $block_def);
    ymlog!(@send block acts)
  }};

  // With Actions around a basic expression
  ( $actions:expr => $($msg:expr),+ ) => {{
    let acts = Some($actions);
    let mut block = ymlog::Block::new();
    ymlog!(@msg block $($msg),+);
    ymlog!(@send block acts)
  }};

}

#[macro_export]
macro_rules! ymlog_old {
  // ---  Main processors
  // A bare message string
  ( $($msg:expr),+ ) => {{
    let mut block = crate::Block::new();
    ymlog!(@msg block $($msg),+);
    // let _ = crate::LOG.lock().unwrap().print(None, &mut block);
  }};

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



  // ---  Parameters

  // Get params from a block
  (@params $block:ident $($type:literal => $params:tt),+) => {



  (@send $block:ident) => {
    println!("The Block: {:#+}", $block);
  }
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
