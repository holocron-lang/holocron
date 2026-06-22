use crate::catalog::{Catalog, CatalogColumn, CatalogRelation};
use crate::error::HolocronError;
use crate::query::filter::{Comparison, Filter};
use crate::query::Query;

/// A query that has passed type-checking against the catalog. The only way to
/// build one is through [`check_query`]; an unchecked query is unrepresentable
/// at this type (HOLO-PARSE-DONT-VALIDATE).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedQuery {
    query: Query,
}

impl CheckedQuery {
    /// The validated underlying query.
    pub fn query(&self) -> &Query {
        &self.query
    }
}

/// Validate a query against the catalog: every referenced column must exist on
/// the target relation, be filterable, and use an operator the column's type
/// supports.
///
/// # Errors
/// Unknown relation, unknown column, non-filterable column, or unsupported
/// operator-for-type.
pub fn check_query(catalog: &Catalog, query: Query) -> Result<CheckedQuery, HolocronError> {
    let relation = catalog
        .relation(&query.relation)
        .ok_or_else(|| HolocronError::unknown_relation(&query.relation))?;
    if let Some(filter) = &query.filter {
        check_filter(filter, &query.relation, relation)?;
    }
    Ok(CheckedQuery { query })
}

fn check_filter(
    filter: &Filter,
    relation_name: &str,
    relation: &CatalogRelation,
) -> Result<(), HolocronError> {
    match filter {
        Filter::And(children) | Filter::Or(children) => {
            for child in children {
                check_filter(child, relation_name, relation)?;
            }
            Ok(())
        }
        Filter::Leaf(comparison) => check_comparison(comparison, relation_name, relation),
    }
}

fn check_comparison(
    comparison: &Comparison,
    relation_name: &str,
    relation: &CatalogRelation,
) -> Result<(), HolocronError> {
    match comparison {
        Comparison::Compare { column, op, .. } => {
            let resolved = resolve_filterable(relation_name, relation, column)?;
            if !op.supported_by(&resolved.data_type) {
                return Err(HolocronError::operator_not_supported(
                    relation_name,
                    column,
                    resolved.data_type.name(),
                    op.name(),
                ));
            }
            Ok(())
        }
        Comparison::Set { column, op, .. } => {
            let resolved = resolve_filterable(relation_name, relation, column)?;
            if !op.supported_by(&resolved.data_type) {
                return Err(HolocronError::operator_not_supported(
                    relation_name,
                    column,
                    resolved.data_type.name(),
                    op.name(),
                ));
            }
            Ok(())
        }
        Comparison::NullCheck { column, .. } => {
            // `=null=` is supported on every type, so resolution + filterable check is enough.
            resolve_filterable(relation_name, relation, column)?;
            Ok(())
        }
    }
}

/// Resolve a referenced column and confirm it is filterable.
fn resolve_filterable<'r>(
    relation_name: &str,
    relation: &'r CatalogRelation,
    column: &str,
) -> Result<&'r CatalogColumn, HolocronError> {
    let resolved = relation.column(column).ok_or_else(|| {
        let candidates = relation
            .columns
            .iter()
            .map(|column| column.name.clone())
            .collect();
        // Queries are built programmatically in this layer; no AST span to attach.
        HolocronError::unknown_column(
            relation_name,
            column,
            candidates,
            crate::span::Span::default(),
        )
    })?;
    if !resolved.filterable {
        return Err(HolocronError::not_filterable(relation_name, column));
    }
    Ok(resolved)
}
