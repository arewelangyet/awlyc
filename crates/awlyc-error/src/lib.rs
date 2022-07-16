use ariadne::{sources, Color, Label, Report, ReportKind};
use smol_str::SmolStr;
use std::fmt;
use text_size::TextRange;

/// FileId is a unique identifier for a file
/// These are necessary for imports
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileId(pub SmolStr);

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Span is the location type for all diagnostics
/// It describes what file something is in, and where in that file it is
#[derive(Debug, PartialEq, Clone)]
pub struct Span {
    /// Range of characters the span covers
    pub range: TextRange,
    /// File the span is in
    pub file_id: FileId,
}

impl Span {
    /// Combine two spans in the same file
    /// a must come before b
    pub fn combine(a: &Self, b: &Self) -> Self {
        assert!(a.range.start() < b.range.end());
        Span {
            range: TextRange::new(a.range.start(), b.range.end()),
            file_id: a.file_id.clone(),
        }
    }
}

impl ariadne::Span for Span {
    type SourceId = FileId;

    fn start(&self) -> usize {
        self.range.start().into()
    }

    fn end(&self) -> usize {
        self.range.end().into()
    }

    fn len(&self) -> usize {
        self.range.len().into()
    }

    fn source(&self) -> &Self::SourceId {
        &self.file_id
    }

    fn contains(&self, offset: usize) -> bool {
        self.start() <= offset && self.end() > offset
    }
}

/// Severity of diagnostic
#[derive(Debug, PartialEq)]
pub enum DiagnosticKind {
    Error,
    Warning,
    Note,
}

/// Diagnostic represents any information the compiler has to tell the user about the input
/// They can be errors/warnings/notes
#[derive(Debug, PartialEq)]
pub struct Diagnostic {
    /// Severity of the diagnostic
    pub kind: DiagnosticKind,
    /// Content of the diagnostic
    pub msg: String,
    /// Where the diagnostic occurs
    pub span: Span,
}

impl Diagnostic {
    pub fn to_report(&self) -> Report<Span> {
        let report = Report::build(
            ReportKind::Error,
            self.span.file_id.clone(),
            self.span.range.start().into(),
        )
        .with_message(self.msg.clone())
        .with_label(
            Label::new(self.span.clone())
                .with_message(self.msg.clone())
                .with_color(Color::Blue),
        );
        report.finish()
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "err")
    }
}

impl std::error::Error for Diagnostic {}

// TODO: huh?
impl serde::de::Error for Diagnostic {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Diagnostic {
            kind: DiagnosticKind::Error,
            msg: msg.to_string(),
            span: Span {
                range: TextRange::new(0.into(), 0.into()),
                file_id: FileId(SmolStr::from("err")),
            },
        }
    }
}

pub struct DiagnosticReporter {
    pub files: Vec<(FileId, String)>,
}

impl DiagnosticReporter {
    pub fn add_file(&mut self, name: SmolStr, src: String) -> FileId {
        self.files.push((FileId(name.clone()), src));
        FileId(name)
    }

    pub fn report(&self, err: &Diagnostic) {
        err.to_report().print(sources(self.files.clone())).unwrap();
    }
}
