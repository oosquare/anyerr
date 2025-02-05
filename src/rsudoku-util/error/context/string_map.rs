use std::borrow::Borrow;
use std::fmt::{format, Debug, Display, Formatter, Result as FmtResult};
use std::hash::Hash;
use std::marker::PhantomData;
use std::slice::Iter as SliceIter;

use super::{AbstractContext, Context, Entry, Iter, Sealed, StringContext};

pub type StringKeyStringMapContext = StringMapContext<String, str>;
pub type StringKeyStringMapEntry = <StringKeyStringMapContext as AbstractContext>::Entry;
pub type StringKeyStringMapIter<'a> = <StringKeyStringMapContext as AbstractContext>::Iter<'a>;

pub type LiteralKeyStringMapContext = StringMapContext<String, str>;
pub type LiteralKeyStringMapEntry = <LiteralKeyStringMapContext as AbstractContext>::Entry;
pub type LiteralKeyStringMapIter<'a> = <LiteralKeyStringMapContext as AbstractContext>::Iter<'a>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringMapContext<K = String, KB = str>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
    entries: Vec<StringMapEntry<K, KB>>,
}

impl<K, KB> StringMapContext<K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn iter(&self) -> StringMapIter<'_, K, KB> {
        self.entries.iter().into()
    }
}

impl<K, KB> From<Vec<StringMapEntry<K, KB>>> for StringMapContext<K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
    fn from(entries: Vec<StringMapEntry<K, KB>>) -> Self {
        Self { entries }
    }
}

impl<K, KB, Q, V> From<Vec<(Q, V)>> for StringMapContext<K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
    Q: Into<<Self as AbstractContext>::Key>,
    V: Into<<Self as AbstractContext>::Value>,
{
    fn from(entries: Vec<(Q, V)>) -> Self {
        entries.into_iter().collect()
    }
}

impl<K, KB> FromIterator<StringMapEntry<K, KB>> for StringMapContext<K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
    fn from_iter<T: IntoIterator<Item = StringMapEntry<K, KB>>>(iter: T) -> Self {
        Self {
            entries: iter.into_iter().collect(),
        }
    }
}

impl<K, KB, Q, V> FromIterator<(Q, V)> for StringMapContext<K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
    Q: Into<<Self as AbstractContext>::Key>,
    V: Into<<Self as AbstractContext>::Value>,
{
    fn from_iter<T: IntoIterator<Item = (Q, V)>>(iter: T) -> Self {
        iter.into_iter()
            .map(|entry| StringMapEntry::from(entry))
            .collect()
    }
}

impl<K, KB> Default for StringMapContext<K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, KB> AbstractContext for StringMapContext<K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
    type Key = K;

    type Value = String;

    type Entry = StringMapEntry<K, KB>;

    type Iter<'a> = StringMapIter<'a, K, KB>;

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        self.entries.iter().into()
    }
}

impl<K, KB> Context for StringMapContext<K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
    fn insert<Q, V>(&mut self, key: Q, value: V)
    where
        Q: Into<Self::Key>,
        V: Into<Self::Value>,
    {
        self.entries.push(Self::Entry::new(key, value));
    }

    fn get<Q>(&self, key: &Q) -> Option<&<Self::Entry as Entry>::ValueBorrowed>
    where
        <Self::Entry as Entry>::KeyBorrowed: Borrow<Q>,
        Q: Debug + Display + Eq + Hash + ?Sized,
    {
        self.entries
            .iter()
            .find(|entry| entry.key().borrow() == key)
            .map(|entry| entry.value())
    }
}

impl<K, KB> Sealed for StringMapContext<K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
}

impl<K, KB> StringContext for StringMapContext<K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StringMapEntry<K = String, KB = str>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
    key: K,
    value: String,
    _phantom: PhantomData<&'static KB>,
}

impl<K, KB, Q, V> From<(Q, V)> for StringMapEntry<K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
    Q: Into<<Self as Entry>::Key>,
    V: Into<<Self as Entry>::Value>,
{
    fn from((key, value): (Q, V)) -> Self {
        Self::new(key.into(), value.into())
    }
}

impl<K, KB> Display for StringMapEntry<K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} = {}", self.key.borrow(), self.value)
    }
}

