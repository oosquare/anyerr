mod inner;

use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::process::{ExitCode, Termination};

use crate::context::AbstractContext;
use crate::kind::Kind;
use crate::AnyError;

use inner::ReportInner;

/// An error reporter which displays data carried by an [`AnyError`].
///
/// [`Report`] captures your function's result, such as [`Result<(), AnyError>`],
/// and displays the error message and other information if an error occurred.
/// It can also be used as `main()`'s returned value, handling the process's
/// termination by implementing the [`Termination`] trait.
pub struct Report<C, K>(ReportVariant<C, K>)
where
    C: AbstractContext<Entry: Display>,
    K: Kind;

impl<C, K> Report<C, K>
where
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    /// Creates a [`Report`] with the given error inside.
    pub fn wrap(error: AnyError<C, K>) -> Self {
        error.into()
    }

    /// Captures the result of a given function and creates a [`Report`].
    ///
    /// # Example
    ///
    /// ```no_run,rust
    /// # use std::process::Termination;
    /// # use anyerr::AnyError as AnyErrorTemplate;
    /// # use anyerr::kind::DefaultErrorKind;
    /// # use anyerr::context::LiteralKeyStringMapContext;
    /// # use anyerr::report::Report;
    /// type AnyError = AnyErrorTemplate<LiteralKeyStringMapContext, DefaultErrorKind>;
    ///
    /// fn main() -> impl Termination {
    ///     Report::capture(|| -> Result<(), AnyError> {
    ///         Err(AnyError::minimal("an error occurred"))
    ///     })
    /// }
    /// ```
    pub fn capture(func: impl FnOnce() -> Result<(), AnyError<C, K>>) -> Self {
        match func() {
            Ok(_) => Self(ReportVariant::Success),
            Err(err) => err.into(),
        }
    }

    /// Prints a pretty error report if `pretty` is `true`, otherwise prints
    /// a compact error report in one line.
    pub fn pretty(self, pretty: bool) -> Self {
        match self.0 {
            ReportVariant::Failure(report) => report.pretty(pretty).into(),
            v => Self(v),
        }
    }

    /// Prints error kinds if `kind` is `true`.
    pub fn kind(self, kind: bool) -> Self {
        match self.0 {
            ReportVariant::Failure(report) => report.kind(kind).into(),
            v => Self(v),
        }
    }

    /// Prints the backtrace if `backtrace` is `true`.
    pub fn backtrace(self, backtrace: bool) -> Self {
        match self.0 {
            ReportVariant::Failure(report) => report.backtrace(backtrace).into(),
            v => Self(v),
        }
    }

    /// Prints the attached context if `context` is `true`.
    pub fn context(self, context: bool) -> Self {
        match self.0 {
            ReportVariant::Failure(report) => report.context(context).into(),
            v => Self(v),
        }
    }

    fn render(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.0 {
            ReportVariant::Failure(report) => report.render(f),
            _ => Ok(()),
        }
    }
}

impl<C, K> From<ReportInner<C, K>> for Report<C, K>
where
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    fn from(value: ReportInner<C, K>) -> Self {
        Self(ReportVariant::Failure(value))
    }
}

impl<C, K> From<AnyError<C, K>> for Report<C, K>
where
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    fn from(error: AnyError<C, K>) -> Self {
        Self(ReportVariant::Failure(ReportInner::from(error)))
    }
}

impl<C, K> Termination for Report<C, K>
where
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    fn report(self) -> ExitCode {
        match self.0 {
            ReportVariant::Success => ExitCode::SUCCESS,
            ReportVariant::Failure(report) => {
                eprintln!("{report}");
                ExitCode::FAILURE
            }
        }
    }
}

impl<C, K> Debug for Report<C, K>
where
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.render(f)
    }
}

impl<C, K> Display for Report<C, K>
where
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.render(f)
    }
}

enum ReportVariant<C, K>
where
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    Success,
    Failure(ReportInner<C, K>),
}
