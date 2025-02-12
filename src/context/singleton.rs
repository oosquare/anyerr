mod facade;

use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::Hash;
use std::marker::PhantomData;
use std::option::Iter as InnerIter;

use crate::context::iter::CommonIter;
use crate::context::{AbstractContext, Context, Entry, SingletonContext};
use crate::converter::IntoConverter;

pub use facade::*;

pub type OptionIter<'a, E> = CommonIter<'a, E, InnerIter<'a, E>>;

/// The common implementation of [`SingletonContext`].
#[derive(Debug)]
pub struct OptionContext<E>
where
    E: Entry<Key = OptionKey, KeyBorrowed = OptionKey>,
{
    entry: Option<E>,
}

impl<E> OptionContext<E>
where
    E: Entry<Key = OptionKey, KeyBorrowed = OptionKey>,
{
    pub fn new() -> Self {
        Default::default()
    }
}

impl<E> Default for OptionContext<E>
where
    E: Entry<Key = OptionKey, KeyBorrowed = OptionKey>,
{
    fn default() -> Self {
        Self { entry: None }
    }
}

impl<E> From<E> for OptionContext<E>
where
    E: Entry<Key = OptionKey, KeyBorrowed = OptionKey>,
{
    fn from(entry: E) -> Self {
        Self { entry: Some(entry) }
    }
}

impl<E> AbstractContext for OptionContext<E>
where
    E: Entry<Key = OptionKey, KeyBorrowed = OptionKey>,
{
    type Key = E::Key;

    type Value = E::Value;

    type Entry = E;

    type Iter<'a>
        = OptionIter<'a, E>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        if self.entry.is_some() {
            OptionIter::from(self.entry.iter())
        } else {
            OptionIter::new()
        }
    }
}

impl<E> Context for OptionContext<E>
where
    E: Entry<Key = OptionKey, KeyBorrowed = OptionKey>,
{
    type Converter = IntoConverter;

    fn insert<Q, R>(&mut self, key: Q, value: R)
    where
        Q: Into<Self::Key>,
        R: Into<Self::Value>,
    {
        self.entry = Some(Self::Entry::new(key.into(), value.into()))
    }

    fn get<Q>(&self, _key: &Q) -> Option<&<Self::Entry as Entry>::ValueBorrowed>
    where
        <Self::Entry as Entry>::KeyBorrowed: Borrow<Q>,
        Q: Debug + Eq + Hash + ?Sized,
    {
        self.value()
    }
}

impl<E> SingletonContext for OptionContext<E>
where
    E: Entry<Key = OptionKey, KeyBorrowed = OptionKey>,
{
    fn value(&self) -> Option<&<Self::Entry as Entry>::ValueBorrowed> {
        self.entry.as_ref().map(Entry::value)
    }
}

/// The entry used by [`OptionContext`].
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct OptionEntry<V, VB>
where
    V: Borrow<VB> + Debug + Send + Sync + 'static,
    VB: Debug + ?Sized + Send + Sync + 'static,
{
    value: V,
    _phantom: PhantomData<Box<VB>>,
}

impl<V, VB> From<V> for OptionEntry<V, VB>
where
    V: Borrow<VB> + Debug + Send + Sync + 'static,
    VB: Debug + ?Sized + Send + Sync + 'static,
{
    fn from(value: V) -> Self {
        Self::new(OptionKey::SELF_VALUE, value)
    }
}

impl<V, VB> Entry for OptionEntry<V, VB>
where
    V: Borrow<VB> + Debug + Send + Sync + 'static,
    VB: Debug + ?Sized + Send + Sync + 'static,
{
    type Key = OptionKey;

    type KeyBorrowed = OptionKey;

    type Value = V;

    type ValueBorrowed = VB;

    fn new<Q, R>(_key: Q, value: R) -> Self
    where
        Q: Into<Self::Key>,
        R: Into<Self::Value>,
    {
        Self {
            value: value.into(),
            _phantom: Default::default(),
        }
    }

    fn key(&self) -> &Self::KeyBorrowed {
        &OptionKey::SELF_VALUE
    }

    fn value(&self) -> &Self::ValueBorrowed {
        self.value.borrow()
    }
}

/// The key of [`OptionEntry`].
///
/// It's in fact equivlaent to the unit type, i.e. `()`, and conversions
/// between `()` are also offered. Usually you just need to use `()` in where
/// the key is expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct OptionKey;

impl OptionKey {
    const SELF_VALUE: OptionKey = OptionKey;
    const UNIT_VALUE: () = ();
}

impl Display for OptionKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "OptionKey")
    }
}

impl From<()> for OptionKey {
    fn from(_value: ()) -> Self {
        Self::SELF_VALUE
    }
}

impl Borrow<()> for OptionKey {
    fn borrow(&self) -> &() {
        &Self::UNIT_VALUE
    }
}

#[cfg(test)]
mod tests {
    use crate::context::Iter;

    use super::*;

    type TestEntry = OptionEntry<String, str>;
    type TestContext = OptionContext<TestEntry>;

    #[test]
    fn option_entry_getter_succeeds() {
        let entry = TestEntry::new((), "test");
        assert_eq!(entry.value(), "test");
    }

    #[test]
    fn option_context_operations_succeeds() {
        let mut ctx = TestContext::new();
        assert!(ctx.get(&()).is_none());

        ctx.insert((), "test");
        assert_eq!(ctx.get(&()).unwrap(), "test");
        ctx.insert((), "test1");
        assert_eq!(ctx.get(&()).unwrap(), "test1");
    }

    #[test]
    fn option_iter_next_succeeds() {
        let mut ctx = TestContext::new();
        let mut iter = ctx.iter();
        assert_eq!(iter.next(), None);

        ctx.insert((), "test");
        let mut iter = ctx.iter();
        assert_eq!(iter.next().unwrap().value(), "test");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn option_iter_compose_succeeds() {
        let mut ctx1 = TestContext::new();
        ctx1.insert((), "test1");
        let mut ctx2 = TestContext::new();
        ctx2.insert((), "test2");

        let mut iter = ctx1.iter().compose(ctx2.iter());
        assert_eq!(iter.next().unwrap().value(), "test1");
        assert_eq!(iter.next().unwrap().value(), "test2");
        assert_eq!(iter.next(), None);
    }
}
