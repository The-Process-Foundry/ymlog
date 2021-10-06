# ymlog

ymlog is a streaming YAML log file writer. Because of the yaml, it is indented and parsable by
default. This is meant to help debug recursive and macro programming, where visually being able to
see nesting in the logs would be beneficial. In addition, the output is always valid YAML, unlike
JSON which cannot be parsed until the initial opening bracket is closed.

## Alpha 0.1 release

I'm likely going to make this into an official slog crate. The alpha will require reading how the
others like slog-json are implemented.

And, of course, standard documentation and test ccoverage.

## Use Cases

## Syntax

Much of the design is wrapped in macros, so that is what we shall use. It is built upon the slog
project.

```rust
//A Simple default message
ymlog! {
  "Hi, I'm an Info level message written at the current indentation level. {}",
  "The standard display formatter is implied by this"
}


```

Commands
