mod inner;

use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::process::{ExitCode, Termination};

use crate::context::AbstractContext;
use crate::kind::Kind;
use crate::AnyError;

use inner::ReportInner;

pub struct Report<T, C, K>(ReportVariant<T, C, K>)
where
    T: Termination,
    C: AbstractContext<Entry: Display>,
    K: Kind;

impl<T, C, K> Report<T, C, K>
where
    T: Termination,
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    pub fn capture(func: impl FnOnce() -> Result<T, AnyError<C, K>>) -> Self {
        match func() {
            Ok(val) => Self(ReportVariant::Success(val)),
            Err(err) => ReportInner::from(err).into(),
        }
    }

    pub fn pretty(self, pretty: bool) -> Self {
        match self.0 {
            v @ ReportVariant::Success(_) => Self(v),
            ReportVariant::Failure(report) => report.pretty(pretty).into(),
        }
    }

    pub fn kind(self, kind: bool) -> Self {
        match self.0 {
            v @ ReportVariant::Success(_) => Self(v),
            ReportVariant::Failure(report) => report.kind(kind).into(),
        }
    }

    pub fn backtrace(self, backtrace: bool) -> Self {
        match self.0 {
            v @ ReportVariant::Success(_) => Self(v),
            ReportVariant::Failure(report) => report.backtrace(backtrace).into(),
        }
    }

    pub fn context(self, context: bool) -> Self {
        match self.0 {
            v @ ReportVariant::Success(_) => Self(v),
            ReportVariant::Failure(report) => report.context(context).into(),
        }
    }

    fn render(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.0 {
            ReportVariant::Failure(report) => report.render(f),
            _ => Ok(()),
        }
    }
}

impl<T, C, K> From<ReportInner<C, K>> for Report<T, C, K>
where
    T: Termination,
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    fn from(value: ReportInner<C, K>) -> Self {
        Self(ReportVariant::Failure(value))
    }
}

impl<T, C, K> Termination for Report<T, C, K>
where
    T: Termination,
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    fn report(self) -> ExitCode {
        match self.0 {
            ReportVariant::Success(val) => val.report(),
            ReportVariant::Failure(report) => {
                eprintln!("{report}");
                ExitCode::FAILURE
            }
        }
    }
}

impl<T, C, K> Debug for Report<T, C, K>
where
    T: Termination,
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.render(f)
    }
}

impl<T, C, K> Display for Report<T, C, K>
where
    T: Termination,
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.render(f)
    }
}

enum ReportVariant<T, C, K>
where
    T: Termination,
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    Success(T),
    Failure(ReportInner<C, K>),
}
