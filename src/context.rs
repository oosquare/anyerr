pub mod iter;
pub mod map;
pub mod singleton;
pub mod unit;

use std::any::Any;
use std::borrow::Borrow;
use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::converter::{Convertable, Converter};

pub use map::{LiteralKeyStringMapContext, StringKeyStringMapContext};
pub use map::{LiteralKeyAnyMapContext, StringKeyAnyMapContext};
pub use map::{AnyValue, DynAnyValue};
pub use singleton::{AnySingletonContext, FixedSingletonContext, StringSingletonContext};
pub use unit::UnitContext;

/// The most fundamental trait of all context storage.
///
/// The [`AbstractContext`] trait requires the structure of each context to be
/// a map, and allows iteration over its entries.
pub trait AbstractContext: Default + Debug + Send + Sync + 'static {
    /// The key of each entry.
    type Key;

    /// The value of each entry.
    type Value;

    /// The entry stored in this context.
    type Entry: Entry<Key = Self::Key, Value = Self::Value>;

    /// The iterator over this context's entries.
    type Iter<'a>: Iter<'a, Entry = Self::Entry>
    where
        Self: 'a;

    /// Returns the context's iterator.
    fn iter(&self) -> Self::Iter<'_>;
}

/// The context that stores nothing.
///
/// This kind of contexts fit very well in the circumstance where you will
/// never employ the context information in key-value pairs, by getting rid of
/// almost all useless relavent methods and the memory overhead with which
/// other context storages may bring.
///
/// For contexts implementing this trait, refer to the [`crate::context::unit`]
/// module.
///
/// # Example
///
/// ```rust
/// # use anyerr::context::{AbstractContext, UnitContext};
/// // `UnitContext` implements `NoContext`.
/// let context = UnitContext;
/// let mut iter = context.iter();
/// assert!(iter.next().is_none());
/// ```
#[allow(private_bounds)]
pub trait NoContext: AbstractContext {}

/// The context that is insertable.
///
/// Types that implements the [`Context`] trait allow insertions to the internal
/// storage and queries by keys. Conversions before insertions are also
/// supported.
pub trait Context: AbstractContext {
    /// The default converter used by the context, which transforms values to
    /// [`AbstractContext::Value`] before insertions.
    type Converter: Converter;

    /// Inserts the context information represented as a key-value pair. Any
    /// types that are compatible with [`AbstractContext::Key`] and
    /// [`AbstractContext::Value`] through the [`Into`] trait respectively are
    /// expected.
    fn insert<Q, R>(&mut self, key: Q, value: R)
    where
        Q: Into<Self::Key>,
        R: Into<Self::Value>;

    /// Converts the context information represented as a key-value pair using
    /// the specified converter `C` and then inserts the converted pair. Any
    /// types that are compatible with [`AbstractContext::Key`] and
    /// [`AbstractContext::Value`] through the [`Into`] and [`Convertable`]
    /// trait respectively are expected.
    fn insert_with<C, Q, R>(&mut self, key: Q, value: R)
    where
        Q: Into<Self::Key>,
        C: Converter,
        R: Convertable<C, Self::Value>,
    {
        self.insert(key, value.to());
    }

    /// Returns the value corresponding to the given key.
    fn get<Q>(&self, key: &Q) -> Option<&<Self::Entry as Entry>::ValueBorrowed>
    where
        <Self::Entry as Entry>::KeyBorrowed: Borrow<Q>,
        Q: Debug + Eq + Hash + ?Sized;
}

/// The insertable context which holds at most one entry.
///
/// Accessing the entry it contains can be done with the method offered by the
/// [`SingletonContext`] trait, without specifying the key.
///
/// For contexts implementing this trait, refer to the
/// [`crate::context::singleton`] module.
///
/// # Example
///
/// ```rust
/// # use anyerr::context::{Context, SingletonContext, StringSingletonContext};
/// // `StringSingletonContext` implements `SingletonContext`.
/// let mut context = StringSingletonContext::new();
/// assert_eq!(context.value(), None);
/// context.insert((), "context");
/// assert_eq!(context.value(), Some("context"));
/// context.insert((), "context2");
/// assert_eq!(context.value(), Some("context2"));
/// ```
pub trait SingletonContext: Context {
    /// Returns the value of the only entry in the context if it exists.
    fn value(&self) -> Option<&<Self::Entry as Entry>::ValueBorrowed>;
}

