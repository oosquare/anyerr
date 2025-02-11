use crate::context::singleton::{OptionContext, OptionEntry};
use crate::context::{AbstractContext, DynAnyValue};

pub type StringSingletonEntry = OptionEntry<String, str>;
pub type StringSingletonContext = OptionContext<StringSingletonEntry>;
pub type StringSingletonIter<'a> = <StringSingletonContext as AbstractContext>::Iter<'a>;

pub type AnySingletonEntry = OptionEntry<Box<DynAnyValue>, DynAnyValue>;
pub type AnySingletonContext = OptionContext<AnySingletonEntry>;
pub type AnySingletonIter<'a> = <AnySingletonContext as AbstractContext>::Iter<'a>;

pub type FixedSingletonEntry<T> = OptionEntry<T, T>;
pub type FixedSingletonContext<T> = OptionContext<FixedSingletonEntry<T>>;
pub type FixedSingletonIter<'a, T> = <FixedSingletonContext<T> as AbstractContext>::Iter<'a>;
