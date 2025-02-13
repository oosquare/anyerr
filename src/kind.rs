use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::Hash;

/// The error kind used by [`AnyError`].
///
/// Each [`AnyError`] instance should have an error kind which represents the
/// basic category of the error. Though almost all Rust applications never use
/// dynamic error types to do anything but error reporting, an error kind
/// sometimes helps control different levels of logging, auditing and other
/// work in a more fine-grained way. If a [`String`]-only error reporting
/// mechanism is indeed your preference, you can just stick to the predefined
/// [`NoErrorKind`], which is typically for this usecase.
///
/// [`AnyError`]: `crate::core::AnyError`
pub trait Kind:
    Debug + Display + Clone + Copy + PartialEq + Eq + Hash + Default + Send + Sync + 'static
{
    /// The kind which indicates that the [`AnyError`] instance wraps an
    /// external error.
    ///
    /// [`AnyError`]: `crate::core::AnyError`
    const RAW_KIND: Self;

    /// The kind which indicates that the error kind is not specified.
    const UNKNOWN_KIND: Self;

    /// Returns true if the error kind is [`Kind::RAW_KIND`].
    fn is_raw(&self) -> bool {
        *self == Self::RAW_KIND
    }

    /// Returns true if the error kind is [`Kind::UNKNOWN_KIND`].
    fn is_unknown(&self) -> bool {
        *self == Self::UNKNOWN_KIND
    }
}

/// A predefined error kind based on the crate author's development experience.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum DefaultErrorKind {
    ValueValidation,
    RuleViolation,
    EntityAbsence,
    InfrastructureFailure,
    Raw,
    #[default]
    Unknown,
}

impl Display for DefaultErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let value = match self {
            Self::ValueValidation => "ValueValidation",
            Self::RuleViolation => "RuleViolation",
            Self::EntityAbsence => "EntityAbsence",
            Self::InfrastructureFailure => "InfrastructureFailure",
            Self::Raw => "Raw",
            Self::Unknown => "Unknown",
        };
        write!(f, "{value}")
    }
}

impl Kind for DefaultErrorKind {
    const RAW_KIND: Self = DefaultErrorKind::Raw;

    const UNKNOWN_KIND: Self = DefaultErrorKind::Unknown;
}

/// A predefined error kind that is used when no error kind is needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NoErrorKind {
    #[default]
    Anything,
}

impl Display for NoErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Anything")
    }
}

impl Kind for NoErrorKind {
    const RAW_KIND: Self = Self::Anything;

    const UNKNOWN_KIND: Self = Self::Anything;
}
