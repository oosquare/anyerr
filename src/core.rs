mod data;

use std::any::{Any, TypeId};
use std::backtrace::Backtrace;
use std::borrow::Borrow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::Hash;

use crate::context::{AbstractContext, Context, Entry};
use crate::converter::Convertable;
use crate::kind::Kind;

use data::{ErrorData, ErrorDataBuilder};

/// The central type for general error handling and reporting.
///
/// With the [`Error`] trait implemented, [`AnyError`] can be used in any place
/// where you expect an error type should work. Not only can it carry either a
/// [`String`]-based error message or an arbitrary error type, but also supports
/// sophisticated error wrapping and context recording, with the help of
/// [`Overlay`] and [`Context`] trait and so on.
///
/// An [`AnyError`] typically holds the following information:
///
/// - An error message, which describes the reason why the error occurs
/// - An error kind implementing [`Kind`], which is supplied on your own
/// - A backtrace capturing a snapshot of the call stack at that point
/// - The source error wrapped in this [`AnyError`]
/// - Some additional context
///
/// A leaf [`AnyError`] can be instantiated with associative functions or its
/// builder through [`AnyError::builder()`], while an intermediate [`AnyError`]
/// which contains another error is often produced with the usage of the
/// [`Overlay`] trait.
///
/// When using [`AnyError`] in your own crate, the recommanded way is to define
/// your specific error kind implementing the [`Kind`] trait and choose a
/// context container which typically implements the [`Context`] trait and then
/// make a type alias for the concrete [`AnyError`] type. The following example
/// shows this:
///
/// ```rust
/// use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
///
/// use anyerr::{AnyError as AnyErrorTemplate, Overlay};
/// use anyerr::context::LiteralKeyStringMapContext;
/// use anyerr::kind::Kind;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// #[non_exhaustive]
/// pub enum ErrKind {
///     ValueValidation,
///     RuleViolation,
///     InfrastructureFailure,
///     EntityAbsence,
///     Raw,
///     #[default]
///     Unknown,
/// }
///
/// impl Display for ErrKind {
///     fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
///         let value = match self {
///             Self::ValueValidation => "ValueValidation",
///             Self::RuleViolation => "RuleViolation",
///             Self::EntityAbsence => "EntityAbsence",
///             Self::InfrastructureFailure => "InfrastructureFailure",
///             Self::Raw => "Raw",
///             Self::Unknown => "Unknown",
///         };
///         write!(f, "{value}")
///     }
/// }
///
/// impl Kind for ErrKind {
///     const RAW_KIND: Self = Self::Raw;
///     const UNKNOWN_KIND: Self = Self::Unknown;
/// }
///
/// pub type AnyError = AnyErrorTemplate<LiteralKeyStringMapContext, ErrKind>;
///
/// fn parse_int(text: &str) -> Result<i32, AnyError> {
///     text.parse::<i32>().map_err(AnyError::wrap)
/// }
///
/// fn do_parsing() -> Result<(), AnyError> {
///     let res = parse_int("not i32").overlay("failed to parse the text")?;
///     println!("Got an `i32`: {res}");
///     Ok(())
/// }
///
/// fn main() {
///     if let Err(err) = do_parsing() {
///         eprintln!("Error: {err}");
///     }
/// }
/// ```
///
/// [`Overlay`]: `crate::overlay::Overlay`
#[derive(Debug)]
pub struct AnyError<C, K>(Box<ErrorData<C, K>>)
where
    C: AbstractContext,
    K: Kind;

