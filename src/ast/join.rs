use crate::span::{Span, Spanned};

/// A view's `from:` — the source relation and the alias the rest of the view uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FromClause {
    pub table: Spanned<String>,
    pub r#as: Spanned<String>,
    pub span: Span,
}

/// A `JOIN … ON`: combine rows from another relation by a matching rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Join {
    pub table: Spanned<String>,
    pub r#as: Spanned<String>,
    /// Join kind; a bare join is `INNER`, matching SQL.
    pub r#type: JoinKind,
    /// The match condition, raw SQL, passed through verbatim (escape hatch).
    pub on: String,
    pub span: Span,
}

/// Which kind of join. `INNER` keeps only matches; `LEFT` keeps all left rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JoinKind {
    #[default]
    Inner,
    Left,
    Right,
    Full,
}
