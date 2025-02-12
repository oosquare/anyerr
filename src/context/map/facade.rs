use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::Hash;

use crate::context::any::DynAnyValue;
use crate::context::map::{MapContext, MapEntry, MapIter};
use crate::context::{AbstractContext, AnyContext, Entry, StringContext};
use crate::converter::{BoxConverter, DebugConverter};

/// A map entry whose value is a [`String`].
pub type StringMapEntry<K, KB> = MapEntry<K, KB, String, str>;
/// A context whose entries are [`StringMapEntry<K, KB>`].
pub type StringMapContext<K, KB> = MapContext<StringMapEntry<K, KB>, DebugConverter>;
/// The iterator of [`StringMapContext<K, KB>`].
pub type StringMapIter<'a, K, KB> = MapIter<'a, StringMapEntry<K, KB>>;

/// A [`StringMapContext<K, KB>`] which uses [`String`] as its keys.
pub type StringKeyStringMapContext = StringMapContext<String, str>;
/// A [`StringMapEntry<K, KB>`] which uses [`String`] as its keys.
pub type StringKeyStringMapEntry = <StringKeyStringMapContext as AbstractContext>::Entry;
/// The iterator of [`StringKeyStringMapContext`].
pub type StringKeyStringMapIter<'a> = <StringKeyStringMapContext as AbstractContext>::Iter<'a>;

/// A [`StringMapContext<K, KB>`] which uses `&'static str` as its keys.
pub type LiteralKeyStringMapContext = StringMapContext<&'static str, str>;
/// A [`StringMapEntry<K, KB>`] which uses `&'static str` as its keys.
pub type LiteralKeyStringMapEntry = <LiteralKeyStringMapContext as AbstractContext>::Entry;
/// The iterator of [`LiteralKeyStringMapContext`].
pub type LiteralKeyStringMapIter<'a> = <LiteralKeyStringMapContext as AbstractContext>::Iter<'a>;

/// A map entry whose value is a [`Box<DynAnyValue>`].
pub type AnyMapEntry<K, KB> = MapEntry<K, KB, Box<DynAnyValue>, DynAnyValue>;
/// A context whose entries are [`AnyMapEntry<K, KB>`].
pub type AnyMapContext<K, KB> = MapContext<AnyMapEntry<K, KB>, BoxConverter>;
/// The iterator of [`AnyMapContext<K, KB>`].
pub type AnyMapIter<'a, K, KB> = MapIter<'a, AnyMapEntry<K, KB>>;

/// A [`AnyMapContext<K, KB>`] which uses [`String`] as its keys.
pub type StringKeyAnyMapContext = AnyMapContext<String, str>;
/// A [`AnyMapEntry<K, KB>`] which uses [`String`] as its keys.
pub type StringKeyAnyMapEntry = <StringKeyAnyMapContext as AbstractContext>::Entry;
/// The iterator of [`StringKeyAnyMapContext`].
pub type StringKeyAnyMapIter<'a> = <StringKeyAnyMapContext as AbstractContext>::Iter<'a>;

/// A [`AnyMapContext<K, KB>`] which uses `&'static str` as its keys.
pub type LiteralKeyAnyMapContext = AnyMapContext<&'static str, str>;
/// A [`StringMapEntry<K, KB>`] which uses `&'static str` as its keys.
pub type LiteralKeyAnyMapEntry = <LiteralKeyAnyMapContext as AbstractContext>::Entry;
/// The iterator of [`LiteralKeyAnyMapContext`].
pub type LiteralKeyAnyMapIter<'a> = <LiteralKeyAnyMapContext as AbstractContext>::Iter<'a>;

impl<K, KB> Display for StringMapEntry<K, KB>
where
    K: Borrow<KB> + Debug + Send + Sync + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + Send + Sync,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} = {}", self.key(), self.value())
    }
}

impl<K, KB> StringContext for StringMapContext<K, KB>
where
    K: Borrow<KB> + Debug + Send + Sync + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + Send + Sync,
{
}

impl<K, KB> Display for AnyMapEntry<K, KB>
where
    K: Borrow<KB> + Debug + Send + Sync + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + Send + Sync,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} = {:?}", self.key(), self.value())
    }
}

impl<K, KB> AnyContext for AnyMapContext<K, KB>
where
    K: Borrow<KB> + Debug + Send + Sync + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + Send + Sync,
{
}

#[cfg(test)]
mod tests {
    use crate::context::Context;

    use super::*;

    type TestContext = AnyMapContext<&'static str, str>;

    #[test]
    fn any_map_context_operation() {
        let mut ctx = TestContext::new();
        ctx.insert_with::<BoxConverter, _, _>("i32", 1i32);
        ctx.insert_with::<BoxConverter, _, _>("string", "test");
        assert_eq!(ctx.value_as::<i32, _>("i32"), Some(&1i32));
        assert_eq!(ctx.value_as::<&str, _>("string"), Some(&"test"));
        assert_eq!(ctx.value_as::<(), _>("i32"), None);
        assert_eq!(ctx.value_as::<(), _>("string"), None);
    }
}
