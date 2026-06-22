use crate::ast::{SchemaDocument, View};
use crate::catalog::{Catalog, CatalogRelation, RelationKind};
use crate::error::HolocronError;
use crate::resolve::scope::Scope;
use crate::resolve::select::resolve_columns;

/// Resolve every view in the document into a catalog relation, extending the
/// table-and-enum catalog produced by the previous phase.
///
/// # Errors
/// Any unresolved source, alias, or column in a view.
pub fn resolve_views(
    mut catalog: Catalog,
    document: &SchemaDocument,
) -> Result<Catalog, HolocronError> {
    for view in &document.views {
        let relation = resolve_view(view, &catalog)?;
        catalog.insert_relation(relation, view.name.span)?;
    }
    Ok(catalog)
}

fn resolve_view(view: &View, catalog: &Catalog) -> Result<CatalogRelation, HolocronError> {
    let scope = Scope::build(view, catalog)?;
    let columns = resolve_columns(view, &scope)?;
    Ok(CatalogRelation {
        name: view.name.value.clone(),
        kind: RelationKind::View,
        columns,
    })
}
