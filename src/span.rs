//! Source spans: a byte range into the input plus a wrapper that pairs any
//! value with the position it came from in the YAML.

use marked_yaml::Span as MarkedSpan;

/// A half-open byte range `[start, end)` into the original YAML source.
///
/// Byte offsets are what `ariadne` consumes for its underlines, so we store
/// them directly rather than carrying line/column and converting on render.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Build a `Span` from a marked-yaml `Span`. When start or end are
    /// missing (the loader can leave them blank), fall back to zero — the
    /// rendered underline will still appear, just at the file's start.
    pub fn from_marked(span: &MarkedSpan) -> Self {
        let start = span.start().map(|marker| marker.character()).unwrap_or(0);
        let end = span.end().map(|marker| marker.character()).unwrap_or(start);
        // Marked spans occasionally come back with end < start; clamp so the
        // underline always covers at least one character.
        let end = end.max(start.saturating_add(1));
        Self { start, end }
    }
}

impl From<Span> for std::ops::Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

/// A value paired with the source span it was parsed from.
///
/// Identifier fields in the AST (table/column/alias names, type names) are
/// `Spanned<String>` so error messages can underline the offending token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}
