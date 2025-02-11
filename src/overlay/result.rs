use crate::context::{AbstractContext, Context};
use crate::converter::Convertable;
use crate::core::AnyError;
use crate::kind::Kind;
use crate::overlay::error::IntermediateAnyError;
use crate::overlay::{Applicable, Intermediate, Overlay};

impl<T, C, K> Overlay for Result<T, AnyError<C, K>>
where
    C: AbstractContext,
    K: Kind,
{
    type Output = Result<T, AnyError<C, K>>;

    type Intermediate = Result<T, IntermediateAnyError<C, K>>;
}

impl<T, C, K> Intermediate for Result<T, IntermediateAnyError<C, K>>
where
    C: AbstractContext,
    K: Kind,
{
    type Output = Result<T, AnyError<C, K>>;

    fn build(self) -> Self::Output {
        self.map_err(Into::into)
    }
}

impl<T, C, K> Applicable<Result<T, AnyError<C, K>>> for String
where
    C: AbstractContext,
    K: Kind,
{
    type Output = Result<T, IntermediateAnyError<C, K>>;

    /// Delegates the parameters to [`AnyError`]'s implementation.
    fn apply(self, target: Result<T, AnyError<C, K>>) -> Self::Output {
        target.map_err(|err| err.overlay(self))
    }
}

impl<T, C, K> Applicable<Result<T, AnyError<C, K>>> for &str
where
    C: AbstractContext,
    K: Kind,
{
    type Output = Result<T, IntermediateAnyError<C, K>>;

    /// Delegates the parameters to [`AnyError`]'s implementation.
    fn apply(self, target: Result<T, AnyError<C, K>>) -> Self::Output {
        target.map_err(|err| err.overlay(self))
    }
}

impl<T, C, K, S> Applicable<Result<T, AnyError<C, K>>> for (S, K)
where
    C: AbstractContext,
    K: Kind,
    S: Into<String>,
{
    type Output = Result<T, IntermediateAnyError<C, K>>;

    /// Delegates the parameters to [`AnyError`]'s implementation.
    fn apply(self, target: Result<T, AnyError<C, K>>) -> Self::Output {
        target.map_err(|err| err.overlay(self))
    }
}

impl<T, C, K, Q, R> Applicable<Result<T, IntermediateAnyError<C, K>>> for (Q, R)
where
    C: Context,
    K: Kind,
    Q: Into<C::Key>,
    R: Convertable<C::Converter, C::Value>,
{
    type Output = Result<T, IntermediateAnyError<C, K>>;

    /// Delegates the parameters to [`IntermediateAnyError`]'s implementation.
    fn apply(self, target: Result<T, IntermediateAnyError<C, K>>) -> Self::Output {
        target.map_err(|err| err.context(self.0, self.1))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::context::LiteralKeyStringMapContext;
    use crate::kind::DefaultErrorKind;

    use super::*;

    type ErrKind = DefaultErrorKind;
    type DefaultAnyError = AnyError<LiteralKeyStringMapContext, ErrKind>;

    #[test]
    fn result_overlay_succeeds_when_message_is_given() {
        let source = Err::<(), _>(DefaultAnyError::minimal("source error"));
        let res = source.overlay(String::from("wrapper error")).build();
        let err = res.unwrap_err();
        assert_eq!(err.to_string(), "wrapper error");
        assert_eq!(err.source().unwrap().to_string(), "source error");

        let source = Err::<(), _>(DefaultAnyError::minimal("source error"));
        let res = source.overlay("wrapper error").build();
        let err = res.unwrap_err();
        assert_eq!(err.to_string(), "wrapper error");
        assert_eq!(err.source().unwrap().to_string(), "source error");
    }

    #[test]
    fn result_overlay_succeeds_when_message_and_kind_are_given() {
        let source = Err::<(), _>(DefaultAnyError::minimal("source error"));
        let res = source
            .overlay(("wrapper error", ErrKind::ValueValidation))
            .build();
        let err = res.unwrap_err();
        assert_eq!(err.to_string(), "wrapper error");
        assert_eq!(err.kind(), ErrKind::ValueValidation);
        assert_eq!(err.source().unwrap().to_string(), "source error");
    }

    #[test]
    fn result_overlay_succeeds_when_result_is_ok() {
        let source = Ok::<i32, DefaultAnyError>(1);
        let res = source.overlay("no error").build();
        assert_eq!(res.unwrap(), 1);
    }

    #[test]
    fn intermediate_result_context_succeeds() {
        let source = Err::<(), _>(DefaultAnyError::minimal("source error"));
        let res = source
            .overlay(String::from("wrapper error"))
            .context("i32", 1)
            .context("&str", "value")
            .build();
        let err = res.unwrap_err();
        assert_eq!(err.to_string(), "wrapper error");
        assert_eq!(err.get("i32"), Some("1"));
        assert_eq!(err.get("&str"), Some("\"value\""));
    }
}