/// The context where each entry's value is a [`String`].
///
/// For contexts implementing this trait, refer to the [`crate::context::map`]
/// module.
///
/// # Example
///
/// ```rust
/// # use anyerr::context::{Context, StringContext, LiteralKeyStringMapContext};
/// # use anyerr::converter::DebugConverter;
/// // `LiteralKeyStringMapContext` implements `StringContext`.
/// let mut context = LiteralKeyStringMapContext::new();
/// context.insert_with::<DebugConverter, _, _>("i32", 42);
/// context.insert_with::<DebugConverter, _, _>("&str", "context");
/// assert_eq!(context.get("i32"), Some("42"));
/// assert_eq!(context.get("&str"), Some("\"context\""));
/// ```
pub trait StringContext
where
    Self: Context<Value = String, Entry: Entry<ValueBorrowed = str>>,
{
}

/// The context where each entry's value is a [`Box<DynAnyValue>`].
///
/// Accessing the entry it contains and conveniently casting the result to a
/// concrete type can be done with the method offered by the [`AnyContext`]
/// trait.
///
/// For contexts implementing this trait, refer to the [`crate::context::map`]
/// module.
///
/// # Example
///
/// ```rust
/// # use anyerr::context::{AnyContext, Context, LiteralKeyAnyMapContext};
/// # use anyerr::converter::BoxConverter;
/// // `LiteralKeyAnyMapContext` implements `AnyContext`.
/// let mut context = LiteralKeyAnyMapContext::new();
/// context.insert_with::<BoxConverter, _, _>("error-code", 42i32);
/// context.insert_with::<BoxConverter, _, _>("cause", "unknown");
/// assert_eq!(context.value_as::<i32, _>("error-code"), Some(&42i32));
/// assert_eq!(context.value_as::<&str, _>("cause"), Some(&"unknown"));
/// ```
pub trait AnyContext
where
    Self: Context<Value = Box<DynAnyValue>, Entry: Entry<ValueBorrowed = DynAnyValue>>,
{
    /// Returns the value corresponding to the given key and tries to cast it
    /// to the type `T`. Returns `None` if the entry doesn't exist or the
    /// downcasting fails.
    fn value_as<T, Q>(&self, key: &Q) -> Option<&T>
    where
        <Self::Entry as Entry>::KeyBorrowed: Borrow<Q>,
        Q: Debug + Eq + Hash + ?Sized,
        T: Any,
    {
        self.get(key).and_then(|value| value.downcast_ref::<T>())
    }
}

/// The common representation of entries in different kinds of contexts.
pub trait Entry: Debug + Send + Sync + 'static {
    /// The identifier used to indexing an entry.
    type Key: Borrow<Self::KeyBorrowed> + Debug + Send + Sync + 'static;

    /// The borrowed type of [`Entry::Key`].
    type KeyBorrowed: Debug + Display + Eq + Hash + ?Sized + Send + Sync + 'static;

    /// The value corresponding to the key.
    type Value: Borrow<Self::ValueBorrowed> + Debug + Send + Sync + 'static;

    /// The borrowed type of [`Entry::Value`].
    type ValueBorrowed: Debug + ?Sized + Send + Sync + 'static;

    /// Creates a new entry with the given key and value.
    fn new<Q, R>(key: Q, value: R) -> Self
    where
        Q: Into<Self::Key>,
        R: Into<Self::Value>;

    /// Returns the entry's key.
    fn key(&self) -> &Self::KeyBorrowed;

    /// Returns the entry's value.
    fn value(&self) -> &Self::ValueBorrowed;
}

/// A dedicated iterator of the context storage.
pub trait Iter<'a>: Default + Iterator<Item = &'a Self::Entry> {
    /// The entry whose reference will be yielded by this iterator.
    type Entry: 'a;

    /// Combines two iterator into a new iterator, which will iterate over the
    /// union of two sets of entries.
    fn compose(self, other: Self) -> Self;
}
