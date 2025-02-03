use std::any::{Any, TypeId};
use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use super::context::{ContextDepth, ContextIter};
use super::data::ErrorData;
use super::ContextMap;

pub trait ErrorExt: Error {
    fn message(&self) -> String;

    fn backtrace(&self) -> &Backtrace;

    fn context(&self, depth: ContextDepth) -> ContextIter;
}

#[derive(Debug)]
pub struct AnyError {
    data: Box<ErrorData>,
}

impl AnyError {
    pub fn wrap<E>(err: E) -> Self
    where
        E: Error + Any + Send + Sync,
    {
        let err: Box<dyn ErrorAndAny> = Box::new(err);
        if err.as_any().is::<AnyError>() {
            *err.as_any_boxed()
                .downcast::<AnyError>()
                .expect("`err` should be `Box<AnyError>`")
        } else {
            let data = ErrorData::Wrapped {
                backtrace: Backtrace::capture(),
                inner: err.as_error_boxed(),
            };
            Self::from(data)
        }
    }

    pub fn is<E>(&self) -> bool
    where
        E: Error + Send + Sync + 'static,
    {
        match &*self.data {
            ErrorData::Simple { .. } => TypeId::of::<E>() == TypeId::of::<Self>(),
            ErrorData::Layered { .. } => TypeId::of::<E>() == TypeId::of::<Self>(),
            ErrorData::Wrapped { inner, .. } => inner.is::<E>(),
        }
    }

    pub fn downcast<E>(self) -> Result<E, Self>
    where
        E: Error + Send + Sync + 'static,
    {
        match *self.data {
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
        match &*self.data {
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
        let use_inner = matches!(&*self.data, ErrorData::Wrapped { .. });
        if !use_inner {
            let any: &mut dyn Any = self;
            any.downcast_mut::<E>()
        } else {
            let ErrorData::Wrapped { inner, .. } = &mut *self.data else {
                unreachable!("`self.data` matches `ErrorData::Wrapped {{ .. }}`");
            };
            inner.downcast_mut::<E>()
        }
    }
}

impl<S: Into<String>> From<S> for AnyError {
    fn from(message: S) -> Self {
        Self::from(ErrorData::Simple {
            message: message.into(),
            backtrace: Backtrace::capture(),
            context: ContextMap::new(),
        })
    }
}

impl From<ErrorData> for AnyError {
    fn from(data: ErrorData) -> Self {
        Self {
            data: Box::new(data),
        }
    }
}

impl Display for AnyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.data.fmt(f)
    }
}

impl Error for AnyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.data.source()
    }
}

impl ErrorExt for AnyError {
    fn message(&self) -> String {
        self.data.message()
    }

    fn backtrace(&self) -> &Backtrace {
        self.data.backtrace()
    }

    fn context(&self, depth: ContextDepth) -> ContextIter {
        self.data.context(depth)
    }
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

#[cfg(test)]
mod tests {
    use std::num::ParseIntError;

    use super::*;

    #[test]
    fn any_error_from_string_succeeds() {
        let err = AnyError::from("error");
        assert_eq!(err.to_string(), "error");
    }

    #[test]
    fn any_error_wrap_succeeds() {
        {
            let inner = "".parse::<u32>().unwrap_err();
            let err = AnyError::wrap(inner.clone());
            assert_eq!(err.to_string(), inner.to_string());
        }
        {
            let inner = AnyError::from("error");
            let err = AnyError::wrap(inner);
            assert!(err.source().is_none());
        }
    }

    #[test]
    fn any_error_downcast_succeeds() {
        {
            let mut err = AnyError::from("error");
            assert!(err.downcast_ref::<AnyError>().is_some());
            assert!(err.downcast_mut::<AnyError>().is_some());
            assert!(err.downcast::<AnyError>().is_ok());
        }
        {
            let source = AnyError::from("inner");
            let mut err = AnyError::from(ErrorData::Layered {
                message: "error".into(),
                context: ContextMap::new(),
                source,
            });
            assert!(err.downcast_ref::<AnyError>().is_some());
            assert!(err.downcast_mut::<AnyError>().is_some());
            assert!(err.downcast::<AnyError>().is_ok());
        }
        {
            let inner = "".parse::<u32>().unwrap_err();
            let mut err = AnyError::wrap(inner);
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

        fn try_increment(val: &str) -> Result<u32, AnyError> {
            let val = try_parse(val).map_err(AnyError::wrap)?;
            Ok(val + 1)
        }

        assert_eq!(try_increment("1").unwrap(), 2);
        assert!(try_increment("").is_err());
    }
}