impl<C, K> AnyError<C, K>
where
    C: AbstractContext,
    K: Kind,
{
    /// Makes an [`AnyError`] with the given error message and use
    /// `K::default()` as the default kind.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use anyerr::AnyError as AnyErrorTemplate;
    /// # use anyerr::kind::DefaultErrorKind;
    /// # use anyerr::context::LiteralKeyStringMapContext;
    /// type AnyError = AnyErrorTemplate<LiteralKeyStringMapContext, DefaultErrorKind>;
    /// let err = AnyError::minimal("an error occurred");
    /// assert_eq!(err.to_string(), "an error occurred");
    /// ```
    pub fn minimal<S: Into<String>>(message: S) -> Self {
        Self::from(ErrorData::<C, K>::Simple {
            kind: K::default(),
            message: message.into(),
            backtrace: Backtrace::capture(),
            context: C::default(),
        })
    }

    /// Makes an [`AnyError`] with the given error message and kind, which is a
    /// quick way to report an error if no additional context is required.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use anyerr::AnyError as AnyErrorTemplate;
    /// # use anyerr::kind::DefaultErrorKind;
    /// # use anyerr::context::LiteralKeyStringMapContext;
    /// type AnyError = AnyErrorTemplate<LiteralKeyStringMapContext, DefaultErrorKind>;
    /// let err = AnyError::quick("a positive number is expected", DefaultErrorKind::ValueValidation);
    /// assert_eq!(err.to_string(), "a positive number is expected");
    /// assert_eq!(err.kind(), DefaultErrorKind::ValueValidation);
    /// ```
    pub fn quick<S: Into<String>>(message: S, kind: K) -> Self {
        Self::from(ErrorData::<C, K>::Simple {
            kind,
            message: message.into(),
            backtrace: Backtrace::capture(),
            context: C::default(),
        })
    }

    /// Wraps an arbitrary error in an [`AnyError`] with a backtrace attached.
    /// Note that if this error is already an [`AnyError`], it'll be returned
    /// directly.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use anyerr::AnyError as AnyErrorTemplate;
    /// # use anyerr::kind::DefaultErrorKind;
    /// # use anyerr::context::LiteralKeyStringMapContext;
    /// type AnyError = AnyErrorTemplate<LiteralKeyStringMapContext, DefaultErrorKind>;
    /// let err = "not i32".parse::<i32>().map_err(AnyError::wrap);
    /// // do anything with `err` in a universal fashion on the fly ...
    /// ```
    pub fn wrap<E>(err: E) -> Self
    where
        E: Error + Any + Send + Sync,
    {
        let err: Box<dyn ErrorAndAny> = Box::new(err);
        if err.as_any().is::<Self>() {
            *err.as_any_boxed()
                .downcast::<Self>()
                .expect("`err` should be `Box<Self>`")
        } else {
            let data = ErrorData::Wrapped {
                backtrace: Backtrace::capture(),
                inner: err.as_error_boxed(),
            };
            Self::from(data)
        }
    }

    /// Returns a dedicated builder [`AnyErrorBuilder`] to instantiate an
    /// [`AnyError`].
    pub fn builder() -> AnyErrorBuilder<C, K> {
        AnyErrorBuilder::new()
    }

    /// Returns the kind of this error.
    pub fn kind(&self) -> K {
        self.0.kind()
    }

    /// Returns the error message of this error.
    pub fn message(&self) -> String {
        self.0.message()
    }

    /// Returns the backtrace captured where the deepest error occurred.
    pub fn backtrace(&self) -> &Backtrace {
        self.0.backtrace()
    }

    /// Returns an iterator which iterates over either all attached context
    /// or those added to the most outer error.
    pub fn context(&self, depth: ContextDepth) -> C::Iter<'_> {
        self.0.context(depth)
    }

    /// Returns true if the inner type is the same as `E`. Note that the error
    /// is not equivalent to the source error, which stands for the current
    /// [`AnyError`]'s cause, while the former means the external error type
    /// wrapped in this [`AnyError`]. Apart from the case where an external
    /// error is wrapped, an [`AnyError`] is considered to be itself, i.e.
    /// [`AnyError::is<E>()`] returns `true` if and only if `E` is [`AnyError`].
    pub fn is<E>(&self) -> bool
    where
        E: Error + Send + Sync + 'static,
    {
        match &*self.0 {
            ErrorData::Simple { .. } => TypeId::of::<E>() == TypeId::of::<Self>(),
            ErrorData::Layered { .. } => TypeId::of::<E>() == TypeId::of::<Self>(),
            ErrorData::Wrapped { inner, .. } => inner.is::<E>(),
        }
    }

    /// Attempts to downcast the [`AnyError`] to the inner error of type `E`.
    ///
    /// # Errors
    ///
    /// This function will return the original [`AnyError`] if the actual type
    /// of the inner error is not `E`.
    pub fn downcast<E>(self) -> Result<E, Self>
    where
        E: Error + Send + Sync + 'static,
    {
        match *self.0 {
            ErrorData::Simple { .. } | ErrorData::Layered { .. } => {
                if self.is::<E>() {
                    let boxed: Box<dyn Any> = Box::new(self);
                    boxed
                        .downcast::<E>()
                        .map(|res| *res)
                        .map_err(|_| unreachable!("`boxed` should be `Box<E>` (i.e. `Box<Self>`)"))
                } else {
                    Err(self)
                }
            }
            ErrorData::Wrapped { inner, backtrace } => inner
                .downcast::<E>()
                .map(|res| *res)
                .map_err(|inner| Self::from(ErrorData::Wrapped { backtrace, inner })),
        }
    }

    /// Returns some reference to the inner error if it is of type `E`, or
    /// `None` if it isn't.
    pub fn downcast_ref<E>(&self) -> Option<&E>
    where
        E: Error + Send + Sync + 'static,
    {
        match &*self.0 {
            ErrorData::Simple { .. } | ErrorData::Layered { .. } => {
                let any: &dyn Any = self;
                any.downcast_ref::<E>()
            }
            ErrorData::Wrapped { inner, .. } => inner.downcast_ref::<E>(),
        }
    }

    /// Returns some mutable reference to the inner error if it is of type `E`,
    /// or `None` if it isn't.
    pub fn downcast_mut<E>(&mut self) -> Option<&mut E>
    where
        E: Error + Send + Sync + 'static,
    {
        let use_inner = matches!(&*self.0, ErrorData::Wrapped { .. });
        if !use_inner {
            let any: &mut dyn Any = self;
            any.downcast_mut::<E>()
        } else {
            let ErrorData::Wrapped { inner, .. } = &mut *self.0 else {
                unreachable!("`self.data` matches `ErrorData::Wrapped {{ .. }}`");
            };
            inner.downcast_mut::<E>()
        }
    }
}

