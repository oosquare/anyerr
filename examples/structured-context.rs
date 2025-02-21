mod err {
    use std::fmt::{Display, Formatter, Result as FmtResult};

    use anyerr::context::map::AnyMapContext;
    use anyerr::AnyError as AnyErrorTemplate;

    pub use anyerr::kind::NoErrorKind as ErrKind;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum ContextKey {
        ErrorCode,
        Timeout,
        Other(&'static str),
    }

    impl Display for ContextKey {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            match self {
                Self::ErrorCode => write!(f, "error-code"),
                Self::Timeout => write!(f, "timeout"),
                Self::Other(key) => write!(f, "{key}"),
            }
        }
    }

    type CustomKeyAnyMapContext = AnyMapContext<ContextKey, ContextKey>;

    pub type AnyError = AnyErrorTemplate<CustomKeyAnyMapContext, ErrKind>;
    pub type AnyResult<T> = Result<T, AnyError>;
}

use err::*;

fn fails() -> AnyResult<()> {
    let err = AnyError::builder()
        .message("an unknown error occurred")
        .context(ContextKey::ErrorCode, 42u32)
        .context(ContextKey::Timeout, false)
        .context(ContextKey::Other("function"), "fails()")
        .build();
    Err(err)
}

fn main() {
    let err = fails().unwrap_err();

    let error_code: &u32 = err.value_as(&ContextKey::ErrorCode).unwrap();
    let is_timeout: &bool = err.value_as(&ContextKey::Timeout).unwrap();
    let function_name: &&str = err.value_as(&ContextKey::Other("function")).unwrap();

    eprintln!("The error code is {error_code}");
    eprintln!("Whether the function failed due to timeout: {is_timeout}");
    eprintln!("The name of the failed function: {function_name}");
}
