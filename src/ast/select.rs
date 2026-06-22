use crate::span::{Span, Spanned};

/// One item in a view's `select:` list. The variant is chosen during lowering
/// by which keys are present (`column` vs `sql`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectItem {
    Column(SelectColumn),
    Expression(SelectExpression),
}

/// A plain column pulled from a `from:`/`join:` alias.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectColumn {
    pub column: Spanned<String>,
    /// The alias the column comes from; resolved in a later phase.
    pub from: Option<Spanned<String>>,
    /// Output name; defaults to the column name when absent.
    pub r#as: Option<String>,
    /// Whether queries may filter on this column.
    pub filterable: bool,
    /// Whether free-text search includes this column.
    pub searchable: bool,
    pub span: Span,
}

/// A raw SQL expression with a declared output name (escape hatch).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectExpression {
    pub sql: String,
    pub r#as: String,
    pub span: Span,
}