impl<C, K> AnyError<C, K>
where
    C: crate::context::SingletonContext,
    K: Kind,
{
    /// Returns the context information carried by this [`AnyError<C, K>`],
    /// where `C` is a [`SingletonContext`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use anyerr::AnyError as AnyErrorTemplate;
    /// # use anyerr::kind::DefaultErrorKind;
    /// # use anyerr::context::StringSingletonContext;
    /// type AnyError = AnyErrorTemplate<StringSingletonContext, DefaultErrorKind>;
    /// let err = AnyError::builder().message("err").context((), "ctx").build();
    /// assert_eq!(err.value(), Some("ctx"));
    /// ```
    ///
    /// [`SingletonContext`]: `crate::context::SingletonContext`
    pub fn value(&self) -> Option<&<C::Entry as Entry>::ValueBorrowed> {
        self.0.value()
    }
}

impl<C, K> AnyError<C, K>
where
    C: crate::context::StringContext,
    K: Kind,
{
    /// Returns the context information carried by this [`AnyError<C, K>`] by
    /// `key`, where `C` is a [`StringContext`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use anyerr::AnyError as AnyErrorTemplate;
    /// # use anyerr::kind::DefaultErrorKind;
    /// # use anyerr::context::LiteralKeyStringMapContext;
    /// type AnyError = AnyErrorTemplate<LiteralKeyStringMapContext, DefaultErrorKind>;
    /// let err = AnyError::builder()
    ///     .message("err")
    ///     .context("&str", "value")
    ///     .context("i32", 42)
    ///     .build();
    /// assert_eq!(err.get("&str"), Some("\"value\""));
    /// assert_eq!(err.get("i32"), Some("42"));
    /// ```
    ///
    /// [`StringContext`]: `crate::context::StringContext`
    pub fn get<Q>(&self, key: &Q) -> Option<&<C::Entry as Entry>::ValueBorrowed>
    where
        <C::Entry as Entry>::KeyBorrowed: Borrow<Q>,
        Q: Debug + Eq + Hash + ?Sized,
    {
        self.0.get(key)
    }
}

