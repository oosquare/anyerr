pub mod error;
pub mod result;

pub trait Overlay: Sized {
    type Output: Overlay;

    type Intermediate: Intermediate<Output = Self::Output>;

    fn overlay<V>(self, value: V) -> Self::Intermediate
    where
        V: Applicable<Self, Output = Self::Intermediate>,
    {
        value.apply(self)
    }
}

pub trait Intermediate: Sized {
    type Output: Overlay;

    fn context<Q, R>(self, key: Q, value: R) -> Self
    where
        (Q, R): Applicable<Self, Output = Self>,
    {
        (key, value).apply(self)
    }

    fn build(self) -> Self::Output;
}

pub trait Applicable<T> {
    type Output;

    fn apply(self, target: T) -> Self::Output;
}
