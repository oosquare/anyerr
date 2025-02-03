use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

use super::base::{AnyError, ErrorExt};
use super::context::{ContextDepth, ContextEntry, ContextIter, ContextMap};

#[derive(Debug)]
pub(super) enum ErrorData {
    Simple {
        message: String,
        backtrace: Backtrace,
        context: ContextMap,
    },
    Layered {
        message: String,
        context: ContextMap,
        source: AnyError,
    },
    Wrapped {
        backtrace: Backtrace,
        inner: Box<dyn Error + Send + Sync + 'static>,
    },
}

impl Display for ErrorData {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Simple { message, .. } => write!(f, "{message}"),
            Self::Layered { message, .. } => write!(f, "{message}"),
            Self::Wrapped { inner, .. } => write!(f, "{inner}"),
        }
    }
}

impl Error for ErrorData {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Simple { .. } => None,
            Self::Layered { source, .. } => Some(source),
            Self::Wrapped { .. } => None,
        }
    }
}

impl ErrorExt for ErrorData {
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

pub(super) struct ErrorDataBuilder {
    message: String,
    context: Vec<ContextEntry>,
    source: Option<AnyError>,
}

impl ErrorDataBuilder {
    pub fn new() -> Self {
        Self {
            message: String::new(),
            context: Vec::new(),
            source: None,
        }
    }

    pub fn message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = message.into();
        self
    }

    pub fn context<V: Debug>(mut self, name: &str, value: &V) -> Self {
        self.context.push(ContextEntry::new(name, value));
        self
    }

    pub fn source(mut self, source: AnyError) -> Self {
        self.source = Some(source);
        self
    }

    pub fn build(self) -> ErrorData {
        match self.source {
            Some(source) => ErrorData::Layered {
                message: self.message,
                context: self.context.into(),
                source,
            },
            None => ErrorData::Simple {
                message: self.message,
                backtrace: Backtrace::capture(),
                context: self.context.into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_data_message_succeeds() {
        {
            let data = ErrorData::Simple {
                message: "simple".into(),
                backtrace: Backtrace::capture(),
                context: ContextMap::new(),
            };
            assert_eq!(data.message(), "simple");
            assert_eq!(data.to_string(), "simple");
        }
        {
            let data = ErrorData::Layered {
                message: "layered".into(),
                context: ContextMap::new(),
                source: AnyError::from(ErrorData::Simple {
                    message: "simple".into(),
                    backtrace: Backtrace::capture(),
                    context: ContextMap::new(),
                }),
            };
            assert_eq!(data.message(), "layered");
            assert_eq!(data.to_string(), "layered");
        }
        {
            let data = ErrorData::Wrapped {
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
            let data = ErrorData::Simple {
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
            let data = ErrorData::Layered {
                message: "layered".into(),
                context: ContextMap::from(vec![ContextEntry::new("name2", &2)]),
                source: AnyError::from(ErrorData::Simple {
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
            let data = ErrorData::Wrapped {
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
            let data = ErrorDataBuilder::new()
                .message("simple")
                .context("name", &1)
                .build();
            assert!(matches!(data, ErrorData::Simple { .. }));
        }
        {
            let data = ErrorDataBuilder::new()
                .message("layered")
                .context("name", &1)
                .source(AnyError::from(ErrorData::Simple {
                    message: "simple".into(),
                    backtrace: Backtrace::capture(),
                    context: ContextMap::from(vec![ContextEntry::new("name1", &1)]),
                }))
                .build();
            assert!(matches!(data, ErrorData::Layered { .. }));
        }
    }
}
