use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::Hash;

use crate::context::map::{DynAnyValue, MapContext, MapEntry, MapIter};
use crate::context::{AbstractContext, AnyContext, Entry, Sealed, StringContext};
use crate::converter::{BoxConverter, DebugConverter};

pub type StringMapEntry<K, KB> = MapEntry<K, KB, String, str>;
pub type StringMapContext<K, KB> = MapContext<StringMapEntry<K, KB>, DebugConverter>;
pub type StringMapIter<'a, K, KB> = MapIter<'a, StringMapEntry<K, KB>>;

pub type StringKeyStringMapContext = StringMapContext<String, str>;
pub type StringKeyStringMapEntry = <StringKeyStringMapContext as AbstractContext>::Entry;
pub type StringKeyStringMapIter<'a> = <StringKeyStringMapContext as AbstractContext>::Iter<'a>;

pub type LiteralKeyStringMapContext = StringMapContext<&'static str, str>;
pub type LiteralKeyStringMapEntry = <LiteralKeyStringMapContext as AbstractContext>::Entry;
pub type LiteralKeyStringMapIter<'a> = <LiteralKeyStringMapContext as AbstractContext>::Iter<'a>;

pub type AnyMapEntry<K, KB> = MapEntry<K, KB, Box<DynAnyValue>, DynAnyValue>;
pub type AnyMapContext<K, KB> = MapContext<AnyMapEntry<K, KB>, BoxConverter>;
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
