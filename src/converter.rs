use std::fmt::Debug;

use super::context::AnyValue;

pub trait Converter: Debug + Send + Sync + 'static {}

#[derive(Debug)]
pub struct DebugConverter;

impl Converter for DebugConverter {}

#[derive(Debug)]
pub struct IntoConverter;

impl Converter for IntoConverter {}

#[derive(Debug)]
pub struct BoxConverter;

impl Converter for BoxConverter {}

pub trait Convertable<C: Converter, T>: Sized {
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
