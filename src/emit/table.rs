use crate::ast::{Index, Table};
use crate::emit::quote_ident;

/// Emit `CREATE TABLE` with column definitions and primary-key constraint.
pub(crate) fn emit_table(output: &mut String, table: &Table) {
    output.push_str("CREATE TABLE ");
    if table.if_not_exists {
        output.push_str("IF NOT EXISTS ");
    }
    output.push_str(&quote_ident(&table.name.value));
    output.push_str(" (\n");

    let mut entries: Vec<String> = Vec::with_capacity(table.columns.len() + 1);
    for (name, column) in &table.columns {
        let mut line = format!("    {} {}", quote_ident(name), column.r#type.value);
        if !column.null {
            line.push_str(" NOT NULL");
        }
        if let Some(default) = &column.default {
            // Default expression is raw SQL, passed through verbatim.
            line.push_str(" DEFAULT ");
            line.push_str(default);
        }
        entries.push(line);
    }

    if let Some(primary_key) = &table.primary_key {
        let mut line = String::from("    ");
        if let Some(name) = &primary_key.name {
            line.push_str("CONSTRAINT ");
            line.push_str(&quote_ident(name));
            line.push(' ');
        }
        line.push_str("PRIMARY KEY (");
        let columns: Vec<String> = primary_key
            .columns
            .iter()
            .map(|column| quote_ident(column))
            .collect();
        line.push_str(&columns.join(", "));
        line.push(')');
        entries.push(line);
    }

    output.push_str(&entries.join(",\n"));
    output.push_str("\n);\n\n");
}

/// Emit `CREATE INDEX` (separately from the table, after every table exists).
pub(crate) fn emit_index(output: &mut String, table_name: &str, index: &Index) {
    output.push_str("CREATE ");
    if index.unique {
        output.push_str("UNIQUE ");
    }
    output.push_str("INDEX ");
    output.push_str(&quote_ident(&index.name));
    output.push_str(" ON ");
    output.push_str(&quote_ident(table_name));
    if let Some(method) = &index.using {
        // Index method is a Postgres identifier (`btree`, `gin`, …), not a string literal.
        output.push_str(" USING ");
        output.push_str(method);
    }
    output.push_str(" (");
    let columns: Vec<String> = index
        .columns
        .iter()
        .map(|column| quote_ident(column))
        .collect();
    output.push_str(&columns.join(", "));
    output.push(')');
    if let Some(predicate) = &index.r#where {
        // Partial-index predicate is raw SQL, passed through verbatim.
        output.push_str(" WHERE ");
        output.push_str(predicate);
    }
    output.push_str(";\n\n");
}
