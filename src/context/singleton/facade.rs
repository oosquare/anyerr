use crate::context::singleton::{OptionContext, OptionEntry};
use crate::context::{AbstractContext, DynAnyValue};

/// A singleton entry whose value is a [`String`].
pub type StringSingletonEntry = OptionEntry<String, str>;
/// A context whose entry is [`StringSingletonEntry`].
pub type StringSingletonContext = OptionContext<StringSingletonEntry>;
/// The iterator of [`StringSingletonContext`].
pub type StringSingletonIter<'a> = <StringSingletonContext as AbstractContext>::Iter<'a>;

/// A singleton entry whose value is a [`Box<DynAnyValue>`].
pub type AnySingletonEntry = OptionEntry<Box<DynAnyValue>, DynAnyValue>;
/// A context whose entry is [`AnySingletonEntry`].
pub type AnySingletonContext = OptionContext<AnySingletonEntry>;
/// The iterator of [`AnySingletonContext`].
pub type AnySingletonIter<'a> = <AnySingletonContext as AbstractContext>::Iter<'a>;

/// A singleton entry whose value is a fixed generic type `T`.
pub type FixedSingletonEntry<T> = OptionEntry<T, T>;
/// A context whose entry is [`FixedSingletonEntry<T>`].
pub type FixedSingletonContext<T> = OptionContext<FixedSingletonEntry<T>>;
/// The iterator of [`FixedSingletonContext<T>`].
pub type FixedSingletonIter<'a, T> = <FixedSingletonContext<T> as AbstractContext>::Iter<'a>;
