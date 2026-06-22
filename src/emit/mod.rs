//! The emit phase: render a validated schema document into PostgreSQL DDL.

mod enum_type;
mod table;
mod view;

use crate::ast::SchemaDocument;

/// Render a schema document as PostgreSQL DDL.
///
/// Assumes the document has already been validated upstream (parse →
/// build_catalog → resolve_views); emission itself is infallible — any
/// references that would have failed are caught before we reach here.
///
/// Statements are emitted in declaration order: types, then tables, then their
/// indexes, then views.
pub fn emit_schema(document: &SchemaDocument) -> String {
    let mut output = String::new();
    for declared in &document.types {
        enum_type::emit(&mut output, declared);
    }
    for one_table in &document.tables {
        table::emit_table(&mut output, one_table);
    }
    // Indexes after every table is created, so they reference real tables.
    for one_table in &document.tables {
        for index in &one_table.indexes {
            table::emit_index(&mut output, &one_table.name.value, index);
        }
    }
    for one_view in &document.views {
        view::emit(&mut output, one_view);
    }
    output
}

/// Quote a SQL identifier, doubling internal double quotes (the Postgres rule).
pub(crate) fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// Quote a SQL string literal, doubling internal single quotes.
pub(crate) fn quote_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
