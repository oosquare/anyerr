use std::any::Any;
use std::backtrace::Backtrace;
use std::borrow::Borrow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::Hash;

use crate::error::context::{AbstractContext, Context, Entry, Iter};
use crate::error::converter::Convertable;
use crate::error::core::{AnyError, ContextDepth};
use crate::error::kind::Kind;

#[derive(Debug)]
pub enum ErrorData<C, K>
where
    C: AbstractContext + 'static,
    K: Kind + 'static,
{
    Simple {
        kind: K,
        message: String,
        backtrace: Backtrace,
        context: C,
    },
    Layered {
        kind: K,
        message: String,
        context: C,
        source: AnyError<C, K>,
    },
    Wrapped {
        backtrace: Backtrace,
        inner: Box<dyn Error + Send + Sync + 'static>,
    },
}

impl<C, K> ErrorData<C, K>
where
    C: AbstractContext + 'static,
    K: Kind + 'static,
{
    pub fn kind(&self) -> K {
        match self {
            Self::Simple { kind, .. } => *kind,
            Self::Layered { kind, .. } => *kind,
            Self::Wrapped { .. } => K::RAW_KIND,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Simple { message, .. } => message.into(),
            Self::Layered { message, .. } => message.into(),
            Self::Wrapped { inner, .. } => inner.to_string(),
        }
    }

    pub fn backtrace(&self) -> &Backtrace {
        match self {
            Self::Simple { backtrace, .. } => backtrace,
            Self::Layered { source, .. } => source.backtrace(),
            Self::Wrapped { backtrace, .. } => backtrace,
        }
    }

    pub fn context(&self, depth: ContextDepth) -> C::Iter<'_> {
        match self {
            Self::Simple { context, .. } => context.iter(),
            Self::Layered {
                context, source, ..
            } => match depth {
                ContextDepth::All => context.iter().compose(source.context(depth)),
                ContextDepth::Shallowest => context.iter(),
            },
            Self::Wrapped { .. } => C::Iter::default(),
        }
    }
}

impl<C, K> ErrorData<C, K>
where
    C: crate::error::context::SingletonContext + 'static,
    K: Kind + 'static,
{
    pub fn value(&self) -> Option<&<C::Entry as Entry>::ValueBorrowed> {
        match self {
            Self::Simple { context, .. } => context.value(),
            Self::Layered { context, .. } => context.value(),
            Self::Wrapped { .. } => None,
        }
    }
}

impl<C, K> ErrorData<C, K>
where
    C: crate::error::context::StringContext + 'static,
    K: Kind + 'static,
{
    pub fn get<Q>(&self, key: &Q) -> Option<&<C::Entry as Entry>::ValueBorrowed>
    where
        <C::Entry as Entry>::KeyBorrowed: Borrow<Q>,
        Q: Debug + Eq + Hash + ?Sized,
    {
        match self {
            Self::Simple { context, .. } => context.get(key),
            Self::Layered { context, .. } => context.get(key),
            Self::Wrapped { .. } => None,
        }
    }
}

impl<C, K> ErrorData<C, K>
where
    C: crate::error::context::AnyContext + 'static,
    K: Kind + 'static,
{
    pub fn value_as<T, Q>(&self, key: &Q) -> Option<&T>
    where
        <C::Entry as Entry>::KeyBorrowed: Borrow<Q>,
        Q: Debug + Eq + Hash + ?Sized,
        T: Any,
    {
        match self {
            Self::Simple { context, .. } => context.value_as::<T, _>(key),
            Self::Layered { context, .. } => context.value_as::<T, _>(key),
            Self::Wrapped { .. } => None,
        }
    }
}

impl<C, K> Display for ErrorData<C, K>
where
    C: AbstractContext + 'static,
    K: Kind + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Simple { message, .. } => write!(f, "{message}"),
            Self::Layered { message, .. } => write!(f, "{message}"),
            Self::Wrapped { inner, .. } => write!(f, "{inner}"),
        }
    }
}

impl<C, K> Error for ErrorData<C, K>
where
    C: AbstractContext + 'static,
    K: Kind + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Simple { .. } => None,
            Self::Layered { source, .. } => Some(source),
            Self::Wrapped { .. } => None,
        }
    }
}

pub struct ErrorDataBuilder<C, K>
where
    C: AbstractContext + 'static,
    K: Kind + 'static,
{
    kind: K,
    message: String,
    context: C,
    source: Option<AnyError<C, K>>,
}

impl<C, K> ErrorDataBuilder<C, K>
where
    C: AbstractContext + 'static,
    K: Kind + 'static,
{
    pub fn new() -> Self {
        Self {
            kind: K::default(),
            message: String::new(),
            context: C::default(),
            source: None,
        }
    }

    pub fn kind(mut self, kind: K) -> Self {
        self.kind = kind;
        self
    }

    pub fn message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = message.into();
        self
    }

    pub fn source(mut self, source: AnyError<C, K>) -> Self {
        self.source = Some(source);
        self
    }

    pub fn build(self, backtrace: Backtrace) -> ErrorData<C, K> {
        match self.source {
            Some(source) => ErrorData::Layered {
                kind: self.kind,
                message: self.message,
                context: self.context,
                source,
            },
            None => ErrorData::Simple {
                kind: self.kind,
                message: self.message,
                backtrace,
                context: self.context,
            },
        }
    }
}

