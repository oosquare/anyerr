use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::Hash;
use std::slice::Iter as SliceIter;

use super::{AbstractContext, Context, Entry, Iter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringMapContext {
    entries: Vec<StringMapEntry>,
}

impl StringMapContext {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn iter(&self) -> StringMapIter {
        self.entries.iter().into()
    }
}

impl From<Vec<StringMapEntry>> for StringMapContext {
    fn from(entries: Vec<StringMapEntry>) -> Self {
        Self { entries }
    }
}

impl<Q, V> From<Vec<(Q, V)>> for StringMapContext
where
    Q: Into<<Self as AbstractContext>::Key>,
    V: Into<<Self as AbstractContext>::Value>,
{
    fn from(entries: Vec<(Q, V)>) -> Self {
        entries.into_iter().collect()
    }
}

impl FromIterator<StringMapEntry> for StringMapContext {
    fn from_iter<T: IntoIterator<Item = StringMapEntry>>(iter: T) -> Self {
        Self {
            entries: iter.into_iter().collect(),
        }
    }
}

impl<Q, V> FromIterator<(Q, V)> for StringMapContext
where
    Q: Into<<Self as AbstractContext>::Key>,
    V: Into<<Self as AbstractContext>::Value>,
{
    fn from_iter<T: IntoIterator<Item = (Q, V)>>(iter: T) -> Self {
        iter.into_iter()
            .map(|entry| StringMapEntry::from(entry))
            .collect()
    }
}

impl Default for StringMapContext {
    fn default() -> Self {
        Self::new()
    }
}

impl AbstractContext for StringMapContext {
    type Key = String;

    type Value = String;

    type Entry = StringMapEntry;

    type Iter<'a> = StringMapIter<'a>;

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        self.entries.iter().into()
    }
}

impl Context for StringMapContext {
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StringMapEntry {
    content: String,
    key_len: usize,
}

impl StringMapEntry {
    fn split_content(&self) -> (&str, &str) {
        self.content.split_at(self.key_len)
    }
}

impl<Q, V> From<(Q, V)> for StringMapEntry
where
    Q: Into<<Self as Entry>::Key>,
    V: Into<<Self as Entry>::Value>,
{
    fn from((key, value): (Q, V)) -> Self {
        Self::new(key.into(), value.into())
    }
}

impl Debug for StringMapEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let (name, value) = self.split_content();
        f.debug_struct("StringEntry")
            .field("key", &name)
            .field("value", &value)
            .finish()
    }
}

impl Display for StringMapEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let (key, value) = self.split_content();
        write!(f, "{key} = {value}")
    }
}

impl Entry for StringMapEntry {
    type Key = String;

    type KeyBorrowed = str;

    type Value = String;

    type ValueBorrowed = str;

    fn new<Q, V>(key: Q, value: V) -> Self
    where
        Q: Into<Self::Key>,
        V: Into<Self::Value>,
    {
        let key = key.into();
        Self {
            content: format!("{}{:?}", key, value.into()),
            key_len: key.len(),
        }
    }

    fn key(&self) -> &Self::KeyBorrowed {
        self.split_content().0
    }

    fn value(&self) -> &Self::ValueBorrowed {
        self.split_content().1
    }
}

pub enum StringMapIter<'a> {
    Node {
        iter: SliceIter<'a, StringMapEntry>,
        next: Option<Box<StringMapIter<'a>>>,
    },
    None,
}

impl<'a> StringMapIter<'a> {
    pub fn new() -> Self {
        Self::None
    }
}

impl<'a> Default for StringMapIter<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<SliceIter<'a, StringMapEntry>> for StringMapIter<'a> {
    fn from(iter: SliceIter<'a, StringMapEntry>) -> Self {
        Self::Node { iter, next: None }
    }
}

impl<'a> Iterator for StringMapIter<'a> {
    type Item = &'a StringMapEntry;

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

impl<'a> Iter<'a> for StringMapIter<'a> {
    type Context = StringMapContext;

    type Entry = StringMapEntry;

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
            let entry = StringMapEntry::new("key", "1");
            assert_eq!("key", entry.key());
            assert_eq!(r#""1""#, entry.value());
        }
        {
            let entry = StringMapEntry::new("key", "&str value");
            assert_eq!("key", entry.key());
            assert_eq!(r#""&str value""#, entry.value());
        }
        {
            let entry = StringMapEntry::new("key", &String::from("String value"));
            assert_eq!("key", entry.key());
            assert_eq!(r#""String value""#, entry.value());
        }
    }

    #[test]
    fn string_entry_to_string_succeeds() {
        {
            let entry = StringMapEntry::new("key", "1");
            assert_eq!(r#"key = "1""#, entry.to_string());
        }
        {
            let entry = StringMapEntry::new("key", "&str value");
            assert_eq!(r#"key = "&str value""#, entry.to_string());
        }

        {
            let entry = StringMapEntry::new("key", &String::from("String value"));
            assert_eq!(r#"key = "String value""#, entry.to_string());
        }
    }

    #[test]
    fn string_context_iter_from_succeeds() {
        let context = StringMapContext::from(vec![
            StringMapEntry::new("key1", "1"),
            StringMapEntry::new("key2", "2"),
        ]);
        let mut iter = context.iter();
        assert_eq!(Some(&StringMapEntry::new("key1", "1")), iter.next());
        assert_eq!(Some(&StringMapEntry::new("key2", "2")), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn string_context_iter_concat_succeeds() {
        let context1 = StringMapContext::from(vec![
            StringMapEntry::new("key1", "1"),
            StringMapEntry::new("key2", "2"),
        ]);
        let context2 = StringMapContext::from(vec![
            StringMapEntry::new("key3", "3"),
            StringMapEntry::new("key4", "4"),
        ]);
        let mut iter = context2.iter().concat(&context1);
        assert_eq!(Some(&StringMapEntry::new("key1", "1")), iter.next());
        assert_eq!(Some(&StringMapEntry::new("key2", "2")), iter.next());
        assert_eq!(Some(&StringMapEntry::new("key3", "3")), iter.next());
        assert_eq!(Some(&StringMapEntry::new("key4", "4")), iter.next());
        assert_eq!(None, iter.next());
    }
}
