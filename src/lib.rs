//! `anyerr` is a comprehensive error handling library designed to offer
//! flexibility, extensibility, and an ergonomic way to handle errors in Rust
//! applications.
//!
//! This library provides a central [`AnyError`] type that can carry arbitrary
//! error information, including a custom error kind, a backtrace, contextual
//! data and so on. It enables developers to create error types composing
//! different levels of errors without sacrificing the ability to preserve rich
//! context information.
//!
//! ## Key Features
//!
//! - **Error Composition**: Wrap and combine errors while preserving their
//!   original information and access the underlying errors if needed.
//! - **Customizable Error Kind**: Make use of predefined error kinds offered
//!   by this crate or define your own error kinds by implementing the [`Kind`]
//!   trait.
//! - **Contextual Data**: Attach rich context information to errors using
//!   different pluggable context types.
//! - **Backtrace Support**: Automatically captures backtraces for easier
//!   debugging.
//! - **Error Reporting**: Customize and write formated and detailed error
//!   messages to `stdout`, loggers and so on.
//!
//! ## Getting Started
//!
//! ### Defining a Custom Error Type
//!
//! [`AnyError`] is the core of this crate. It works in a way resembled to
//! [`Box<dyn Error>`], by implementing the [`Error`] trait and leverage the
//! functionality of the [`Any`] trait, while it's also [`Send`] and [`Sync`],
//! allowing safely accesses across multiple concurrent threads. [`AnyError`]
//! is easy to get started with, though, it's not somthing like
//! [`Box<dyn Error>`] that can be used directly in your codebase, but a highly
//! customizable type requiring you to make decisions about its components.
//!
//! An [`AnyError<C, K>`] has two generic type parameters `C` and `K`, stand
//! for the context storage and the error kind respectively.
//!
//! [`AbstractContext`] is implemented for `C`, so is [`Context`] usually but
//! it's not required. With `C` implementing [`Context`], you can attach
//! additional contextual data to an [`AnyError`] for better debugging. An
//! example of one of the most useful contexts is
//! [`LiteralKeyStringMapContext`], which holds entries of a `&'static str`
//! and `String` pair structure, and stores the the [`Debug`] representation
//! of values.
//!
//! `K` is required to implement the trait [`Kind`], specifying a general kind
//! of the error. Although a structured error handling style is not preferred
//! under this circumstance, an error kind enables more fine-grained logging
//! and tracing or enhances experience of other aspects. [`DefaultErrorKind`]
//! is a [`Kind`] provided by this crate, and the design of its variant is
//! based on the author's web backend developemnt experience.
//!
//! Once you have chosen the components you need, you can define your custom
//! error type, by supplying [`AnyError`] with the selected context and error
//! kind. Here's an example:
//!
//! ```rust
//! // Make this module accessible to your whole crate.
//! mod err {
//!     use anyerr::AnyError as AnyErrorTemplate;
//!     use anyerr::context::LiteralKeyStringMapContext;
//!
//!     pub use anyerr::{Intermediate, Overlay}; // These are helper traits.
//!     pub use anyerr::kind::DefaultErrorKind as ErrKind;
//!     pub use anyerr::Report;
//!
//!     pub type AnyError = AnyErrorTemplate<LiteralKeyStringMapContext, ErrKind>;
//!     pub type AnyResult<T> = Result<T, AnyError>;
//! }
//! // Include this in wherever you need `AnyError`.
//! use err::*;
//! ```
//!
//! ### Creating and Using Errors
//!
//! Here's how to create [`AnyError`] in your application:
//!
//! ```rust
//! # mod err {
//! #     use anyerr::AnyError as AnyErrorTemplate;
//! #     use anyerr::context::LiteralKeyStringMapContext;
//! #
//! #     pub use anyerr::{Intermediate, Overlay};
//! #     pub use anyerr::kind::DefaultErrorKind as ErrKind;
//! #     pub use anyerr::Report;
//! #
//! #     pub type AnyError = AnyErrorTemplate<LiteralKeyStringMapContext, ErrKind>;
//! #     pub type AnyResult<T> = Result<T, AnyError>;
//! # }
//! use err::*;
//!
//! fn fail() -> AnyResult<()> {
//!     // Use `AnyError::minimal()` to create a simple [`String`]-based error.
//!     Err(AnyError::minimal("this function always fails"))
//! }
//!
//! fn check_positive(x: i32) -> AnyResult<()> {
//!     if x > 0 {
//!         return Ok(());
//!     }
//!     // Use `AnyError::quick()` to quickly create an error with an error
//!     // message and an error kind.
//!     Err(AnyError::quick(
//!         "expects `x` to be a positive number",
//!         ErrKind::ValueValidation
//!     ))
//! }
//!
//! fn try_add_username(
//!     usernames: &mut Vec<String>,
//!     new_username: String
//! ) -> AnyResult<usize> {
//!     let res = usernames.iter()
//!         .enumerate()
//!         .find(|(_, username)| **username == new_username)
//!         .map(|(index, _)| index);
//!     if let Some(index) = res {
//!         // Use `AnyError::builder()` to create an error with all essential
//!         // context you'll need.
//!         let err = AnyError::builder()
//!             .message("the username already exists")
//!             .kind(ErrKind::RuleViolation)
//!             .context("new_username", new_username)
//!             .context("index", index)
//!             .build();
//!         Err(err)
//!     } else {
//!         usernames.push(new_username);
//!         Ok(usernames.len() - 1)
//!     }
//! }
//!
//! fn parse_i32(input: &str) -> AnyResult<i32> {
//!     // Use `AnyError::wrap()` to wrap any other error type.
//!     input.parse::<i32>().map_err(AnyError::wrap)
//! }
//! ```
//!
//! Let's take the third function `try_add_username()` as an example to
//! demonstrate how we can use [`AnyError`]:
//!
//! ```rust
//! # mod err {
//! #     use anyerr::AnyError as AnyErrorTemplate;
//! #     use anyerr::context::LiteralKeyStringMapContext;
//! #
//! #     pub use anyerr::{Intermediate, Overlay};
//! #     pub use anyerr::kind::DefaultErrorKind as ErrKind;
//! #     pub use anyerr::Report;
//! #
//! #     pub type AnyError = AnyErrorTemplate<LiteralKeyStringMapContext, ErrKind>;
//! #     pub type AnyResult<T> = Result<T, AnyError>;
//! # }
//! #
//! use err::*;
//! #
//! # fn try_add_username<S: Into<String>>(
//! #     usernames: &mut Vec<String>,
//! #     new_username: S
//! # ) -> AnyResult<usize> {
//! #     let new_username = new_username.into();
//! #     let res = usernames.iter()
//! #         .enumerate()
//! #         .find(|(_, username)| **username == new_username)
//! #         .map(|(index, _)| index);
//! #     if let Some(index) = res {
//! #         // Use `AnyError::builder()` to create an error with all essential
//! #         // context you'll need.
//! #         let err = AnyError::builder()
//! #             .message("the username already exists")
//! #             .kind(ErrKind::RuleViolation)
//! #             .context("new_username", new_username)
//! #             .context("index", index)
//! #             .build();
//! #         Err(err)
//! #     } else {
//! #         usernames.push(new_username);
//! #         Ok(usernames.len() - 1)
//! #     }
//! # }
//!
//! fn main() {
//!     let mut usernames = Vec::new();
//!
//!     let res = try_add_username(&mut usernames, "foo").unwrap();
//!     assert_eq!(res, 0);
//!
//!     let err = try_add_username(&mut usernames, "foo").unwrap_err();
//!     assert_eq!(err.to_string(), "the username already exists"); // Or `err.message()`.
//!     assert_eq!(err.kind(), ErrKind::RuleViolation);
//!     assert_eq!(err.get("new_username"), Some("\"foo\""));
//!     assert_eq!(err.get("index"), Some("0"));
//! }
//! ```
//!
//! ### Error Wrapping and Chaining
//!
//! The `AnyError` type supports convenient error wrapping, allowing you to
//! maintain the original error while adding additional context. Methods in
//! the [`Overlay`] and [`Intermediate`] helper traits provides ergonomic
//! means for you to make an overlay of your existing error and attach rich
//! context to it.
//!
//! Say we'd like to reteive a `User` entity by its username from the
//! `UserRepository`. It's acknowledged that the query may fails due to a
//! variety of reasons, but we don't care about the details but whether we
//! could get that entity. The following codeblock demonstrates this idea.
//!
//! ```no_run,rust
//! # mod err {
//! #     use anyerr::AnyError as AnyErrorTemplate;
//! #     use anyerr::context::LiteralKeyStringMapContext;
//! #
//! #     pub use anyerr::{Intermediate, Overlay};
//! #     pub use anyerr::kind::DefaultErrorKind as ErrKind;
//! #     pub use anyerr::Report;
//! #
//! #     pub type AnyError = AnyErrorTemplate<LiteralKeyStringMapContext, ErrKind>;
//! #     pub type AnyResult<T> = Result<T, AnyError>;
//! # }
//! #
//! # use std::sync::Arc;
//! use err::*;
//! #
//! # struct User;
//! #
//! # struct Data;
//! #
//! # type DeserializationError = AnyError;
//! #
//! # impl TryFrom<Data> for User {
//! #     type Error = DeserializationError;
//! #
//! #     fn try_from(_data: Data) -> Result<User, Self::Error> {
//! #         Err(DeserializationError::minimal("Could not deserialize a `User` from the given data"))
//! #     }
//! # }
//! # struct Connection;
//! #
//! # type DbError = AnyError;
//! #
//! # impl Connection {
//! #     fn query(&self, statement: &str) -> Result<Data, DbError> {
//! #         Err(DbError::minimal("could not run the SQL query"))
//! #     }
//! # }
//!
//! struct UserRepository {
//!     conn: Arc<Connection>,
//! }
//!
//! impl UserRepository {
//!     pub fn find_by_username(&self, username: &str) -> AnyResult<User> {
//!         // Don't build SQL statements yourself in practice.
//!         let statement = format!("SELECT * FROM users WHERE users.username = '{username}'");
//!         let data = self.conn.query(&statement)
//!             .overlay(("could not get a `User` due to SQL execution error", ErrKind::EntityAbsence))
//!             .context("username", username)
//!             .context("statement", statement)?;
//!         let entity = User::try_from(data)
//!             .overlay(("could not get a `User` due to serialization error", ErrKind::EntityAbsence))
//!             .context("username", username)?;
//!         Ok(entity)
//!     }
//! }
//! ```
//!
//! ### Error Reporting
//!
//! You might have the experience that you wrote the code which iterated over
//! the error chain and formated causes. It's pretty tedious to manually and
//! repeatly write such code. Therefore, this crate does this for you by
//! providing [`Report`]. [`Report`] captures your function's result and then
//! you can output the error report directly to terminals, loggers or whatever.
//!
//! ```rust
//! # mod err {
//! #     use anyerr::AnyError as AnyErrorTemplate;
//! #     use anyerr::context::LiteralKeyStringMapContext;
//! #
//! #     pub use anyerr::{Intermediate, Overlay};
//! #     pub use anyerr::kind::DefaultErrorKind as ErrKind;
//! #     pub use anyerr::Report;
//! #
//! #     pub type AnyError = AnyErrorTemplate<LiteralKeyStringMapContext, ErrKind>;
//! #     pub type AnyResult<T> = Result<T, AnyError>;
//! # }
//! #
//! use err::*;
//!
//! fn source_error() -> AnyResult<()> {
//!     let err = AnyError::builder()
//!         .message("the source error is here")
//!         .kind(ErrKind::InfrastructureFailure)
//!         .context("key1", "value1")
//!         .context("key2", "value2")
//!         .build();
//!     Err(err)
//! }
//!
//! fn intermediate_error() -> AnyResult<()> {
//!     source_error()
//!         .overlay("the intermediate error is here")
//!         .context("key3", "value3")?;
//!     Ok(())
//! }
//!
//! fn toplevel_error() -> AnyResult<()> {
//!     intermediate_error()
//!         .overlay("the toplevel error is here")?;
//!     Ok(())
//! }
//!
//! let report1 = Report::wrap(toplevel_error().unwrap_err()).pretty(false);
//! let report2 = Report::capture(|| -> AnyResult<()> { toplevel_error() });
//! println!("Error: {report1}");
//! println!("{report2}");
//! ```
//!
//! The output of `report1`:
//!
//! ```ignored,plain
//! Error: (Unknown) the toplevel error is here: (Unknown) the intermediate error is here: (InfrastructureFailure) the source error is here [key3 = "value3", key1 = "value1", key2 = "value2"]
//! ```
//!
//! The output of `report2`:
//!
//! ```ignored,plain
//! Error:
//!     (InfrastructureFailure) the toplevel error is here
//! Caused by:
//!     (Unknown) the intermediate error is here
//!     [key3 = "value3"]
//! Caused by:
//!     (Unknown) the source error is here
//!     [key1 = "value1", key2 = "value2"]
//!
//! Stack backtrace:
//!   0: anyerr::core::data::ErrorDataBuilder<C,K>::build
//!             at ./src/core/data.rs:210:28
//!   1: anyerr::core::AnyErrorBuilder<C,K>::build
//!             at ./src/core.rs:415:24
//!   2: anyerr::source_error
//!             at ./src/main.rs:18:15
//!   3: anyerr::intermediate_error
//!             at ./src/main.rs:28:5
//!   4: anyerr::toplevel_error
//!             at ./src/main.rs:35:5
//!   5: anyerr::main::{{closure}}
//!             at ./src/main.rs:40:43
//!   6: anyerr::report::Report<C,K>::capture
//!             at ./src/report.rs:52:15
//!   7: anyerr::main
//!             at ./src/main.rs:40:5
//!    ...
//! ```
//!
//! Using [`Report`] in `main()`'s returning position is also allowed:
//!
//! ```no_run,rust
//! # mod err {
//! #     use anyerr::AnyError as AnyErrorTemplate;
//! #     use anyerr::context::LiteralKeyStringMapContext;
//! #
//! #     pub use anyerr::{Intermediate, Overlay};
//! #     pub use anyerr::kind::DefaultErrorKind as ErrKind;
//! #     pub use anyerr::Report;
//! #
//! #     pub type AnyError = AnyErrorTemplate<LiteralKeyStringMapContext, ErrKind>;
//! #     pub type AnyResult<T> = Result<T, AnyError>;
//! # }
//! #
//! use std::process::Termination;
//! use err::*;
//! #
//! #  fn source_error() -> AnyResult<()> {
//! #     let err = AnyError::builder()
//! #         .message("the source error is here")
//! #         .kind(ErrKind::InfrastructureFailure)
//! #         .context("key1", "value1")
//! #         .context("key2", "value2")
//! #         .build();
//! #     Err(err)
//! # }
//! #
//! # fn intermediate_error() -> AnyResult<()> {
//! #     source_error()
//! #         .overlay("the intermediate error is here")
//! #         .context("key3", "value3")?;
//! #     Ok(())
//! # }
//! #
//! # fn toplevel_error() -> AnyResult<()> {
//! #     intermediate_error()
//! #         .overlay("the toplevel error is here")?;
//! #     Ok(())
//! # }
//!
//! fn main() -> impl Termination {
//!     Report::capture(|| {
//!         toplevel_error()?;
//!         Ok(())
//!     })
//! }
//! ```
//!
//! For more information about error reporting customization, see the
//! documentations of [`Report`].
//!
//! ## Advanced Usage
//!
//! ### Different Context Types
//!
//! This crate allows using different context types, such as
//! [`SingletonContext`], [`StringContext`], [`AnyContext`] or the ones you
//! developed by yourself, depending on how you want to manage and retrieve
//! additional information from your errors. It's even viable that you don't
//! want your error type to carry a context storage, through the [`NoContext`]
//! trait. Each context type offers unique capabilities for structuring error
//! metadata.
//!
//! For more information, refer to the types in the [`crate::context`] module.
//!
//! ### Usage without an Error Kind
//!
//! For some reasons, you may not want each error to have an error kind. This
//! crate offers you [`NoErrorKind`], which actually has only one variant as
//! its default value. By selecting [`NoErrorKind`], you no longer need to
//! do anything with error kinds.
//!
//! [`Any`]: std::any::Any
//! [`AbstractContext`]: crate::context::AbstractContext
//! [`Context`]: crate::context::Context
//! [`Debug`]: std::fmt::Debug
//! [`DefaultErrorKind`]: crate::kind::DefaultErrorKind
//! [`Error`]: std::error::Error
//! [`Kind`]: crate::kind::Kind
//! [`LiteralKeyStringMapContext`]: crate::context::LiteralKeyStringMapContext
//! [`NoContext`]: crate::context::NoContext
//! [`NoErrorKind`]: crate::kind::NoErrorKind
//! [`SingletonContext`]: crate::context::SingletonContext
//! [`StringContext`]: crate::context::StringContext
//! [`AnyContext`]: crate::context::AnyContext

pub mod context;
pub mod converter;
pub mod core;
pub mod kind;
pub mod overlay;
pub mod report;

pub use core::AnyError;
pub use overlay::{Intermediate, Overlay};
pub use report::Report;
