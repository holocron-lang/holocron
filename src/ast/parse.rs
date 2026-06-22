//! YAML → AST lowering. Walks the marked-yaml tree and produces our typed
//! AST, attaching a `Span` to every node so later phases can underline the
//! offending YAML token in error messages.

use indexmap::IndexMap;
use marked_yaml::types::{MarkedMappingNode, MarkedScalarNode, MarkedSequenceNode};
use marked_yaml::{parse_yaml, Node};

use crate::ast::document::SchemaDocument;
use crate::ast::enum_type::EnumType;
use crate::ast::join::{FromClause, Join, JoinKind};
use crate::ast::select::{SelectColumn, SelectExpression, SelectItem};
use crate::ast::table::{Column, Index, PrimaryKey, Table};
use crate::ast::view::View;
use crate::error::HolocronError;
use crate::span::{Span, Spanned};

/// Parse YAML into the schema AST.
///
/// # Errors
/// Returns [`HolocronError::Parse`] when the YAML is malformed or does not
/// fit the schema shape. Errors carry the source span of the offending token.
pub fn parse_schema(source: &str) -> Result<SchemaDocument, HolocronError> {
    let root = parse_yaml(0, source).map_err(|error| {
        // marked-yaml's LoadError carries position info; embed it in the message.
        HolocronError::parse(error.to_string(), Span::default())
    })?;
    lower_document(&root)
}

// -----------------------------------------------------------------------------
// Top-level document
// -----------------------------------------------------------------------------

const DOCUMENT_KEYS: &[&str] = &["types", "tables", "views"];

fn lower_document(node: &Node) -> Result<SchemaDocument, HolocronError> {
    let mapping = expect_mapping(node, "schema document")?;
    require_known_keys(mapping, DOCUMENT_KEYS)?;
    let mut document = SchemaDocument::default();
    if let Some(value) = mapping.get("types") {
        document.types = lower_sequence(value, "types", lower_enum_type)?;
    }
    if let Some(value) = mapping.get("tables") {
        document.tables = lower_sequence(value, "tables", lower_table)?;
    }
    if let Some(value) = mapping.get("views") {
        document.views = lower_sequence(value, "views", lower_view)?;
    }
    Ok(document)
}

// -----------------------------------------------------------------------------
// Enum types
// -----------------------------------------------------------------------------

const ENUM_KEYS: &[&str] = &["name", "enum"];

fn lower_enum_type(node: &Node) -> Result<EnumType, HolocronError> {
    let mapping = expect_mapping(node, "enum type")?;
    require_known_keys(mapping, ENUM_KEYS)?;
    let name = required_spanned_string(mapping, "name", "enum type")?;
    let values_node = required_field(mapping, "enum", "enum type")?;
    let values = lower_sequence(values_node, "enum values", lower_spanned_string)?;
    Ok(EnumType {
        name,
        r#enum: values,
        span: span_of(node),
    })
}

// -----------------------------------------------------------------------------
// Tables
// -----------------------------------------------------------------------------

const TABLE_KEYS: &[&str] = &["name", "if_not_exists", "columns", "primary_key", "indexes"];

fn lower_table(node: &Node) -> Result<Table, HolocronError> {
    let mapping = expect_mapping(node, "table")?;
    require_known_keys(mapping, TABLE_KEYS)?;
    let name = required_spanned_string(mapping, "name", "table")?;
    let if_not_exists = optional_bool(mapping, "if_not_exists")?.unwrap_or(false);
    let columns_node = required_field(mapping, "columns", "table")?;
    let columns = lower_columns(columns_node)?;
    let primary_key = match mapping.get("primary_key") {
        Some(value) => Some(lower_primary_key(value)?),
        None => None,
    };
    let indexes = match mapping.get("indexes") {
        Some(value) => lower_sequence(value, "indexes", lower_index)?,
        None => Vec::new(),
    };
    Ok(Table {
        name,
        if_not_exists,
        columns,
        primary_key,
        indexes,
        span: span_of(node),
    })
}

const COLUMN_KEYS: &[&str] = &["type", "null", "default"];

fn lower_columns(node: &Node) -> Result<IndexMap<String, Column>, HolocronError> {
    let mapping = expect_mapping(node, "columns")?;
    let mut columns = IndexMap::with_capacity(mapping.len());
    for (key, value) in mapping.iter() {
        let column = lower_column(value)?;
        columns.insert(key.as_str().to_string(), column);
    }
    Ok(columns)
}

