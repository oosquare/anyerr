use crate::error::context::{AbstractContext, Context};
use crate::error::converter::Convertable;
use crate::error::core::{AnyError, AnyErrorBuilder};
use crate::error::kind::Kind;
use crate::error::overlay::{Applicable, Intermediate, Overlay};

impl<C, K> Overlay for AnyError<C, K>
where
    C: AbstractContext,
    K: Kind,
{
    type Output = Self;

    type Intermediate = IntermediateAnyError<C, K>;
}

pub struct IntermediateAnyError<C, K>
where
    C: AbstractContext,
    K: Kind,
{
    builder: AnyErrorBuilder<C, K>,
}

impl<C, K> From<AnyErrorBuilder<C, K>> for IntermediateAnyError<C, K>
where
    C: AbstractContext,
    K: Kind,
{
    fn from(builder: AnyErrorBuilder<C, K>) -> Self {
        Self { builder }
    }
}

impl<C, K> From<IntermediateAnyError<C, K>> for AnyError<C, K>
where
    C: AbstractContext,
    K: Kind,
{
    fn from(value: IntermediateAnyError<C, K>) -> Self {
        value.builder.build()
    }
}

impl<C, K> Intermediate for IntermediateAnyError<C, K>
where
    C: AbstractContext,
    K: Kind,
{
    type Output = AnyError<C, K>;

    fn build(self) -> Self::Output {
        self.into()
    }
}

impl<C, K> Applicable<AnyError<C, K>> for String
where
    C: AbstractContext,
    K: Kind,
{
    type Output = IntermediateAnyError<C, K>;

    fn apply(self, target: AnyError<C, K>) -> Self::Output {
        AnyError::builder().message(self).source(target).into()
    }
}

impl<C, K> Applicable<AnyError<C, K>> for &str
where
    C: AbstractContext,
    K: Kind,
{
    type Output = IntermediateAnyError<C, K>;

    fn apply(self, target: AnyError<C, K>) -> Self::Output {
        AnyError::builder().message(self).source(target).into()
    }
}

impl<C, K, S> Applicable<AnyError<C, K>> for (S, K)
where
    C: AbstractContext,
    K: Kind,
    S: Into<String>,
{
    type Output = IntermediateAnyError<C, K>;

    fn apply(self, target: AnyError<C, K>) -> Self::Output {
        AnyError::builder()
            .message(self.0)
            .kind(self.1)
            .source(target)
            .into()
    }
}

impl<C, K, Q, R> Applicable<IntermediateAnyError<C, K>> for (Q, R)
where
    C: Context,
    K: Kind,
    Q: Into<C::Key>,
    R: Convertable<C::Converter, C::Value>,
{
    type Output = IntermediateAnyError<C, K>;

    fn apply(self, target: IntermediateAnyError<C, K>) -> Self::Output {
        target.builder.context(self.0, self.1).into()
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::error::context::LiteralKeyStringMapContext;
    use crate::error::kind::DefaultAnyErrorKind;

    use super::*;

    type ErrKind = DefaultAnyErrorKind;
    type DefaultAnyError = AnyError<LiteralKeyStringMapContext, ErrKind>;

    #[test]
    fn any_error_overlay_succeeds_when_message_is_given() {
        let source = DefaultAnyError::minimal("source error");
        let err = source.overlay(String::from("wrapper error")).build();
        assert_eq!(err.to_string(), "wrapper error");
        assert_eq!(err.source().unwrap().to_string(), "source error");

        let source = DefaultAnyError::minimal("source error");
        let err = source.overlay("wrapper error").build();
        assert_eq!(err.to_string(), "wrapper error");
        assert_eq!(err.source().unwrap().to_string(), "source error");
    }

    #[test]
    fn any_error_overlay_succeeds_when_message_and_kind_are_given() {
        let source = DefaultAnyError::minimal("source error");
        let err = source
            .overlay(("wrapper error", ErrKind::ValueValidation))
            .build();
        assert_eq!(err.to_string(), "wrapper error");
        assert_eq!(err.kind(), ErrKind::ValueValidation);
        assert_eq!(err.source().unwrap().to_string(), "source error");
    }

    #[test]
    fn intermediate_any_error_context_succeeds() {
        let source = DefaultAnyError::minimal("source error");
        let err = source
            .overlay("wrapper error")
            .context("key", "value")
            .build();
        assert_eq!(err.to_string(), "wrapper error");
        assert_eq!(err.get("key"), Some("\"value\""));
    }

    #[test]
    fn intermediate_any_error_into_any_error_succeeds_with_try_operator() {
        fn source_error_func() -> Result<(), DefaultAnyError> {
            Err(AnyError::minimal("source error"))
        }
        fn wrapper_error_func() -> Result<(), DefaultAnyError> {
            source_error_func().map_err(|err| err.overlay("wrapper error"))?;
            Ok(())
        }
        let err = wrapper_error_func().unwrap_err();
        assert_eq!(err.to_string(), "wrapper error");
    }
}
