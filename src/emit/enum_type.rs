use crate::ast::EnumType;
use crate::emit::{quote_ident, quote_literal};

/// Emit `CREATE TYPE <name> AS ENUM ('v1', 'v2', ...);`.
pub(crate) fn emit(output: &mut String, declared: &EnumType) {
    output.push_str("CREATE TYPE ");
    output.push_str(&quote_ident(&declared.name.value));
    output.push_str(" AS ENUM (");
    let values: Vec<String> = declared
        .r#enum
        .iter()
        .map(|value| quote_literal(&value.value))
        .collect();
    output.push_str(&values.join(", "));
    output.push_str(");\n\n");
}
