pub mod iter;
pub mod map;
pub mod singleton;
pub mod unit;

use std::any::Any;
use std::borrow::Borrow;
use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::converter::{Convertable, Converter};

pub use map::{AnyMapContext, LiteralKeyStringMapContext, StringKeyStringMapContext};
pub use map::{AnyValue, DynAnyValue};
pub use singleton::{AnySingletonContext, FixedSingletonContext, StringSingletonContext};
pub use unit::UnitContext;

pub trait AbstractContext: Default + Debug + Send + Sync + 'static {
    type Key;

    type Value;

    type Entry: Entry<Key = Self::Key, Value = Self::Value>;

    type Iter<'a>: Iter<'a, Entry = Self::Entry>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_>;
}

#[allow(private_bounds)]
pub trait NoContext: AbstractContext + Sealed {}

pub trait Context: AbstractContext {
    type Converter: Converter;

    fn insert<Q, R>(&mut self, key: Q, value: R)
    where
        Q: Into<Self::Key>,
        R: Into<Self::Value>;

    fn insert_with<C, Q, R>(&mut self, key: Q, value: R)
    where
        Q: Into<Self::Key>,
        C: Converter,
        R: Convertable<C, Self::Value>,
    {
        self.insert(key, value.to());
    }

    fn get<Q>(&self, key: &Q) -> Option<&<Self::Entry as Entry>::ValueBorrowed>
    where
        <Self::Entry as Entry>::KeyBorrowed: Borrow<Q>,
        Q: Debug + Eq + Hash + ?Sized;
}

#[allow(private_bounds)]
pub trait SingletonContext: Context + Sealed {
    fn value(&self) -> Option<&<Self::Entry as Entry>::ValueBorrowed>;
}

#[allow(private_bounds)]
pub trait StringContext
where
    Self: Context<Value = String, Entry: Entry<ValueBorrowed = str>>,
    Self: Sealed,
{
}

#[allow(private_bounds)]
pub trait AnyContext
where
    Self: Context<Value = Box<DynAnyValue>, Entry: Entry<ValueBorrowed = DynAnyValue>>,
    Self: Sealed,
{
    fn value_as<T, Q>(&self, key: &Q) -> Option<&T>
    where
        <Self::Entry as Entry>::KeyBorrowed: Borrow<Q>,
        Q: Debug + Eq + Hash + ?Sized,
        T: Any,
    {
        self.get(key).and_then(|value| value.downcast_ref::<T>())
    }
}

pub trait Entry: Debug + Send + Sync + 'static {
    type Key: Borrow<Self::KeyBorrowed> + Debug + Send + Sync + 'static;

    type KeyBorrowed: Debug + Display + Eq + Hash + ?Sized + Send + Sync + 'static;

    type Value: Borrow<Self::ValueBorrowed> + Debug + Send + Sync + 'static;

    type ValueBorrowed: Debug + ?Sized + Send + Sync + 'static;

    fn new<Q, R>(key: Q, value: R) -> Self
    where
        Q: Into<Self::Key>,
        R: Into<Self::Value>;

    fn key(&self) -> &Self::KeyBorrowed;

    fn value(&self) -> &Self::ValueBorrowed;
}

pub trait Iter<'a>: Default + Iterator<Item = &'a Self::Entry> {
    type Entry: 'a;

    fn compose(self, other: Self) -> Self;
}

trait Sealed {}
