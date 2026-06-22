use crate::ast::join::{FromClause, Join};
use crate::ast::select::SelectItem;
use crate::span::{Span, Spanned};

/// A `CREATE VIEW … AS`: a saved query exposed like a table. Clause fields
/// after `select`/`from`/`join` are raw SQL, passed through verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    pub name: Spanned<String>,
    pub or_replace: bool,
    pub from: FromClause,
    pub join: Vec<Join>,
    pub select: Vec<SelectItem>,
    pub r#where: Option<String>,
    pub group_by: Option<String>,
    pub having: Option<String>,
    pub order_by: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub distinct_on: Option<Vec<String>>,
    pub span: Span,
}
