# yamlog

YAMLog is a streaming YAML log file writer. Because of the yaml, it is indented and parsable by
default. This is meant to help debug recursive and macro programming, where visually being able to
see nesting in the logs is beneficial. In addition, the output valid YAML after a crash, unlike
JSON which cannot be parsed until all the brackets, braces, and parens are closed.

## Alpha 0.1 release

Make similar or build upon env_logger.

Standard documentation and test coverage of the current features. Functionality has reached "good
enough" even though pretty printing YAML blocks is hacked in. Fixing it requires going deep
downstream to the rust-yaml project, as well as serde_yaml.

## Use Cases

The basic use is to emulate the debugger's step into/out functionality in an output log file. When
open in an IDE, it is easy to find the log statements from the final iteration run before crashing.

## Syntax

```rust
//A Simple default message
yamlog! {
  "Hi, I'm an Info level message written at the current indentation level. {}",
  "The standard display formatter is implied by this"
}

yamlog! {
  "T+_-" => "Set to Trace level, and indent this message. The next one have the prior indent"
}

```

Commands
