use crate::ast::enum_type::EnumType;
use crate::ast::table::Table;
use crate::ast::view::View;

/// A whole parsed schema file: the top-level `types`/`tables`/`views` lists.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchemaDocument {
    pub types: Vec<EnumType>,
    pub tables: Vec<Table>,
    pub views: Vec<View>,
}
