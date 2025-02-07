use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::Hash;

use crate::error::context::{AbstractContext, AnyContext, Entry, Sealed, StringContext};

use super::any::AnyValue;
use super::{MapContext, MapEntry, MapIter};

pub type StringMapEntry<K, KB> = MapEntry<K, KB, String, str>;
pub type StringMapContext<K, KB> = MapContext<StringMapEntry<K, KB>>;
pub type StringMapIter<'a, K, KB> = MapIter<'a, StringMapEntry<K, KB>>;

pub type StringKeyStringMapContext = StringMapContext<String, str>;
pub type StringKeyStringMapEntry = <StringKeyStringMapContext as AbstractContext>::Entry;
pub type StringKeyStringMapIter<'a> = <StringKeyStringMapContext as AbstractContext>::Iter<'a>;

pub type LiteralKeyStringMapContext = StringMapContext<&'static str, str>;
pub type LiteralKeyStringMapEntry = <LiteralKeyStringMapContext as AbstractContext>::Entry;
pub type LiteralKeyStringMapIter<'a> = <LiteralKeyStringMapContext as AbstractContext>::Iter<'a>;

pub type AnyMapEntry<K, KB> = MapEntry<
    K,
    KB,
    Box<dyn AnyValue + Send + Sync + 'static>,
    dyn AnyValue + Send + Sync + 'static,
>;
pub type AnyMapContext<K, KB> = MapContext<AnyMapEntry<K, KB>>;
pub type AnyMapIter<'a, K, KB> = MapIter<'a, AnyMapEntry<K, KB>>;

impl<K, KB> Display for StringMapEntry<K, KB>
where
    K: Borrow<KB> + Debug + Send + Sync + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + Send + Sync,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} = {}", self.key(), self.value())
    }
}

impl<K, KB> Sealed for StringMapContext<K, KB>
where
    K: Borrow<KB> + Debug + Send + Sync + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + Send + Sync,
{
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

impl<K, KB> Sealed for AnyMapContext<K, KB>
where
    K: Borrow<KB> + Debug + Send + Sync + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + Send + Sync,
{
}

impl<K, KB> AnyContext for AnyMapContext<K, KB>
where
    K: Borrow<KB> + Debug + Send + Sync + 'static,
    KB: Debug + Display + Eq + Hash + ?Sized + Send + Sync,
{
}
