use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

use crate::context::AbstractContext;
use crate::core::ContextDepth;
use crate::kind::Kind;
use crate::AnyError;

pub struct ReportInner<C, K>
where
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    error: AnyError<C, K>,
    pretty: bool,
    kind: bool,
    backtrace: bool,
    context: bool,
}

impl<C, K> ReportInner<C, K>
where
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    pub fn pretty(self, pretty: bool) -> Self {
        Self { pretty, ..self }
    }

    pub fn kind(self, kind: bool) -> Self {
        Self { kind, ..self }
    }

    pub fn backtrace(self, backtrace: bool) -> Self {
        Self { backtrace, ..self }
    }

    pub fn context(self, context: bool) -> Self {
        Self { context, ..self }
    }

    pub fn render(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.pretty {
            self.render_pretty_report(f)
        } else {
            self.render_compact_report(f)
        }
    }

    fn render_pretty_report(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.render_single_pretty_error(f, "Error:", &self.error)?;

        let mut source = self.error.source();
        loop {
            let error = source.and_then(|error| error.downcast_ref::<AnyError<C, K>>());
            if let Some(error) = error {
                self.render_single_pretty_error(f, "Caused by:", error)?;
                source = error.source();
            } else {
                break;
            }
        }

        self.render_backtrace(f)?;
        Ok(())
    }

    fn render_single_pretty_error(
        &self,
        f: &mut Formatter<'_>,
        prefix: &str,
        error: &AnyError<C, K>,
    ) -> FmtResult {
        writeln!(f, "{}", prefix)?;
        self.render_pretty_single_error_message(f, error)?;
        self.render_pretty_one_line_context(f, error)?;
        Ok(())
    }

    fn render_pretty_single_error_message(
        &self,
        f: &mut Formatter<'_>,
        error: &AnyError<C, K>,
    ) -> FmtResult {
        write!(f, "    ")?;
        if self.kind {
            writeln!(f, "({}) {}", error.kind(), error)
        } else {
            writeln!(f, "{}", error)
        }
    }

    fn render_pretty_one_line_context(
        &self,
        f: &mut Formatter<'_>,
        error: &AnyError<C, K>,
    ) -> FmtResult {
        if !self.context {
            return Ok(());
        }
        let mut context = error.context(ContextDepth::Shallowest).peekable();
        if context.peek().is_none() {
            return Ok(());
        }
        write!(f, "    [")?;
        let mut first = true;
        for entry in context {
            if first {
                write!(f, "{entry}")?;
                first = false;
            } else {
                write!(f, ", {entry}")?;
            }
        }
        writeln!(f, "]")?;
        Ok(())
    }

    fn render_backtrace(&self, f: &mut Formatter<'_>) -> FmtResult {
        if !self.backtrace {
            return Ok(());
        }

        writeln!(f)?;
        writeln!(f, "Stack backtrace:")?;
        writeln!(f, "{}", self.error.backtrace())?;
        Ok(())
    }

    fn render_compact_report(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.render_one_line_message(f)?;
        self.render_compact_one_line_context(f, &self.error)?;
        Ok(())
    }

    fn render_one_line_message(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.render_compact_single_error_message(f, &self.error)?;
        let mut source = self.error.source();
        loop {
            let error = source.and_then(|error| error.downcast_ref::<AnyError<C, K>>());
            if let Some(error) = error {
                write!(f, ": ")?;
                self.render_compact_single_error_message(f, error)?;
                source = error.source();
            } else {
                break;
            }
        }
        Ok(())
    }

    fn render_compact_one_line_context(
        &self,
        f: &mut Formatter<'_>,
        error: &AnyError<C, K>,
    ) -> FmtResult {
        if !self.context {
            return Ok(());
        }
        let mut context = error.context(ContextDepth::All).peekable();
        if context.peek().is_none() {
            return Ok(());
        }
        write!(f, " ")?;
        write!(f, "[")?;
        let mut first = true;
        for entry in context {
            if first {
                write!(f, "{entry}")?;
                first = false;
            } else {
                write!(f, ", {entry}")?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }

    fn render_compact_single_error_message(
        &self,
        f: &mut Formatter<'_>,
        error: &AnyError<C, K>,
    ) -> FmtResult {
        if self.kind {
            write!(f, "({}) {}", error.kind(), error)
        } else {
            write!(f, "{}", error)
        }
    }
}

impl<C, K> From<AnyError<C, K>> for ReportInner<C, K>
where
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    fn from(error: AnyError<C, K>) -> Self {
        Self {
            error,
            pretty: true,
            kind: true,
            backtrace: true,
            context: true,
        }
    }
}