fn lower_column(node: &Node) -> Result<Column, HolocronError> {
    let mapping = expect_mapping(node, "column")?;
    require_known_keys(mapping, COLUMN_KEYS)?;
    let r#type = required_spanned_string(mapping, "type", "column")?;
    let null = optional_bool(mapping, "null")?.unwrap_or(false);
    let default = optional_string(mapping, "default")?;
    Ok(Column {
        r#type,
        null,
        default,
        span: span_of(node),
    })
}

const PRIMARY_KEY_KEYS: &[&str] = &["name", "columns"];

fn lower_primary_key(node: &Node) -> Result<PrimaryKey, HolocronError> {
    let mapping = expect_mapping(node, "primary_key")?;
    require_known_keys(mapping, PRIMARY_KEY_KEYS)?;
    let name = optional_string(mapping, "name")?;
    let columns_node = required_field(mapping, "columns", "primary_key")?;
    let columns = lower_string_list(columns_node, "primary_key columns")?;
    Ok(PrimaryKey {
        name,
        columns,
        span: span_of(node),
    })
}

const INDEX_KEYS: &[&str] = &["name", "columns", "unique", "using", "where"];

fn lower_index(node: &Node) -> Result<Index, HolocronError> {
    let mapping = expect_mapping(node, "index")?;
    require_known_keys(mapping, INDEX_KEYS)?;
    let name = required_string(mapping, "name", "index")?;
    let columns_node = required_field(mapping, "columns", "index")?;
    let columns = lower_string_list(columns_node, "index columns")?;
    let unique = optional_bool(mapping, "unique")?.unwrap_or(false);
    let using = optional_string(mapping, "using")?;
    let r#where = optional_string(mapping, "where")?;
    Ok(Index {
        name,
        columns,
        unique,
        using,
        r#where,
        span: span_of(node),
    })
}

// -----------------------------------------------------------------------------
// Views
// -----------------------------------------------------------------------------

const VIEW_KEYS: &[&str] = &[
    "name",
    "or_replace",
    "from",
    "join",
    "select",
    "where",
    "group_by",
    "having",
    "order_by",
    "limit",
    "offset",
    "distinct_on",
];

fn lower_view(node: &Node) -> Result<View, HolocronError> {
    let mapping = expect_mapping(node, "view")?;
    require_known_keys(mapping, VIEW_KEYS)?;
    let name = required_spanned_string(mapping, "name", "view")?;
    let or_replace = optional_bool(mapping, "or_replace")?.unwrap_or(false);
    let from_node = required_field(mapping, "from", "view")?;
    let from = lower_from_clause(from_node)?;
    let join = match mapping.get("join") {
        Some(value) => lower_sequence(value, "join", lower_join)?,
        None => Vec::new(),
    };
    let select_node = required_field(mapping, "select", "view")?;
    let select = lower_sequence(select_node, "select", lower_select_item)?;
    Ok(View {
        name,
        or_replace,
        from,
        join,
        select,
        r#where: optional_string(mapping, "where")?,
        group_by: optional_string(mapping, "group_by")?,
        having: optional_string(mapping, "having")?,
        order_by: optional_string(mapping, "order_by")?,
        limit: optional_u64(mapping, "limit")?,
        offset: optional_u64(mapping, "offset")?,
        distinct_on: match mapping.get("distinct_on") {
            Some(value) => Some(lower_string_list(value, "distinct_on")?),
            None => None,
        },
        span: span_of(node),
    })
}

const FROM_KEYS: &[&str] = &["table", "as"];

fn lower_from_clause(node: &Node) -> Result<FromClause, HolocronError> {
    let mapping = expect_mapping(node, "from")?;
    require_known_keys(mapping, FROM_KEYS)?;
    let table = required_spanned_string(mapping, "table", "from")?;
    let r#as = required_spanned_string(mapping, "as", "from")?;
    Ok(FromClause {
        table,
        r#as,
        span: span_of(node),
    })
}

const JOIN_KEYS: &[&str] = &["table", "as", "type", "on"];

