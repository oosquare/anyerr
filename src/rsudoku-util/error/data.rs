use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

use super::base::{AnyError, ErrorExt};
use super::context::{ContextDepth, ContextEntry, ContextIter, ContextMap};
use super::kind::AnyErrorKind;

#[derive(Debug)]
pub(super) enum ErrorData<K>
where
    K: AnyErrorKind + 'static,
{
    Simple {
        kind: K,
        message: String,
        backtrace: Backtrace,
        context: ContextMap,
    },
    Layered {
        kind: K,
        message: String,
        context: ContextMap,
        source: AnyError<K>,
    },
    Wrapped {
        backtrace: Backtrace,
        inner: Box<dyn Error + Send + Sync + 'static>,
    },
}

impl<K> Display for ErrorData<K>
where
    K: AnyErrorKind + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Simple { message, .. } => write!(f, "{message}"),
            Self::Layered { message, .. } => write!(f, "{message}"),
            Self::Wrapped { inner, .. } => write!(f, "{inner}"),
        }
    }
}

impl<K> Error for ErrorData<K>
where
    K: AnyErrorKind + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Simple { .. } => None,
            Self::Layered { source, .. } => Some(source),
            Self::Wrapped { .. } => None,
        }
    }
}

impl<K> ErrorExt for ErrorData<K>
where
    K: AnyErrorKind + 'static,
{
    type ErrorKind = K;

    fn kind(&self) -> Self::ErrorKind {
        match self {
            Self::Simple { kind, .. } => *kind,
            Self::Layered { kind, .. } => *kind,
            Self::Wrapped { .. } => K::RAW_KIND,
        }
    }
    fn message(&self) -> String {
        match self {
            Self::Simple { message, .. } => message.into(),
            Self::Layered { message, .. } => message.into(),
            Self::Wrapped { inner, .. } => inner.to_string(),
        }
    }

    fn backtrace(&self) -> &Backtrace {
        match self {
            Self::Simple { backtrace, .. } => backtrace,
            Self::Layered { source, .. } => source.backtrace(),
            Self::Wrapped { backtrace, .. } => backtrace,
        }
    }

    fn context(&self, depth: ContextDepth) -> ContextIter {
        match self {
            Self::Simple { context, .. } => context.iter(),
            Self::Layered {
                context, source, ..
            } => match depth {
                ContextDepth::All => source.context(depth).concat(context),
                ContextDepth::Shallowest => context.iter(),
            },
            Self::Wrapped { .. } => ContextIter::new(),
        }
    }
}

pub(super) struct ErrorDataBuilder<K>
where
    K: AnyErrorKind + 'static,
{
    kind: K,
    message: String,
    context: Vec<ContextEntry>,
    source: Option<AnyError<K>>,
}

impl<K> ErrorDataBuilder<K>
where
    K: AnyErrorKind,
{
    pub fn new() -> Self {
        Self {
            kind: K::default(),
            message: String::new(),
            context: Vec::new(),
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

    pub fn context<V: Debug>(mut self, name: &str, value: &V) -> Self {
        self.context.push(ContextEntry::new(name, value));
        self
    }

    pub fn source(mut self, source: AnyError<K>) -> Self {
        self.source = Some(source);
        self
    }

    pub fn build(self, backtrace: Backtrace) -> ErrorData<K> {
        match self.source {
            Some(source) => ErrorData::Layered {
                kind: self.kind,
                message: self.message,
                context: self.context.into(),
                source,
            },
            None => ErrorData::Simple {
                kind: self.kind,
                message: self.message,
                backtrace,
                context: self.context.into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::kind::DefaultAnyErrorKind;

    use super::*;

    type DefaultErrorData = ErrorData<DefaultAnyErrorKind>;
    type DefaultErrorDataBuilder = ErrorDataBuilder<DefaultAnyErrorKind>;

    #[test]
    fn error_data_message_succeeds() {
        {
            let data = DefaultErrorData::Simple {
                kind: DefaultAnyErrorKind::Unknown,
                message: "simple".into(),
                backtrace: Backtrace::capture(),
                context: ContextMap::new(),
            };
            assert_eq!(data.message(), "simple");
            assert_eq!(data.to_string(), "simple");
        }
        {
            let data = DefaultErrorData::Layered {
                kind: DefaultAnyErrorKind::Unknown,
                message: "layered".into(),
                context: ContextMap::new(),
                source: AnyError::from(DefaultErrorData::Simple {
                    kind: DefaultAnyErrorKind::Unknown,
                    message: "simple".into(),
                    backtrace: Backtrace::capture(),
                    context: ContextMap::new(),
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
                context: ContextMap::from(vec![ContextEntry::new("name", &1)]),
            };

            let mut iter = data.context(ContextDepth::All);
            assert_eq!(iter.next(), Some(&ContextEntry::new("name", &1)));
            assert_eq!(iter.next(), None);

            let mut iter = data.context(ContextDepth::Shallowest);
            assert_eq!(iter.next(), Some(&ContextEntry::new("name", &1)));
            assert_eq!(iter.next(), None);
        }
        {
            let data = DefaultErrorData::Layered {
                kind: DefaultAnyErrorKind::Unknown,
                message: "layered".into(),
                context: ContextMap::from(vec![ContextEntry::new("name2", &2)]),
                source: AnyError::from(DefaultErrorData::Simple {
                    kind: DefaultAnyErrorKind::Unknown,
                    message: "simple".into(),
                    backtrace: Backtrace::capture(),
                    context: ContextMap::from(vec![ContextEntry::new("name1", &1)]),
                }),
            };

            let mut iter = data.context(ContextDepth::All);
            assert_eq!(iter.next(), Some(&ContextEntry::new("name2", &2)));
            assert_eq!(iter.next(), Some(&ContextEntry::new("name1", &1)));
            assert_eq!(iter.next(), None);

            let mut iter = data.context(ContextDepth::Shallowest);
            assert_eq!(iter.next(), Some(&ContextEntry::new("name2", &2)));
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
                .context("name", &1)
                .build(Backtrace::capture());
            assert!(matches!(data, ErrorData::Simple { .. }));
            assert_eq!(data.kind(), DefaultAnyErrorKind::ValueValidation);
        }
        {
            let data = DefaultErrorDataBuilder::new()
                .message("layered")
                .context("name", &1)
                .source(AnyError::from(DefaultErrorData::Simple {
                    kind: DefaultAnyErrorKind::Unknown,
                    message: "simple".into(),
                    backtrace: Backtrace::capture(),
                    context: ContextMap::from(vec![ContextEntry::new("name1", &1)]),
                }))
                .build(Backtrace::capture());
            assert_eq!(data.kind(), DefaultAnyErrorKind::default());
            assert!(matches!(data, ErrorData::Layered { .. }));
        }
    }
}