impl<C, K> ErrorDataBuilder<C, K>
where
    C: Context + 'static,
    K: Kind + 'static,
{
    pub fn context<Q, R>(mut self, key: Q, value: R) -> Self
    where
        Q: Into<C::Key>,
        R: Convertable<C::Converter, C::Value>,
    {
        self.context.insert_with::<C::Converter, _, _>(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::error::context::map::LiteralKeyStringMapEntry;
    use crate::error::context::{Entry, LiteralKeyStringMapContext};
    use crate::error::kind::DefaultAnyErrorKind;

    use super::*;

    type DefaultErrorData = ErrorData<LiteralKeyStringMapContext, DefaultAnyErrorKind>;
    type DefaultErrorDataBuilder =
        ErrorDataBuilder<LiteralKeyStringMapContext, DefaultAnyErrorKind>;

    #[test]
    fn error_data_message_succeeds() {
        {
            let data = DefaultErrorData::Simple {
                kind: DefaultAnyErrorKind::Unknown,
                message: "simple".into(),
                backtrace: Backtrace::capture(),
                context: LiteralKeyStringMapContext::new(),
            };
            assert_eq!(data.message(), "simple");
            assert_eq!(data.to_string(), "simple");
        }
        {
            let data = DefaultErrorData::Layered {
                kind: DefaultAnyErrorKind::Unknown,
                message: "layered".into(),
                context: LiteralKeyStringMapContext::new(),
                source: AnyError::from(DefaultErrorData::Simple {
                    kind: DefaultAnyErrorKind::Unknown,
                    message: "simple".into(),
                    backtrace: Backtrace::capture(),
                    context: LiteralKeyStringMapContext::new(),
                }),
            };
            assert_eq!(data.message(), "layered");
            assert_eq!(data.to_string(), "layered");
        }
        {
            let data = DefaultErrorData::Wrapped {
                backtrace: Backtrace::capture(),
                inner: "wrapped".into(),
            };
            assert_eq!(data.message(), "wrapped");
            assert_eq!(data.to_string(), "wrapped");
        }
    }

    #[test]
    fn error_data_context_succeeds() {
        {
            let data = DefaultErrorData::Simple {
                kind: DefaultAnyErrorKind::Unknown,
                message: "simple".into(),
                backtrace: Backtrace::capture(),
                context: LiteralKeyStringMapContext::from(vec![("key", "1")]),
            };

            let mut iter = data.context(ContextDepth::All);
            assert_eq!(
                iter.next(),
                Some(&LiteralKeyStringMapEntry::new("key", "1"))
            );
            assert_eq!(iter.next(), None);

            let mut iter = data.context(ContextDepth::Shallowest);
            assert_eq!(
                iter.next(),
                Some(&LiteralKeyStringMapEntry::new("key", "1"))
            );
            assert_eq!(iter.next(), None);
        }
        {
            let data = DefaultErrorData::Layered {
                kind: DefaultAnyErrorKind::Unknown,
                message: "layered".into(),
                context: LiteralKeyStringMapContext::from(vec![("key2", "2")]),
                source: AnyError::from(DefaultErrorData::Simple {
                    kind: DefaultAnyErrorKind::Unknown,
                    message: "simple".into(),
                    backtrace: Backtrace::capture(),
                    context: LiteralKeyStringMapContext::from(vec![("key1", "1")]),
                }),
            };

            let mut iter = data.context(ContextDepth::All);
            assert_eq!(
                iter.next(),
                Some(&LiteralKeyStringMapEntry::new("key2", "2"))
            );
            assert_eq!(
                iter.next(),
                Some(&LiteralKeyStringMapEntry::new("key1", "1"))
            );
            assert_eq!(iter.next(), None);

            let mut iter = data.context(ContextDepth::Shallowest);
            assert_eq!(
                iter.next(),
                Some(&LiteralKeyStringMapEntry::new("key2", "2"))
            );
            assert_eq!(iter.next(), None);
        }
        {
            let data = DefaultErrorData::Wrapped {
                backtrace: Backtrace::capture(),
                inner: "wrapped".into(),
            };

            let mut iter = data.context(ContextDepth::All);
            assert_eq!(iter.next(), None);

            let mut iter = data.context(ContextDepth::Shallowest);
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn error_data_builder_build() {
        {
            let data = DefaultErrorDataBuilder::new()
                .kind(DefaultAnyErrorKind::ValueValidation)
                .message("simple")
                .context("key", "1")
                .build(Backtrace::capture());
            assert!(matches!(data, ErrorData::Simple { .. }));
            assert_eq!(data.kind(), DefaultAnyErrorKind::ValueValidation);
        }
        {
            let data = DefaultErrorDataBuilder::new()
                .message("layered")
                .context("key", "1")
                .source(AnyError::from(DefaultErrorData::Simple {
                    kind: DefaultAnyErrorKind::Unknown,
                    message: "simple".into(),
                    backtrace: Backtrace::capture(),
                    context: LiteralKeyStringMapContext::from(vec![("key1", "1")]),
                }))
                .build(Backtrace::capture());
            assert_eq!(data.kind(), DefaultAnyErrorKind::default());
            assert!(matches!(data, ErrorData::Layered { .. }));
        }
    }
}