fn lower_join(node: &Node) -> Result<Join, HolocronError> {
    let mapping = expect_mapping(node, "join")?;
    require_known_keys(mapping, JOIN_KEYS)?;
    let table = required_spanned_string(mapping, "table", "join")?;
    let r#as = required_spanned_string(mapping, "as", "join")?;
    let r#type = match mapping.get("type") {
        Some(value) => lower_join_kind(value)?,
        None => JoinKind::Inner,
    };
    let on = required_string(mapping, "on", "join")?;
    Ok(Join {
        table,
        r#as,
        r#type,
        on,
        span: span_of(node),
    })
}

fn lower_join_kind(node: &Node) -> Result<JoinKind, HolocronError> {
    let scalar = expect_scalar(node, "join type")?;
    match scalar.as_str().to_uppercase().as_str() {
        "INNER" => Ok(JoinKind::Inner),
        "LEFT" => Ok(JoinKind::Left),
        "RIGHT" => Ok(JoinKind::Right),
        "FULL" => Ok(JoinKind::Full),
        other => Err(HolocronError::parse(
            format!("unknown join type `{other}` (expected INNER, LEFT, RIGHT, or FULL)"),
            span_of(node),
        )),
    }
}

// -----------------------------------------------------------------------------
// Select items
// -----------------------------------------------------------------------------

const SELECT_COLUMN_KEYS: &[&str] = &["column", "from", "as", "filterable", "searchable"];
const SELECT_EXPRESSION_KEYS: &[&str] = &["sql", "as"];

fn lower_select_item(node: &Node) -> Result<SelectItem, HolocronError> {
    let mapping = expect_mapping(node, "select item")?;
    if mapping.get("column").is_some() {
        lower_select_column(node, mapping).map(SelectItem::Column)
    } else if mapping.get("sql").is_some() {
        lower_select_expression(node, mapping).map(SelectItem::Expression)
    } else {
        Err(HolocronError::parse(
            "select item must contain either `column` or `sql`",
            span_of(node),
        ))
    }
}

fn lower_select_column(
    node: &Node,
    mapping: &MarkedMappingNode,
) -> Result<SelectColumn, HolocronError> {
    require_known_keys(mapping, SELECT_COLUMN_KEYS)?;
    let column = required_spanned_string(mapping, "column", "select column")?;
    let from = optional_spanned_string(mapping, "from")?;
    let r#as = optional_string(mapping, "as")?;
    // Defaults: filterable = true (opt-out), searchable = false (opt-in).
    let filterable = optional_bool(mapping, "filterable")?.unwrap_or(true);
    let searchable = optional_bool(mapping, "searchable")?.unwrap_or(false);
    Ok(SelectColumn {
        column,
        from,
        r#as,
        filterable,
        searchable,
        span: span_of(node),
    })
}

fn lower_select_expression(
    node: &Node,
    mapping: &MarkedMappingNode,
) -> Result<SelectExpression, HolocronError> {
    require_known_keys(mapping, SELECT_EXPRESSION_KEYS)?;
    let sql = required_string(mapping, "sql", "select expression")?;
    let r#as = required_string(mapping, "as", "select expression")?;
    Ok(SelectExpression {
        sql,
        r#as,
        span: span_of(node),
    })
}

// -----------------------------------------------------------------------------
// Lowering helpers (the building blocks every lowering above is built from)
// -----------------------------------------------------------------------------

fn lower_sequence<T>(
    node: &Node,
    context: &str,
    lower_item: impl Fn(&Node) -> Result<T, HolocronError>,
) -> Result<Vec<T>, HolocronError> {
    let sequence = expect_sequence(node, context)?;
    sequence.iter().map(lower_item).collect()
}

fn lower_string_list(node: &Node, context: &str) -> Result<Vec<String>, HolocronError> {
    let sequence = expect_sequence(node, context)?;
    sequence
        .iter()
        .map(|item| expect_scalar(item, context).map(|scalar| scalar.as_str().to_string()))
        .collect()
}

fn lower_spanned_string(node: &Node) -> Result<Spanned<String>, HolocronError> {
    let scalar = expect_scalar(node, "string")?;
    Ok(Spanned::new(scalar.as_str().to_string(), span_of(node)))
}

fn expect_mapping<'a>(
    node: &'a Node,
    context: &str,
) -> Result<&'a MarkedMappingNode, HolocronError> {
    node.as_mapping().ok_or_else(|| {
        HolocronError::parse(format!("expected a mapping for `{context}`"), span_of(node))
    })
}

