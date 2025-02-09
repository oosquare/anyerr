use std::fmt::Debug;

use crate::error::context::{Entry, Iter};

#[derive(Debug)]
pub enum CommonIter<'a, E, I>
where
    E: Entry + 'a,
    I: Iterator<Item = &'a E>,
{
    Node {
        iter: I,
        next: Option<Box<CommonIter<'a, E, I>>>,
    },
    None,
}

impl<'a, E, I> CommonIter<'a, E, I>
where
    E: Entry + 'a,
    I: Iterator<Item = &'a E>,
{
    pub fn new() -> Self {
        Self::None
    }

    fn append(&mut self, other: Self) {
        if matches!(other, Self::None) {
            return;
        } else if let Self::Node { next, .. } = self {
            if let Some(next) = next {
                next.append(other);
            } else {
                *next = Some(Box::new(other));
            }
        } else {
            *self = other;
        }
    }
}

impl<'a, E, I> Default for CommonIter<'a, E, I>
where
    E: Entry + 'a,
    I: Iterator<Item = &'a E>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, E, I> From<I> for CommonIter<'a, E, I>
where
    E: Entry + 'a,
    I: Iterator<Item = &'a E>,
{
    fn from(iter: I) -> Self {
        Self::Node { iter, next: None }
    }
}

impl<'a, E, I> Iterator for CommonIter<'a, E, I>
where
    E: Entry + 'a,
    I: Iterator<Item = &'a E>,
{
    type Item = &'a E;

    fn next(&mut self) -> Option<Self::Item> {
        while let Self::Node { iter, next, .. } = self {
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

impl<'a, E, I> Iter<'a> for CommonIter<'a, E, I>
where
    E: Entry + 'a,
    I: Iterator<Item = &'a E>,
{
    type Entry = E;

    fn compose(mut self, other: Self) -> Self {
        self.append(other);
        self
    }
}
