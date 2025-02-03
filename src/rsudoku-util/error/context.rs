use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::slice::Iter;

use getset::Getters;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Getters)]
#[getset(get = "pub")]
pub struct ContextEntry {
    content: String,
    name_len: usize,
}

impl ContextEntry {
    pub(super) fn new<V: Debug>(name: &str, value: &V) -> Self {
        Self {
            content: format!("{name}{value:?}"),
            name_len: name.len(),
        }
    }

    pub fn name(&self) -> &str {
        self.split_content().0
    }

    pub fn value(&self) -> &str {
        self.split_content().1
    }

    fn split_content(&self) -> (&str, &str) {
        self.content.split_at(self.name_len)
    }
}

impl Debug for ContextEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let (name, value) = self.split_content();
        f.debug_struct("ContextEntry")
            .field("name", &name)
            .field("value", &value)
            .finish()
    }
}

impl Display for ContextEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let (name, value) = self.split_content();
        write!(f, "{name} = {value}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMap {
    entries: Vec<ContextEntry>,
}

impl ContextMap {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn iter(&self) -> ContextIter {
        self.entries.iter().into()
    }
}

impl From<Vec<ContextEntry>> for ContextMap {
    fn from(entries: Vec<ContextEntry>) -> Self {
        Self { entries }
    }
}

impl FromIterator<ContextEntry> for ContextMap {
    fn from_iter<T: IntoIterator<Item = ContextEntry>>(iter: T) -> Self {
        Self {
            entries: iter.into_iter().collect(),
        }
    }
}

pub enum ContextIter<'a> {
    Node {
        iter: Iter<'a, ContextEntry>,
        next: Option<Box<ContextIter<'a>>>,
    },
    None,
}

impl<'a> ContextIter<'a> {
    pub(super) fn new() -> Self {
        Self::None
    }

    pub(super) fn concat(self, context: &'a ContextMap) -> Self {
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

impl<'a> From<Iter<'a, ContextEntry>> for ContextIter<'a> {
    fn from(iter: Iter<'a, ContextEntry>) -> Self {
        Self::Node { iter, next: None }
    }
}

impl<'a> Iterator for ContextIter<'a> {
    type Item = &'a ContextEntry;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextDepth {
    All,
    Shallowest,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_entry_getter_succeeds() {
        {
            let entry = ContextEntry::new("name", &1);
            assert_eq!("name", entry.name());
            assert_eq!(r#"1"#, entry.value());
        }
        {
            let entry = ContextEntry::new("name", &"&str value");
            assert_eq!("name", entry.name());
            assert_eq!(r#""&str value""#, entry.value());
        }
        {
            let entry = ContextEntry::new("name", &String::from("String value"));
            assert_eq!("name", entry.name());
            assert_eq!(r#""String value""#, entry.value());
        }
    }

    #[test]
    fn context_entry_to_string_succeeds() {
        {
            let entry = ContextEntry::new("name", &1);
            assert_eq!(r#"name = 1"#, entry.to_string());
        }
        {
            let entry = ContextEntry::new("name", &"&str value");
            assert_eq!(r#"name = "&str value""#, entry.to_string());
        }

        {
            let entry = ContextEntry::new("name", &String::from("String value"));
            assert_eq!(r#"name = "String value""#, entry.to_string());
        }
    }

    #[test]
    fn context_iter_from_succeeds() {
        let context = ContextMap::from(vec![
            ContextEntry::new("name1", &1),
            ContextEntry::new("name2", &"2"),
        ]);
        let mut iter = context.iter();
        assert_eq!(Some(&ContextEntry::new("name1", &1)), iter.next());
        assert_eq!(Some(&ContextEntry::new("name2", &"2")), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn context_iter_concat_succeeds() {
        let context1 = ContextMap::from(vec![
            ContextEntry::new("name1", &1),
            ContextEntry::new("name2", &2),
        ]);
        let context2 = ContextMap::from(vec![
            ContextEntry::new("name3", &3),
            ContextEntry::new("name4", &4),
        ]);
        let mut iter = context2.iter().concat(&context1);
        assert_eq!(Some(&ContextEntry::new("name1", &1)), iter.next());
        assert_eq!(Some(&ContextEntry::new("name2", &2)), iter.next());
        assert_eq!(Some(&ContextEntry::new("name3", &3)), iter.next());
        assert_eq!(Some(&ContextEntry::new("name4", &4)), iter.next());
        assert_eq!(None, iter.next());
    }
}
