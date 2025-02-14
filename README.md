# `anyerr`

`anyerr` is a flexible and powerful error-handling library for Rust. It provides a highly customizable error type called `AnyError`, allowing you to capture and manage rich error information, including custom error kinds, backtraces, and contextual data.

## Features

- **Error Composition**: Combine and wrap errors while preserving original information.
- **Customizable Error Kinds**: Use predefined error kinds or create your own.
- **Contextual Data Support**: Attach rich, structured context to your errors for better debugging.
- **Backtrace Support**: Automatically captures backtraces to simplify error diagnosis.
- **Error Reporting**: Customize and write formated and detailed error messages to `stdout`, loggers and so on.

## Installation

`anyerr` is currently under active development, and a stable version will be published to [crates.io](https://crates.io) soon.

Add `anyerr` to your `Cargo.toml`:

```toml
[dependencies]
anyerr = { git = "https://github.com/oosquare/anyerr.git" }
```

## Getting Started

### Defining a Custom Error Type

`AnyError` is the core type of this library. It works similarly to `Box<dyn Error>`, but with added capabilities for handling contexts and custom error kinds. You can define your own error type with `AnyError` by choosing appropriate components.

Here's an example:

```rust
// Make this module accessible to your whole crate.
mod err {
    use anyerr::AnyError as AnyErrorTemplate;
    use anyerr::context::LiteralKeyStringMapContext;

    pub use anyerr::{Intermediate, Overlay};
    pub use anyerr::kind::DefaultErrorKind as ErrKind;
    pub use anyerr::Report;

    pub type AnyError = AnyErrorTemplate<LiteralKeyStringMapContext, ErrKind>;
    pub type AnyResult<T> = Result<T, AnyError>;
}
```

### Creating and Using Errors

You can create errors with different levels of detail, depending on your needs.

```rust
use err::*;

fn fail() -> AnyResult<()> {
    // Use `AnyError::minimal()` to create a simple [`String`]-based error.
    Err(AnyError::minimal("this function always fails"))
}

fn check_positive(x: i32) -> AnyResult<()> {
    if x > 0 {
        return Ok(());
    }
    // Use `AnyError::quick()` to quickly create an error with an error
    // message and an error kind.
    Err(AnyError::quick(
        "expects `x` to be a positive number",
        ErrKind::ValueValidation
    ))
}

fn try_add_username(
    usernames: &mut Vec<String>,
    new_username: String
) -> AnyResult<usize> {
    let res = usernames.iter()
        .enumerate()
        .find(|(_, username)| **username == new_username)
        .map(|(index, _)| index);
    if let Some(index) = res {
        // Use `AnyError::builder()` to create an error with all essential
        // context you'll need.
        let err = AnyError::builder()
            .message("the username already exists")
            .kind(ErrKind::RuleViolation)
            .context("new_username", new_username)
            .context("index", index)
            .build();
        Err(err)
    } else {
        usernames.push(new_username);
        Ok(usernames.len() - 1)
    }
}

fn parse_i32(input: &str) -> AnyResult<i32> {
    // Use `AnyError::wrap()` to wrap any other error type.
    input.parse::<i32>().map_err(AnyError::wrap)
}
```

Using `AnyError` is also rather straightforward.

```rust
use err::*;

fn main() {
    let mut usernames = Vec::new();

    let res = try_add_username(&mut usernames, "foo").unwrap();
    assert_eq!(res, 0);

    let err = try_add_username(&mut usernames, "foo").unwrap_err();
    assert_eq!(err.to_string(), "the username already exists"); // Or `err.message()`.
    assert_eq!(err.kind(), ErrKind::RuleViolation);
    assert_eq!(err.get("new_username"), Some("\"foo\""));
    assert_eq!(err.get("index"), Some("0"));
}
```

### Chaining Errors

`AnyError` allows attaching chaining errors for better logging and easier debugging.

```rust
use err::*;

struct UserRepository {
    conn: Arc<Connection>,
}

impl UserRepository {
    pub fn find_by_username(&self, username: &str) -> AnyResult<User> {
        // Don't build SQL statements yourself in practice.
        let statement = format!("SELECT * FROM users WHERE users.username = '{username}'");
        let data = self.conn.query(&statement)
            .overlay(("could not get a `User` due to SQL execution error", ErrKind::EntityAbsence))
            .context("username", username)
            .context("statement", statement)?;
        let entity = User::try_from(data)
            .overlay(("could not get a `User` due to serialization error", Errkind::EntityAbsence))
            .context("username", username)?;
        Ok(entity)
    }
}
```

### Error Reporting

Use `Report` to display a formated error message with details:

```rust
use err::*;

fn source_error() -> AnyResult<()> {
    let err = AnyError::builder()
        .message("the source error is here")
        .kind(ErrKind::InfrastructureFailure)
        .context("key1", "value1")
        .context("key2", "value2")
        .build();
    Err(err)
}

fn intermediate_error() -> AnyResult<()> {
    source_error()
        .overlay("the intermediate error is here")
        .context("key3", "value3")?;
    Ok(())
}

fn toplevel_error() -> AnyResult<()> {
    intermediate_error()
        .overlay("the toplevel error is here")?;
    Ok(())
}

fn main() -> impl Termination {
    Report::capture(|| {
        toplevel_error()?;
        Ok(())
    })
}
```

The expected output is

```plain
Error:
    (Unknown) the toplevel error is here
Caused by:
    (Unknown) the intermediate error is here
    [key3 = "value3"]
Caused by:
    (InfrastructureFailure) the source error is here
    [key1 = "value1", key2 = "value2"]

Stack backtrace:
  0: anyerr::core::data::ErrorDataBuilder<C,K>::build
            at ./src/core/data.rs:210:28
  1: anyerr::core::AnyErrorBuilder<C,K>::build
            at ./src/core.rs:415:24
  2: anyerr::source_error
            at ./src/main.rs:18:15
  3: anyerr::intermediate_error
            at ./src/main.rs:28:5
  4: anyerr::toplevel_error
            at ./src/main.rs:35:5
  5: anyerr::main::{{closure}}
            at ./src/main.rs:40:43
  6: anyerr::report::Report<C,K>::capture
            at ./src/report.rs:52:15
  7: anyerr::main
            at ./src/main.rs:40:5
   ...
```

### Advanced Usage

See API documentation for more features and advanced usages of different types in this crate.

## License

Copyright (C) 2025 Justin Chen

This project is licensed under the [MIT License](https://github.com/oosquare/anyerr/blob/main/LICENSE).