impl<C, K> From<ReportInner<C, K>> for AnyError<C, K>
where
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    fn from(report: ReportInner<C, K>) -> Self {
        report.error
    }
}

impl<C, K> Debug for ReportInner<C, K>
where
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.render(f)
    }
}

impl<C, K> Display for ReportInner<C, K>
where
    C: AbstractContext<Entry: Display>,
    K: Kind,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.render(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::context::StringKeyStringMapContext;
    use crate::kind::DefaultErrorKind as ErrKind;
    use crate::{Intermediate, Overlay};

    use super::*;

    type TestError = AnyError<StringKeyStringMapContext, ErrKind>;

    #[test]
    fn report_inner_display_succeeds_when_pretty_is_true() {
        let report = ReportInner::from(new_test_error())
            .pretty(true)
            .kind(true)
            .backtrace(false)
            .context(true);
        let mut expected = String::new();
        expected.push_str("Error:\n");
        expected.push_str("    (Unknown) error3\n");
        expected.push_str("    [key3.1 = \"value\", key3.2 = \"value\"]\n");
        expected.push_str("Caused by:\n");
        expected.push_str("    (RuleViolation) error2\n");
        expected.push_str("    [key2.1 = \"value\"]\n");
        expected.push_str("Caused by:\n");
        expected.push_str("    (ValueValidation) error1\n");
        assert_eq!(report.to_string(), expected);

        let report = ReportInner::from(new_test_error())
            .pretty(true)
            .kind(false)
            .backtrace(false)
            .context(true);
        let mut expected = String::new();
        expected.push_str("Error:\n");
        expected.push_str("    error3\n");
        expected.push_str("    [key3.1 = \"value\", key3.2 = \"value\"]\n");
        expected.push_str("Caused by:\n");
        expected.push_str("    error2\n");
        expected.push_str("    [key2.1 = \"value\"]\n");
        expected.push_str("Caused by:\n");
        expected.push_str("    error1\n");
        assert_eq!(report.to_string(), expected);

        let report = ReportInner::from(new_test_error())
            .pretty(true)
            .kind(true)
            .backtrace(false)
            .context(false);
        let mut expected = String::new();
        expected.push_str("Error:\n");
        expected.push_str("    (Unknown) error3\n");
        expected.push_str("Caused by:\n");
        expected.push_str("    (RuleViolation) error2\n");
        expected.push_str("Caused by:\n");
        expected.push_str("    (ValueValidation) error1\n");
        assert_eq!(report.to_string(), expected);
    }

    #[test]
    fn report_inner_display_succeeds_when_pretty_is_false() {
        let report = ReportInner::from(new_test_error()).pretty(false);
        assert_eq!(report.to_string(), "(Unknown) error3: (RuleViolation) error2: (ValueValidation) error1 [key3.1 = \"value\", key3.2 = \"value\", key2.1 = \"value\"]");

        let report = ReportInner::from(new_test_error())
            .pretty(false)
            .kind(false);
        assert_eq!(
            report.to_string(),
            "error3: error2: error1 [key3.1 = \"value\", key3.2 = \"value\", key2.1 = \"value\"]"
        );

        let report = ReportInner::from(new_test_error())
            .pretty(false)
            .context(false);
        assert_eq!(
            report.to_string(),
            "(Unknown) error3: (RuleViolation) error2: (ValueValidation) error1"
        );
    }

    fn new_test_error() -> TestError {
        let error1 = TestError::quick("error1", ErrKind::ValueValidation);
        let error2 = error1
            .overlay(("error2", ErrKind::RuleViolation))
            .context("key2.1", "value")
            .build();
        let error3 = error2
            .overlay("error3")
            .context("key3.1", "value")
            .context("key3.2", "value")
            .build();
        error3
    }
}
