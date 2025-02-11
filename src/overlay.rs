pub mod error;
pub mod result;

/// The type that can wrap others of the same type recursively with additional
/// data attached.
///
/// A top-level error that contains another one, forming a multi-layer
/// structure, is resembled to an overlay over the inner error, which is where
/// the inspiration of the name [`Overlay`] comes from. Implemented for
/// error-related types, the [`Overlay`] allows add context information and
/// wrap errors conveniently.
///
/// For ergonomic consideration, the whole error wrapping procedure of adding
/// a new message and kind and attaching more context is split to multiple
/// steps in a builder-like fashion. The follow example demostrate how the
/// [`Overlay`] and other helper traits make error handling more easily.
///
/// You may also need to go through the [`Intermediate`] trait for advanced
/// usage.
///
/// # Example
///
/// ```rust
/// # use anyerr::{AnyError as AnyErrorTemplate, Intermediate, Overlay};
/// # use anyerr::kind::DefaultErrorKind;
/// # use anyerr::context::LiteralKeyStringMapContext;
/// type AnyError = AnyErrorTemplate<LiteralKeyStringMapContext, DefaultErrorKind>;
///
/// fn parse_text(text: &str) -> Result<u32, AnyError> {
///     text.parse::<u32>().map_err(AnyError::wrap)
/// }
///
/// fn try_increment(text: &str) -> Result<u32, AnyError> {
///     let x = parse_text(text)
///         .overlay("failed to parse the given text to `u32`")
///         .context("text", text)?;
///     Ok(x + 1)
/// }
///
/// assert_eq!(try_increment("0").unwrap(), 1);
/// assert!(try_increment("-1").is_err());
/// ```
pub trait Overlay: Sized {
    /// The type of the final result of the wrapping procedure.
    type Output: Overlay;

    /// The intermediate type used to add more context to the result.
    type Intermediate: Intermediate<Output = Self::Output>;

    /// Starts the wrapping procedure by adding some most essential data, such
    /// as the error messages and the error kind.
    ///
    /// Different types implementing [`Overlay`] may accept different sorts of
    /// values as this method's input, and these are determined by whether
    /// those values have the [`Applicable`] trait implemented. Refer to
    /// implementors of the [`Applicable`] trait for more information.
    fn overlay<V>(self, value: V) -> Self::Intermediate
    where
        V: Applicable<Self, Output = Self::Intermediate>,
    {
        value.apply(self)
    }
}

/// The intermediate type helps attach additional context to the resulting
/// overlay.
///
/// The implementors of the [`Intermediate`] are produced by the [`Overlay`]
/// trait, and is only used as a temporary builder to add some optional context
/// to the final output.
pub trait Intermediate: Sized {
    /// The type of the final result of the wrapping procedure.
    type Output: Overlay;

    /// Attachs additional context to the final output.
    ///
    /// Different types implementing [`Intermediate`] may accept different
    /// sorts of values as this method's input, and these are determined by
    /// whether those values have the [`Applicable`] trait implemented. Refer
    /// to implementors of the [`Applicable`] trait for more information.
    fn context<Q, R>(self, key: Q, value: R) -> Self
    where
        (Q, R): Applicable<Self, Output = Self>,
    {
        (key, value).apply(self)
    }

    /// Instantiates the output with all provided data.
    fn build(self) -> Self::Output;
}

/// The helper which determines whether a type can be applied to the target.
pub trait Applicable<T> {
    /// The type of the result produced by applying the value to the target.
    type Output;

    /// Makes the `target` do something with the `self`.
    fn apply(self, target: T) -> Self::Output;
}
