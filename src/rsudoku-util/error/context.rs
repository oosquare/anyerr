pub mod map;
pub mod unit;

use std::any::Any;
use std::borrow::Borrow;
use std::fmt::{Debug, Display};
use std::hash::Hash;

use super::converter::{Convertable, Converter};

pub use map::{AnyValue, DynAnyValue, LiteralKeyStringMapContext, StringKeyStringMapContext};
pub use unit::UnitContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextDepth {
    All,
    Shallowest,
}

pub trait AbstractContext: Default + Debug + Send + Sync {
    type Key;

    type Value;

    type Entry: Entry<Key = Self::Key, Value = Self::Value>;

    type Iter<'a>: Iter<'a, Context = Self, Entry = Self::Entry>
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
        Q: Debug + Display + Eq + Hash + ?Sized;
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
        Q: Debug + Display + Eq + Hash + ?Sized,
        T: Any,
    {
        self.get(key).and_then(|value| value.downcast_ref::<T>())
    }
}

pub trait Entry: Debug + Send + Sync {
    type Key: Borrow<Self::KeyBorrowed> + Debug + Send + Sync + 'static;

    type KeyBorrowed: Debug + Display + Eq + Hash + ?Sized + Send + Sync;

    type Value: Borrow<Self::ValueBorrowed> + Debug + Send + Sync + 'static;

    type ValueBorrowed: Debug + ?Sized + Send + Sync;

    fn new<Q, R>(key: Q, value: R) -> Self
    where
        Q: Into<Self::Key>,
        R: Into<Self::Value>;

    fn key(&self) -> &Self::KeyBorrowed;

    fn value(&self) -> &Self::ValueBorrowed;
}

pub trait Iter<'a>: Default + Iterator<Item = &'a Self::Entry> {
    type Context: AbstractContext<Entry = Self::Entry, Iter<'a> = Self> + 'a;

    type Entry: 'a;

    fn concat(self, context: &'a Self::Context) -> Self;
}

trait Sealed {}
