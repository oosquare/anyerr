use std::any::Any;
use std::fmt::Debug;

/// A type alias of the trait object of the [`AnyValue`] trait.
pub type DynAnyValue = dyn AnyValue + Send + Sync + 'static;

/// A [`Send`] and [`Sync`] type that is capable of type reflection and
/// displaying itself.
pub trait AnyValue: Any + Debug + Send + Sync + 'static {
    /// Returns a reference to itself as a trait object of the [`Any`] trait,
    /// i.e. upcasts itself to a trait object of [`Any`].
    fn as_any(&self) -> &dyn Any;
}

impl<T> AnyValue for T
where
    T: Any + Debug + Send + Sync,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl dyn AnyValue + Send + Sync + 'static {
    /// Returns `true` if the inner type is the same as `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.as_any().is::<T>()
    }

    /// Returns some reference to the inner value if it is of type `T`, or
    /// `None` if it isn't.
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_value_rtti_succeeds() {
        let x: Box<dyn AnyValue + Send + Sync + 'static> = Box::new(String::from("any value"));
        assert!(x.is::<String>() && !x.is::<i32>());
        assert_eq!(x.downcast_ref::<String>().unwrap(), "any value");
    }

    #[test]
    fn any_value_debug_succeeds() {
        let x: Box<dyn AnyValue + Send + Sync + 'static> = Box::new(String::from("any value"));
        assert_eq!(format!("{x:?}"), "\"any value\"");
    }
}
