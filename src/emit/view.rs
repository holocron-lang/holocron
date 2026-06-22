use crate::ast::{JoinKind, SelectColumn, SelectExpression, SelectItem, View};
use crate::emit::quote_ident;

/// Emit `CREATE [OR REPLACE] VIEW name AS <select-body>;`. Structured parts
/// (select items, from, joins) are rendered; clause bodies (where, group_by,
/// having, order_by) are raw SQL passed through verbatim.
pub(crate) fn emit(output: &mut String, view: &View) {
    output.push_str("CREATE ");
    if view.or_replace {
        output.push_str("OR REPLACE ");
    }
    output.push_str("VIEW ");
    output.push_str(&quote_ident(&view.name.value));
    output.push_str(" AS\nSELECT");

    if let Some(distinct_on) = &view.distinct_on {
        let columns: Vec<String> = distinct_on
            .iter()
            .map(|column| quote_ident(column))
            .collect();
        output.push_str(" DISTINCT ON (");
        output.push_str(&columns.join(", "));
        output.push(')');
    }
    output.push('\n');

    let items: Vec<String> = view.select.iter().map(emit_select_item).collect();
    let last = items.len() - 1;
    for (index, item) in items.iter().enumerate() {
        output.push_str("    ");
        output.push_str(item);
        if index < last {
            output.push(',');
        }
        output.push('\n');
    }

    output.push_str("FROM ");
    output.push_str(&quote_ident(&view.from.table.value));
    output.push_str(" AS ");
    output.push_str(&quote_ident(&view.from.r#as.value));

    for join in &view.join {
        output.push('\n');
        output.push_str(join_kind_sql(join.r#type));
        output.push_str(" JOIN ");
        output.push_str(&quote_ident(&join.table.value));
        output.push_str(" AS ");
        output.push_str(&quote_ident(&join.r#as.value));
        // Join condition is raw SQL, passed through verbatim.
        output.push_str(" ON ");
        output.push_str(&join.on);
    }

    push_raw_clause(output, "WHERE", view.r#where.as_deref());
    push_raw_clause(output, "GROUP BY", view.group_by.as_deref());
    push_raw_clause(output, "HAVING", view.having.as_deref());
    push_raw_clause(output, "ORDER BY", view.order_by.as_deref());

    if let Some(limit) = view.limit {
        output.push_str(&format!("\nLIMIT {limit}"));
    }
    if let Some(offset) = view.offset {
        output.push_str(&format!("\nOFFSET {offset}"));
    }

    output.push_str(";\n\n");
}

fn emit_select_item(item: &SelectItem) -> String {
    match item {
        SelectItem::Column(column) => emit_select_column(column),
        SelectItem::Expression(expression) => emit_select_expression(expression),
    }
}

fn emit_select_column(column: &SelectColumn) -> String {
    let mut sql = String::new();
    if let Some(alias) = &column.from {
        sql.push_str(&quote_ident(&alias.value));
        sql.push('.');
    }
    sql.push_str(&quote_ident(&column.column.value));
    // Only emit AS when the output name differs from the source — same name
    // adds no meaning and just clutters the SQL.
    if let Some(output_name) = &column.r#as {
        if *output_name != column.column.value {
            sql.push_str(" AS ");
            sql.push_str(&quote_ident(output_name));
        }
    }
    sql
}

fn emit_select_expression(expression: &SelectExpression) -> String {
    // Expression body is raw SQL; the output alias is a real identifier.
    format!("{} AS {}", expression.sql, quote_ident(&expression.r#as))
}

fn push_raw_clause(output: &mut String, keyword: &str, body: Option<&str>) {
    if let Some(body) = body {
        output.push('\n');
        output.push_str(keyword);
        output.push(' ');
        output.push_str(body);
    }
}

fn join_kind_sql(kind: JoinKind) -> &'static str {
    match kind {
        JoinKind::Inner => "INNER",
        JoinKind::Left => "LEFT",
        JoinKind::Right => "RIGHT",
        JoinKind::Full => "FULL",
    }
}
