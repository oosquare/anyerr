pub mod string_map;
pub mod unit;

pub use string_map::StringMapContext;
pub use unit::UnitContext;

use std::borrow::Borrow;
use std::fmt::{Debug, Display};
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextDepth {
    All,
    Shallowest,
}

pub trait AbstractContext: Default + Debug {
    type Key;

    type Value;

    type Entry: Entry<Key = Self::Key, Value = Self::Value>;

    type Iter<'a>: Iter<'a, Context = Self, Entry = Self::Entry>
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Self::Iter<'a>;
}

pub trait NoContext: AbstractContext {}

pub trait Context: AbstractContext {
    fn insert<Q, V>(&mut self, key: Q, value: V)
    where
        Q: Into<Self::Key>,
        V: Into<Self::Value>;

    fn get<Q>(&self, key: &Q) -> Option<&<Self::Entry as Entry>::ValueBorrowed>
    where
        <Self::Entry as Entry>::KeyBorrowed: Borrow<Q>,
        Q: Debug + Display + Eq + Hash + ?Sized;
}

pub trait Entry {
    type Key: Borrow<Self::KeyBorrowed>;

    type KeyBorrowed: Debug + Display + Eq + Hash + ?Sized;

    type Value: Borrow<Self::ValueBorrowed>;

    type ValueBorrowed: Debug + ?Sized;

    fn new<Q, V>(key: Q, value: V) -> Self
    where
        Q: Into<Self::Key>,
        V: Into<Self::Value>;

    fn key(&self) -> &Self::KeyBorrowed;

    fn value(&self) -> &Self::ValueBorrowed;
}

pub trait Iter<'a>: Default + Iterator<Item = &'a Self::Entry> {
    type Context: AbstractContext<Entry = Self::Entry, Iter<'a> = Self> + 'a;

    type Entry: 'a;

    fn concat(self, context: &'a Self::Context) -> Self;
}
