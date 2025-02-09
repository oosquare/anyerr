mod any;
mod facade;

pub use any::*;
pub use facade::*;

use std::borrow::Borrow;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::marker::PhantomData;
use std::slice::Iter as SliceIter;

use crate::error::context::iter::CommonIter;
use crate::error::context::{AbstractContext, Context, Entry};
use crate::error::converter::Converter;

pub type MapIter<'a, E> = CommonIter<'a, E, SliceIter<'a, E>>;

#[derive(Debug, PartialEq, Eq)]
pub struct MapContext<E: Entry, C: Converter> {
    entries: Vec<E>,
    _phantom: PhantomData<C>,
}

impl<E: Entry, C: Converter> MapContext<E, C> {
    pub fn new() -> Self {
        Self::from(Vec::<E>::new())
    }

    pub fn iter(&self) -> MapIter<'_, E> {
        self.entries.iter().into()
    }
}

impl<E: Entry, C: Converter> From<Vec<E>> for MapContext<E, C> {
    fn from(entries: Vec<E>) -> Self {
        Self {
            entries,
            _phantom: Default::default(),
        }
    }
}

impl<E: Entry, C: Converter, Q, R> From<Vec<(Q, R)>> for MapContext<E, C>
where
    Q: Into<<Self as AbstractContext>::Key>,
    R: Into<<Self as AbstractContext>::Value>,
{
    fn from(entries: Vec<(Q, R)>) -> Self {
        entries.into_iter().collect()
    }
}

impl<E: Entry, C: Converter> FromIterator<E> for MapContext<E, C> {
    fn from_iter<T: IntoIterator<Item = E>>(iter: T) -> Self {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}

impl<E: Entry, C: Converter, Q, R> FromIterator<(Q, R)> for MapContext<E, C>
where
    Q: Into<<Self as AbstractContext>::Key>,
    R: Into<<Self as AbstractContext>::Value>,
{
    fn from_iter<T: IntoIterator<Item = (Q, R)>>(iter: T) -> Self {
        iter.into_iter()
            .map(|(key, value)| E::new(key.into(), value.into()))
            .collect()
    }
}

impl<E: Entry, C: Converter> Default for MapContext<E, C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Entry, C: Converter> AbstractContext for MapContext<E, C> {
    type Key = E::Key;

    type Value = E::Value;

    type Entry = E;

    type Iter<'a>
        = MapIter<'a, E>
    where
        E: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        self.entries.iter().into()
    }
}

impl<E: Entry, C: Converter> Context for MapContext<E, C> {
    type Converter = C;

    fn insert<Q, R>(&mut self, key: Q, value: R)
    where
        Q: Into<Self::Key>,
        R: Into<Self::Value>,
    {
        self.entries.push(Self::Entry::new(key, value));
    }

    fn get<Q>(&self, key: &Q) -> Option<&<Self::Entry as Entry>::ValueBorrowed>
    where
        <Self::Entry as Entry>::KeyBorrowed: Borrow<Q>,
        Q: Debug + Eq + Hash + ?Sized,
    {
        self.entries
            .iter()
            .find(|entry| entry.key().borrow() == key)
            .map(Entry::value)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct MapEntry<K, KB, V, VB>
where
    K: Borrow<KB> + Debug + Send + Sync + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + Send + Sync,
    V: Borrow<VB> + Debug + Send + Sync + 'static,
    VB: Debug + ?Sized + Send + Sync,
{
    key: K,
    value: V,
    _phantom: PhantomData<(Box<KB>, Box<VB>)>,
}

impl<K, KB, V, VB, Q, R> From<(Q, R)> for MapEntry<K, KB, V, VB>
where
    K: Borrow<KB> + Debug + Send + Sync + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + Send + Sync,
    V: Borrow<VB> + Debug + Send + Sync + 'static,
    VB: Debug + ?Sized + Send + Sync,
    Q: Into<<Self as Entry>::Key>,
    R: Into<<Self as Entry>::Value>,
{
    fn from((key, value): (Q, R)) -> Self {
        Self::new(key.into(), value.into())
    }
}

impl<K, KB, V, VB> Entry for MapEntry<K, KB, V, VB>
where
    K: Borrow<KB> + Debug + Send + Sync + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + Send + Sync,
    V: Borrow<VB> + Debug + Send + Sync + 'static,
    VB: Debug + ?Sized + Send + Sync,
{
    type Key = K;

    type KeyBorrowed = KB;

    type Value = V;

    type ValueBorrowed = VB;

    fn new<Q, R>(key: Q, value: R) -> Self
    where
        Q: Into<Self::Key>,
        R: Into<Self::Value>,
    {
        Self {
            key: key.into(),
            value: value.into(),
            _phantom: Default::default(),
        }
    }

    fn key(&self) -> &Self::KeyBorrowed {
        self.key.borrow()
    }

    fn value(&self) -> &Self::ValueBorrowed {
        self.value.borrow()
    }
}

#[cfg(test)]
mod tests {
    use crate::error::context::Iter;
    use crate::error::converter::DebugConverter;

    use super::*;

    type TestEntry = MapEntry<String, str, String, str>;
    type TestContext = MapContext<TestEntry, DebugConverter>;

    #[test]
    fn string_entry_getter_succeeds() {
        let entry = TestEntry::new("key", "1");
        assert_eq!("key", entry.key());
        assert_eq!("1", entry.value());
    }

    #[test]
    fn string_map_context_operation_succeeds() {
        let mut context = TestContext::new();
        context.insert("key1", "1");
        context.insert_with::<DebugConverter, _, _>("key2", 2);
        context.insert_with::<DebugConverter, _, _>("key3", "3");
        assert_eq!(context.get("key1").unwrap(), "1");
        assert_eq!(context.get("key2").unwrap(), "2");
        assert_eq!(context.get("key3").unwrap(), "\"3\"");
    }

    #[test]
    fn string_map_context_iter_from_succeeds() {
        let context = TestContext::from(vec![
            TestEntry::new("key1", "1"),
            TestEntry::new("key2", "2"),
        ]);
        let mut iter = context.iter();
        assert_eq!(Some(&TestEntry::new("key1", "1")), iter.next());
        assert_eq!(Some(&TestEntry::new("key2", "2")), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn string_map_context_iter_concat_succeeds() {
        let context1 = TestContext::from(vec![
            TestEntry::new("key1", "1"),
            TestEntry::new("key2", "2"),
        ]);
        let context2 = TestContext::from(vec![
            TestEntry::new("key3", "3"),
            TestEntry::new("key4", "4"),
        ]);
        let mut iter = context1.iter().compose(context2.iter());
        assert_eq!(Some(&TestEntry::new("key1", "1")), iter.next());
        assert_eq!(Some(&TestEntry::new("key2", "2")), iter.next());
        assert_eq!(Some(&TestEntry::new("key3", "3")), iter.next());
        assert_eq!(Some(&TestEntry::new("key4", "4")), iter.next());
        assert_eq!(None, iter.next());
    }
}
