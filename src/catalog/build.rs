use indexmap::IndexMap;

use crate::ast::SchemaDocument;
use crate::catalog::{Catalog, CatalogColumn, CatalogRelation, CatalogType, RelationKind};
use crate::error::HolocronError;
use crate::span::Span;

/// Lower the parsed document into a resolved catalog (tables + enums).
///
/// # Errors
/// An unknown column type, a duplicate enum name, or a duplicate relation name.
pub fn build_catalog(document: &SchemaDocument) -> Result<Catalog, HolocronError> {
    let enums = build_enums(document)?;
    let mut catalog = Catalog {
        relations: IndexMap::new(),
        enums,
        relation_spans: IndexMap::new(),
    };
    add_tables(&mut catalog, document)?;
    Ok(catalog)
}

fn build_enums(document: &SchemaDocument) -> Result<IndexMap<String, Vec<String>>, HolocronError> {
    let mut enums = IndexMap::new();
    // Track the first span we saw for each enum name so duplicate diagnostics
    // can underline both occurrences.
    let mut first_spans: IndexMap<String, Span> = IndexMap::new();
    for declared in &document.types {
        let name = declared.name.value.clone();
        if let Some(&first_span) = first_spans.get(&name) {
            return Err(HolocronError::duplicate_enum(
                name,
                first_span,
                declared.name.span,
            ));
        }
        let values: Vec<String> = declared
            .r#enum
            .iter()
            .map(|value| value.value.clone())
            .collect();
        first_spans.insert(name.clone(), declared.name.span);
        enums.insert(name, values);
    }
    Ok(enums)
}

fn add_tables(catalog: &mut Catalog, document: &SchemaDocument) -> Result<(), HolocronError> {
    for table in &document.tables {
        let mut columns = Vec::with_capacity(table.columns.len());
        for (name, column) in &table.columns {
            let data_type =
                resolve_type(&column.r#type.value, &catalog.enums).ok_or_else(|| {
                    HolocronError::unknown_type(
                        table.name.value.clone(),
                        name,
                        column.r#type.value.clone(),
                        column.r#type.span,
                    )
                })?;
            columns.push(CatalogColumn {
                name: name.clone(),
                data_type,
                nullable: column.null,
                // Table columns are filterable by default; searchable is opt-in
                // (view select items can override both).
                filterable: true,
                searchable: false,
            });
        }
        let relation = CatalogRelation {
            name: table.name.value.clone(),
            kind: RelationKind::Table,
            columns,
        };
        catalog.insert_relation(relation, table.name.span)?;
    }
    Ok(())
}

/// Resolve a YAML type name: a built-in, or a declared enum, else `None`.
fn resolve_type(name: &str, enums: &IndexMap<String, Vec<String>>) -> Option<CatalogType> {
    CatalogType::from_sql_name(name).or_else(|| {
        enums
            .contains_key(name)
            .then(|| CatalogType::Enum(name.to_string()))
    })
}
