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
    pub fn minimal<S: Into<String>>(message: S) -> Self {
        Self::from(ErrorData::<C, K>::Simple {
            kind: K::default(),
            message: message.into(),
            backtrace: Backtrace::capture(),
            context: C::default(),
        })
    }

    pub fn quick<S: Into<String>>(message: S, kind: K) -> Self {
        Self::from(ErrorData::<C, K>::Simple {
            kind,
            message: message.into(),
            backtrace: Backtrace::capture(),
            context: C::default(),
        })
    }

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

    pub fn builder() -> AnyErrorBuilder<C, K> {
        AnyErrorBuilder::new()
    }

    pub fn kind(&self) -> K {
        self.0.kind()
    }

    pub fn message(&self) -> String {
        self.0.message()
    }

    pub fn backtrace(&self) -> &Backtrace {
        self.0.backtrace()
    }

    pub fn context(&self, depth: ContextDepth) -> C::Iter<'_> {
        self.0.context(depth)
    }

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

    pub fn downcast<E>(self) -> Result<E, Self>
    where
        E: Error + Send + Sync + 'static,
    {
        match *self.0 {
            ErrorData::Simple { .. } | ErrorData::Layered { .. } => {
                if self.is::<E>() {
                    let boxed: Box<dyn Any> = Box::new(self);
                    let err = boxed
                        .downcast::<E>()
                        .map(|res| *res)
                        .expect("`boxed` should be `Box<E>` (i.e. `Box<Self>`)");
                    Ok(err)
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
    pub fn value(&self) -> Option<&<C::Entry as Entry>::ValueBorrowed> {
        self.0.value()
    }
}

impl<C, K> AnyError<C, K>
where
    C: crate::context::StringContext,
    K: Kind,
{
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

    pub fn kind(self, kind: K) -> Self {
        Self(self.0.kind(kind))
    }

    pub fn message<S: Into<String>>(self, message: S) -> Self {
        Self(self.0.message(message))
    }

    pub fn source(self, source: AnyError<C, K>) -> Self {
        Self(self.0.source(source))
    }

    pub fn build(self) -> AnyError<C, K> {
        AnyError::from(self.0.build(Backtrace::capture()))
    }
}

impl<C, K> AnyErrorBuilder<C, K>
where
    C: Context,
    K: Kind,
{
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
    use crate::kind::DefaultAnyErrorKind;

    use super::*;

    type DefaultAnyError = AnyError<StringKeyStringMapContext, DefaultAnyErrorKind>;
    type DefaultErrorData = ErrorData<StringKeyStringMapContext, DefaultAnyErrorKind>;

    #[test]
    fn any_error_builder_succeeds() {
        let inner = "-1".parse::<u32>().unwrap_err();
        let source = DefaultAnyError::wrap(inner);

        let err = DefaultAnyError::builder()
            .kind(DefaultAnyErrorKind::ValueValidation)
            .message("could not parse `&str` to `u32`")
            .context("string", "-1")
            .context("target-type", String::from("u32"))
            .context("expected", -1)
            .source(source)
            .build();

        assert_eq!(err.kind(), DefaultAnyErrorKind::ValueValidation);
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
                kind: DefaultAnyErrorKind::Unknown,
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