impl<C, K> AnyError<C, K>
where
    C: crate::context::AnyContext,
    K: Kind,
{
    /// Returns the context information carried by this [`AnyError<C, K>`] by
    /// `key` and then attempts to convert the result to a `T`, where `C` is a
    /// [`AnyContext`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use anyerr::AnyError as AnyErrorTemplate;
    /// # use anyerr::kind::DefaultErrorKind;
    /// # use anyerr::context::AnyMapContext;
    /// type AnyError = AnyErrorTemplate<AnyMapContext<String, str>, DefaultErrorKind>;
    /// let err = AnyError::builder()
    ///     .message("err")
    ///     .context("&str", "value")
    ///     .context("i32", 42)
    ///     .build();
    /// assert_eq!(err.value_as::<&'static str, _>("&str"), Some(&"value"));
    /// assert_eq!(err.value_as::<i32, _>("i32"), Some(&42));
    /// ```
    ///
    /// [`AnyContext`]: `crate::context::AnyContext`
    pub fn value_as<T, Q>(&self, key: &Q) -> Option<&T>
    where
        <C::Entry as Entry>::KeyBorrowed: Borrow<Q>,
        Q: Debug + Eq + Hash + ?Sized,
        T: Any,
    {
        self.0.value_as::<T, _>(key)
    }
}

impl<C, K> From<ErrorData<C, K>> for AnyError<C, K>
where
    C: AbstractContext,
    K: Kind,
{
    fn from(data: ErrorData<C, K>) -> Self {
        Self(Box::new(data))
    }
}

impl<C, K> Display for AnyError<C, K>
where
    C: AbstractContext,
    K: Kind,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl<C, K> Error for AnyError<C, K>
where
    C: AbstractContext,
    K: Kind,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextDepth {
    All,
    Shallowest,
}

trait ErrorAndAny: Error + Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;

    fn as_any_boxed(self: Box<Self>) -> Box<dyn Any>;

    fn as_error_boxed(self: Box<Self>) -> Box<dyn Error + Send + Sync + 'static>;
}

impl<E: Error + Any + Send + Sync> ErrorAndAny for E {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_boxed(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn as_error_boxed(self: Box<Self>) -> Box<dyn Error + Send + Sync + 'static> {
        self
    }
}

/// The builder of [`AnyError`].
///
/// The only way to get an [`AnyErrorBuilder`] is calling the
/// [`AnyError::builder()`] function. [`AnyErrorBuilder`] can build almost all
/// kinds of [`AnyError`] except wrapping an external error, which can be done
/// with [`AnyError::wrap()`].
///
/// For some cases where you'd like to provide more context for your error, you
/// may refer to the following example:
///
/// ```rust
/// # use anyerr::AnyError as AnyErrorTemplate;
/// # use anyerr::kind::DefaultErrorKind;
/// # use anyerr::context::LiteralKeyStringMapContext;
/// type AnyError = AnyErrorTemplate<LiteralKeyStringMapContext, DefaultErrorKind>;
///
/// let inner = "-1".parse::<u32>().unwrap_err();
/// let source = AnyError::wrap(inner);
///
/// let err = AnyError::builder()
///     .kind(DefaultErrorKind::ValueValidation)
///     .message("could not parse `&str` to `u32`")
///     .context("string", "-1")
///     .context("target-type", String::from("u32"))
///     .context("expected", -1)
///     .source(source)
///     .build();
///
/// assert_eq!(err.kind(), DefaultErrorKind::ValueValidation);
/// assert_eq!(err.to_string(), "could not parse `&str` to `u32`");
/// ```
pub struct AnyErrorBuilder<C, K>(ErrorDataBuilder<C, K>)
where
    C: AbstractContext,
    K: Kind;

