use indexmap::IndexMap;

use crate::span::{Span, Spanned};

/// A `CREATE TABLE`: physical storage. Columns are an ordered map so emitted
/// DDL is deterministic (insertion order is preserved).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub name: Spanned<String>,
    pub if_not_exists: bool,
    pub columns: IndexMap<String, Column>,
    pub primary_key: Option<PrimaryKey>,
    pub indexes: Vec<Index>,
    pub span: Span,
}

/// One column definition: a type, plus nullability and an optional default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    /// Built-in type name or a declared enum name; resolved in a later phase.
    /// Spanned so `UnknownType` errors underline the exact token.
    pub r#type: Spanned<String>,
    /// `true` permits NULL; columns are NOT NULL by default.
    pub null: bool,
    /// Raw SQL default expression, passed through verbatim.
    pub default: Option<String>,
    pub span: Span,
}

/// The column(s) that uniquely identify a row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimaryKey {
    pub name: Option<String>,
    pub columns: Vec<String>,
    pub span: Span,
}

/// A `CREATE INDEX`: a lookup shortcut; affects performance, never results.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    /// Index method (`btree`, `gin`, …); the engine's default when absent.
    pub using: Option<String>,
    /// Partial-index predicate, raw SQL, passed through verbatim.
    pub r#where: Option<String>,
    pub span: Span,
}
