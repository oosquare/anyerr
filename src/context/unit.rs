use std::fmt::{Display, Formatter, Result as FmtResult};
use std::marker::PhantomData;

use crate::context::{AbstractContext, Entry, Iter, NoContext};

/// The context that stores nothing.
///
/// [`UnitContext`] is a ZST, thus integrating your error type with it leads to
/// zero memory overhead.
#[derive(Debug)]
pub struct UnitContext;

impl Default for UnitContext {
    fn default() -> Self {
        Self
    }
}

impl AbstractContext for UnitContext {
    type Key = Dummy;

    type Value = Dummy;

    type Entry = DummyEntry;

    type Iter<'a> = UnitIter<'a>;

    fn iter(&self) -> Self::Iter<'_> {
        Self::Iter::default()
    }
}

impl NoContext for UnitContext {}

/// An uninhabit type, used as dummy keys or values.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Dummy {}

impl Display for Dummy {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{self:?}")
    }
}

/// An uninhabit type, used as entries.
#[derive(Debug)]
pub enum DummyEntry {}

impl Entry for DummyEntry {
    type Key = Dummy;

    type KeyBorrowed = Dummy;

    type Value = Dummy;

    type ValueBorrowed = Dummy;

    fn new<Q, V>(_key: Q, _value: V) -> Self
    where
        Q: Into<Self::Key>,
        V: Into<Self::Value>,
    {
        unreachable!("`_key` and `_value` are instances of the `Dummy` type, which is uninhabited")
    }

    fn key(&self) -> &Self::KeyBorrowed {
        unreachable!("`_key` and `_value` are instances of the `Dummy` type, which is uninhabited")
    }

    fn value(&self) -> &Self::ValueBorrowed {
        unreachable!("`_key` and `_value` are instances of the `Dummy` type, which is uninhabited")
    }
}

/// The iterator of [`UnitContext`], producing nothing.
#[derive(Debug, Default)]
pub struct UnitIter<'a> {
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Iterator for UnitIter<'a> {
    type Item = &'a DummyEntry;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<'a> Iter<'a> for UnitIter<'a> {
    type Entry = DummyEntry;

    fn compose(self, _other: Self) -> Self {
        self
    }
}