fn expect_sequence<'a>(
    node: &'a Node,
    context: &str,
) -> Result<&'a MarkedSequenceNode, HolocronError> {
    node.as_sequence().ok_or_else(|| {
        HolocronError::parse(format!("expected a list for `{context}`"), span_of(node))
    })
}

fn expect_scalar<'a>(node: &'a Node, context: &str) -> Result<&'a MarkedScalarNode, HolocronError> {
    node.as_scalar().ok_or_else(|| {
        HolocronError::parse(format!("expected a scalar for `{context}`"), span_of(node))
    })
}

fn required_field<'a>(
    mapping: &'a MarkedMappingNode,
    key: &str,
    context: &str,
) -> Result<&'a Node, HolocronError> {
    mapping.get(key).ok_or_else(|| {
        HolocronError::parse(
            format!("missing required key `{key}` on {context}"),
            span_of_mapping(mapping),
        )
    })
}

fn required_string(
    mapping: &MarkedMappingNode,
    key: &str,
    context: &str,
) -> Result<String, HolocronError> {
    let value = required_field(mapping, key, context)?;
    let scalar = expect_scalar(value, key)?;
    Ok(scalar.as_str().to_string())
}

fn required_spanned_string(
    mapping: &MarkedMappingNode,
    key: &str,
    context: &str,
) -> Result<Spanned<String>, HolocronError> {
    let value = required_field(mapping, key, context)?;
    lower_spanned_string(value)
}

fn optional_string(
    mapping: &MarkedMappingNode,
    key: &str,
) -> Result<Option<String>, HolocronError> {
    match mapping.get(key) {
        Some(value) => {
            let scalar = expect_scalar(value, key)?;
            Ok(Some(scalar.as_str().to_string()))
        }
        None => Ok(None),
    }
}

fn optional_spanned_string(
    mapping: &MarkedMappingNode,
    key: &str,
) -> Result<Option<Spanned<String>>, HolocronError> {
    match mapping.get(key) {
        Some(value) => Ok(Some(lower_spanned_string(value)?)),
        None => Ok(None),
    }
}

fn optional_bool(mapping: &MarkedMappingNode, key: &str) -> Result<Option<bool>, HolocronError> {
    match mapping.get(key) {
        Some(value) => {
            let scalar = expect_scalar(value, key)?;
            scalar.as_bool().map(Some).ok_or_else(|| {
                HolocronError::parse(format!("expected a boolean for `{key}`"), span_of(value))
            })
        }
        None => Ok(None),
    }
}

fn optional_u64(mapping: &MarkedMappingNode, key: &str) -> Result<Option<u64>, HolocronError> {
    match mapping.get(key) {
        Some(value) => {
            let scalar = expect_scalar(value, key)?;
            scalar.as_str().parse::<u64>().map(Some).map_err(|_| {
                HolocronError::parse(
                    format!("expected a non-negative integer for `{key}`"),
                    span_of(value),
                )
            })
        }
        None => Ok(None),
    }
}

/// Reject unknown keys — the manual equivalent of `#[serde(deny_unknown_fields)]`.
fn require_known_keys(mapping: &MarkedMappingNode, allowed: &[&str]) -> Result<(), HolocronError> {
    for (key, _) in mapping.iter() {
        let name = key.as_str();
        if !allowed.contains(&name) {
            let key_span = Span::from_marked(key.span());
            return Err(HolocronError::parse(
                format!(
                    "unknown key `{name}` (expected one of {})",
                    allowed
                        .iter()
                        .map(|key| format!("`{key}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                key_span,
            ));
        }
    }
    Ok(())
}

fn span_of(node: &Node) -> Span {
    match node {
        // marked-yaml returns scalar `end` as the same character as `start`,
        // which would underline a single character. Compute the true end from
        // the value's byte length so the underline covers the whole token.
        Node::Scalar(scalar) => {
            let marked = scalar.span();
            let start = marked.start().map(|marker| marker.character()).unwrap_or(0);
            let end = start + scalar.as_str().len();
            Span::new(start, end.max(start + 1))
        }
        Node::Mapping(mapping) => Span::from_marked(mapping.span()),
        Node::Sequence(sequence) => Span::from_marked(sequence.span()),
    }
}

fn span_of_mapping(mapping: &MarkedMappingNode) -> Span {
    Span::from_marked(mapping.span())
}
