use std::fmt::Debug;

pub trait Converter<S, T> {
    fn convert(source: S) -> T;

    fn run(&self, source: S) -> T {
        Self::convert(source)
    }
}

pub struct DebugConverter;

impl<S: Debug, T: From<String>> Converter<S, T> for DebugConverter {
    fn convert(source: S) -> T {
        format!("{source:?}").into()
    }
}

pub struct IntoConverter;

impl<S: Into<T>, T> Converter<S, T> for IntoConverter {
    fn convert(source: S) -> T {
        source.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converter_convert_succeeds() {
        {
            assert_eq!(<DebugConverter as Converter<_, String>>::convert(1), "1");
            assert_eq!(
                <DebugConverter as Converter<_, String>>::convert("1"),
                r#""1""#
            );
        }
        {
            assert_eq!(
                <IntoConverter as Converter<_, String>>::convert("str"),
                String::from("str")
            );
        }
    }

    #[test]
    fn converter_convert_succeeds_when_delegated_by_function() {
        fn do_with<C: Converter<S, T>, S, T>(source: S) -> T {
            C::convert(source)
        }

        {
            assert_eq!(do_with::<DebugConverter, _, String>(1), "1");
            assert_eq!(do_with::<DebugConverter, _, String>("1"), "\"1\"");
        }
        {
            assert_eq!(
                do_with::<IntoConverter, _, String>("str"),
                String::from("str")
            );
        }
    }
}
