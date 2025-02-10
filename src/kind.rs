use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::Hash;

pub trait Kind:
    Debug + Display + Clone + Copy + PartialEq + Eq + Hash + Default + Send + Sync + 'static
{
    const RAW_KIND: Self;

    const UNKNOWN_KIND: Self;

    fn is_raw(&self) -> bool {
        *self == Self::RAW_KIND
    }

    fn is_unknown(&self) -> bool {
        *self == Self::UNKNOWN_KIND
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum DefaultAnyErrorKind {
    ValueValidation,
    RuleViolation,
    EntityAbsence,
    InfrastructureFailure,
    Raw,
    #[default]
    Unknown,
}

impl Display for DefaultAnyErrorKind {
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

impl Kind for DefaultAnyErrorKind {
    const RAW_KIND: Self = DefaultAnyErrorKind::Raw;

    const UNKNOWN_KIND: Self = DefaultAnyErrorKind::Unknown;
}