impl<C, K> AnyErrorBuilder<C, K>
where
    C: AbstractContext,
    K: Kind,
{
    fn new() -> Self {
        Self(ErrorDataBuilder::new())
    }

    /// Specifies the error kind of the resulting error.
    pub fn kind(self, kind: K) -> Self {
        Self(self.0.kind(kind))
    }

    /// Specifies the error message of the resulting error.
    pub fn message<S: Into<String>>(self, message: S) -> Self {
        Self(self.0.message(message))
    }

    /// Specifies the cause of the resulting error.
    pub fn source(self, source: AnyError<C, K>) -> Self {
        Self(self.0.source(source))
    }

    /// Returns the error with the provided data for each fields.
    pub fn build(self) -> AnyError<C, K> {
        AnyError::from(self.0.build(Backtrace::capture()))
    }
}

impl<C, K> AnyErrorBuilder<C, K>
where
    C: Context,
    K: Kind,
{
    /// Adds some context represented as a key-value pair to the resulting
    /// error.
    pub fn context<Q, R>(self, key: Q, value: R) -> Self
    where
        Q: Into<C::Key>,
        R: Convertable<C::Converter, C::Value>,
    {
        Self(self.0.context(key, value))
    }
}

#[cfg(test)]
mod tests {
    use std::num::ParseIntError;

    use crate::context::StringKeyStringMapContext;
    use crate::kind::DefaultErrorKind;

    use super::*;

    type DefaultAnyError = AnyError<StringKeyStringMapContext, DefaultErrorKind>;
    type DefaultErrorData = ErrorData<StringKeyStringMapContext, DefaultErrorKind>;

    #[test]
    fn any_error_builder_succeeds() {
        let inner = "-1".parse::<u32>().unwrap_err();
        let source = DefaultAnyError::wrap(inner);

        let err = DefaultAnyError::builder()
            .kind(DefaultErrorKind::ValueValidation)
            .message("could not parse `&str` to `u32`")
            .context("string", "-1")
            .context("target-type", String::from("u32"))
            .context("expected", -1)
            .source(source)
            .build();

        assert_eq!(err.kind(), DefaultErrorKind::ValueValidation);
        assert_eq!(err.to_string(), "could not parse `&str` to `u32`");
        assert_eq!(err.context(ContextDepth::All).count(), 3);
        assert!(err.source().is_some());
    }

    #[test]
    fn any_error_wrap_succeeds() {
        {
            let inner = "".parse::<u32>().unwrap_err();
            let err = DefaultAnyError::wrap(inner.clone());
            assert_eq!(err.to_string(), inner.to_string());
        }
        {
            let inner = DefaultAnyError::minimal("error");
            let err = DefaultAnyError::wrap(inner);
            assert!(err.source().is_none());
        }
    }

    #[test]
    fn any_error_downcast_succeeds() {
        {
            let mut err = DefaultAnyError::minimal("error");
            assert!(err.downcast_ref::<DefaultAnyError>().is_some());
            assert!(err.downcast_mut::<DefaultAnyError>().is_some());
            assert!(err.downcast::<DefaultAnyError>().is_ok());
        }
        {
            let source = DefaultAnyError::minimal("inner");
            let mut err = DefaultAnyError::from(DefaultErrorData::Layered {
                kind: DefaultErrorKind::Unknown,
                message: "error".into(),
                context: StringKeyStringMapContext::new(),
                source,
            });
            assert!(err.downcast_ref::<DefaultAnyError>().is_some());
            assert!(err.downcast_mut::<DefaultAnyError>().is_some());
            assert!(err.downcast::<DefaultAnyError>().is_ok());
        }
        {
            let inner = "".parse::<u32>().unwrap_err();
            let mut err = DefaultAnyError::wrap(inner);
            assert!(err.downcast_ref::<ParseIntError>().is_some());
            assert!(err.downcast_mut::<ParseIntError>().is_some());
            assert!(err.downcast::<ParseIntError>().is_ok());
        }
    }

    #[test]
    fn any_error_propagation_succeeds() {
        fn try_parse(val: &str) -> Result<u32, ParseIntError> {
            val.parse()
        }

        fn try_increment(val: &str) -> Result<u32, DefaultAnyError> {
            let val = try_parse(val).map_err(AnyError::wrap)?;
            Ok(val + 1)
        }

        assert_eq!(try_increment("1").unwrap(), 2);
        assert!(try_increment("").is_err());
    }
}
