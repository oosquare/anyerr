use std::fmt::Debug;

use crate::context::AnyValue;

/// The trait used to select the method to transforming values.
///
/// Implementers of this trait define how values are mapped to other types. For
/// example, the [`DebugConverter`] implements a conversion using the
/// [`Debug::fmt()`] method to format values as strings. This is a trait that
/// doesn't actually perform the conversion itself, but specifies how values
/// should be transformed.
///
/// For more converters, refer to the [`crate::converter`] module. To learn
/// about the underlying type that works with the conversion, see the
/// [`Convertable`] trait.
pub trait Converter: Debug + Send + Sync + 'static {}

/// A converter that uses the [`Debug`] trait to format values
#[derive(Debug)]
pub struct DebugConverter;

impl Converter for DebugConverter {}

/// A converter that uses the [`Into`] trait to convert values into another
/// type.
#[derive(Debug)]
pub struct IntoConverter;

impl Converter for IntoConverter {}

/// A converter that converts values into a type-erased [`Box`] containing any
/// value.
#[derive(Debug)]
pub struct BoxConverter;

impl Converter for BoxConverter {}

/// The trait marking a type that is able to be converted to another one using
/// a [`Converter`].
///
/// The [`Convertable`] trait allows types to be converted into another type `T`
/// using a specified converter `C`.
pub trait Convertable<C: Converter, T>: Sized {
    /// Converts the value into another type `T` using the specified converter
    /// `C`. The conversion should not fail, so not a [`Result<T, E>`] but a
    /// `T` is expected.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use anyerr::converter::{Convertable, DebugConverter};
    /// assert_eq!(<_ as Convertable<DebugConverter, String>>::to(42), "42");
    /// assert_eq!(<_ as Convertable<DebugConverter, String>>::to("str"), "\"str\"");
    /// ```
    fn to(self) -> T;
}

impl<S: Debug, T: From<String>> Convertable<DebugConverter, T> for S {
    fn to(self) -> T {
        format!("{self:?}").into()
    }
}

impl<S: Into<T>, T> Convertable<IntoConverter, T> for S {
    fn to(self) -> T {
        self.into()
    }
}

impl<S, T> Convertable<BoxConverter, T> for S
where
    S: AnyValue,
    T: From<Box<dyn AnyValue + Send + Sync + 'static>>,
{
    fn to(self) -> T {
        let res: Box<dyn AnyValue + Send + Sync + 'static> = Box::new(self);
        res.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type BoxAnyValue = Box<dyn AnyValue + Send + Sync + 'static>;

    #[test]
    fn converter_convert_succeeds() {
        assert_eq!(<_ as Convertable<DebugConverter, String>>::to(1), "1");
        assert_eq!(
            <_ as Convertable<DebugConverter, String>>::to("1"),
            r#""1""#
        );

        assert_eq!(
            <_ as Convertable<IntoConverter, String>>::to("str"),
            String::from("str")
        );

        let res = <_ as Convertable<BoxConverter, BoxAnyValue>>::to("1");
        assert_eq!(format!("{res:?}"), "\"1\"");
    }

    #[test]
    fn converter_convert_succeeds_when_delegated_by_function() {
        fn do_with<C: Converter, S: Convertable<C, T>, T>(source: S) -> T {
            source.to()
        }

        assert_eq!(do_with::<DebugConverter, _, String>(1), "1");
        assert_eq!(do_with::<DebugConverter, _, String>("1"), "\"1\"");

        assert_eq!(
            do_with::<IntoConverter, _, String>("str"),
            String::from("str")
        );

        let res = do_with::<BoxConverter, _, BoxAnyValue>("1");
        assert_eq!(format!("{res:?}"), "\"1\"");
    }
}
