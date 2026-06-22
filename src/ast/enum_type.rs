use crate::span::{Span, Spanned};

/// A `CREATE TYPE … AS ENUM`: a named, fixed set of allowed string values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumType {
    pub name: Spanned<String>,
    pub r#enum: Vec<Spanned<String>>,
    pub span: Span,
}