impl<K, KB> Entry for StringMapEntry<K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
    type Key = K;

    type KeyBorrowed = KB;

    type Value = String;

    type ValueBorrowed = str;

    fn new<Q, V>(key: Q, value: V) -> Self
    where
        Q: Into<Self::Key>,
        V: Into<Self::Value>,
    {
        Self {
            key: key.into(),
            value: format!("{:?}", value.into()),
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

pub enum StringMapIter<'a, K = String, KB = str>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
    Node {
        iter: SliceIter<'a, StringMapEntry<K, KB>>,
        next: Option<Box<StringMapIter<'a, K, KB>>>,
    },
    None,
}

impl<'a, K, KB> StringMapIter<'a, K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
    pub fn new() -> Self {
        Self::None
    }
}

impl<'a, K, KB> Default for StringMapIter<'a, K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, K, KB> From<SliceIter<'a, StringMapEntry<K, KB>>> for StringMapIter<'a, K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
    fn from(iter: SliceIter<'a, StringMapEntry<K, KB>>) -> Self {
        Self::Node { iter, next: None }
    }
}

impl<'a, K, KB> Iterator for StringMapIter<'a, K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
    type Item = &'a StringMapEntry<K, KB>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Self::Node { iter, next } = self {
            if let Some(item) = iter.next() {
                return Some(item);
            } else if let Some(next) = next.take() {
                *self = *next;
            } else {
                *self = Self::None;
            }
        }
        None
    }
}

impl<'a, K, KB> Iter<'a> for StringMapIter<'a, K, KB>
where
    K: Borrow<KB> + Debug + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + 'static,
{
    type Context = StringMapContext<K, KB>;

    type Entry = StringMapEntry<K, KB>;

    fn concat(self, context: &'a Self::Context) -> Self {
        if context.entries.is_empty() {
            return self;
        }
        let iter = context.entries.iter();
        if let Self::None = self {
            Self::Node { iter, next: None }
        } else {
            Self::Node {
                iter,
                next: Some(Box::new(self)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_entry_getter_succeeds() {
        {
            let entry = StringKeyStringMapEntry::new("key", "1");
            assert_eq!("key", entry.key());
            assert_eq!(r#""1""#, entry.value());
        }
        {
            let entry = StringKeyStringMapEntry::new("key", "&str value");
            assert_eq!("key", entry.key());
            assert_eq!(r#""&str value""#, entry.value());
        }
        {
            let entry = StringKeyStringMapEntry::new("key", &String::from("String value"));
            assert_eq!("key", entry.key());
            assert_eq!(r#""String value""#, entry.value());
        }
    }

    #[test]
    fn string_entry_to_string_succeeds() {
        {
            let entry = StringKeyStringMapEntry::new("key", "1");
            assert_eq!(r#"key = "1""#, entry.to_string());
        }
        {
            let entry = StringKeyStringMapEntry::new("key", "&str value");
            assert_eq!(r#"key = "&str value""#, entry.to_string());
        }

        {
            let entry = StringKeyStringMapEntry::new("key", &String::from("String value"));
            assert_eq!(r#"key = "String value""#, entry.to_string());
        }
    }

    #[test]
    fn string_context_iter_from_succeeds() {
        let context = StringMapContext::from(vec![
            StringKeyStringMapEntry::new("key1", "1"),
            StringKeyStringMapEntry::new("key2", "2"),
        ]);
        let mut iter = context.iter();
        assert_eq!(
            Some(&StringKeyStringMapEntry::new("key1", "1")),
            iter.next()
        );
        assert_eq!(
            Some(&StringKeyStringMapEntry::new("key2", "2")),
            iter.next()
        );
        assert_eq!(None, iter.next());
    }

    #[test]
    fn string_context_iter_concat_succeeds() {
        let context1 = StringMapContext::from(vec![
            StringKeyStringMapEntry::new("key1", "1"),
            StringKeyStringMapEntry::new("key2", "2"),
        ]);
        let context2 = StringMapContext::from(vec![
            StringKeyStringMapEntry::new("key3", "3"),
            StringKeyStringMapEntry::new("key4", "4"),
        ]);
        let mut iter = context2.iter().concat(&context1);
        assert_eq!(
            Some(&StringKeyStringMapEntry::new("key1", "1")),
            iter.next()
        );
        assert_eq!(
            Some(&StringKeyStringMapEntry::new("key2", "2")),
            iter.next()
        );
        assert_eq!(
            Some(&StringKeyStringMapEntry::new("key3", "3")),
            iter.next()
        );
        assert_eq!(
            Some(&StringKeyStringMapEntry::new("key4", "4")),
            iter.next()
        );
        assert_eq!(None, iter.next());
    }
}
