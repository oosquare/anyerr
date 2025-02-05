use std::fmt::{Display, Formatter, Result as FmtResult};
use std::marker::PhantomData;

use super::{AbstractContext, Entry, Iter, NoContext, Sealed};

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

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        Self::Iter::default()
    }
}

impl Sealed for UnitContext {}

impl NoContext for UnitContext {}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Dummy {}

impl Display for Dummy {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{self:?}")
    }
}

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

#[derive(Debug)]
pub struct UnitIter<'a> {
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Default for UnitIter<'a> {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<'a> Iterator for UnitIter<'a> {
    type Item = &'a DummyEntry;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<'a> Iter<'a> for UnitIter<'a> {
    type Context = UnitContext;

    type Entry = DummyEntry;

    fn concat(self, _context: &'a Self::Context) -> Self {
        self
    }
}
