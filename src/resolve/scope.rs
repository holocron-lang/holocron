use indexmap::IndexMap;

use crate::ast::View;
use crate::catalog::{Catalog, CatalogRelation};
use crate::error::HolocronError;
use crate::span::{Span, Spanned};

/// One alias declared inside a view: where it was declared (so we can underline
/// it on a duplicate) and the relation it points at.
struct AliasBinding<'catalog> {
    span: Span,
    relation: &'catalog CatalogRelation,
}

/// The aliases visible inside one view: each `from`/`join` alias bound to its
/// relation in the catalog. Lexical scope — visible only within the view.
pub(crate) struct Scope<'catalog> {
    aliases: IndexMap<String, AliasBinding<'catalog>>,
}

impl<'catalog> Scope<'catalog> {
    /// Resolve every `from`/`join` source of a view to a catalog relation.
    ///
    /// # Errors
    /// A source referencing an unknown relation, or two sources sharing an alias.
    pub(crate) fn build(view: &View, catalog: &'catalog Catalog) -> Result<Self, HolocronError> {
        let mut aliases = IndexMap::new();
        bind(
            &mut aliases,
            &view.name.value,
            &view.from.r#as,
            &view.from.table,
            catalog,
        )?;
        for join in &view.join {
            bind(
                &mut aliases,
                &view.name.value,
                &join.r#as,
                &join.table,
                catalog,
            )?;
        }
        Ok(Self { aliases })
    }

    /// The relation an alias is bound to, if declared.
    pub(crate) fn relation(&self, alias: &str) -> Option<&'catalog CatalogRelation> {
        self.aliases.get(alias).map(|binding| binding.relation)
    }

    /// The sole source, when there is exactly one — used to infer an omitted `from`.
    pub(crate) fn sole_relation(&self) -> Option<&'catalog CatalogRelation> {
        if self.aliases.len() == 1 {
            self.aliases.values().next().map(|binding| binding.relation)
        } else {
            None
        }
    }

    /// Every alias declared in this view (for "did you mean" notes on errors).
    pub(crate) fn alias_names(&self) -> Vec<String> {
        self.aliases.keys().cloned().collect()
    }
}

fn bind<'catalog>(
    aliases: &mut IndexMap<String, AliasBinding<'catalog>>,
    view: &str,
    alias: &Spanned<String>,
    relation: &Spanned<String>,
    catalog: &'catalog Catalog,
) -> Result<(), HolocronError> {
    let resolved = catalog.relation(&relation.value).ok_or_else(|| {
        let candidates = catalog
            .relations()
            .map(|relation| relation.name.clone())
            .collect();
        HolocronError::unknown_source(
            view,
            &alias.value,
            &relation.value,
            candidates,
            relation.span,
        )
    })?;
    if let Some(existing) = aliases.get(&alias.value) {
        return Err(HolocronError::duplicate_alias(
            view,
            &alias.value,
            existing.span,
            alias.span,
        ));
    }
    aliases.insert(
        alias.value.clone(),
        AliasBinding {
            span: alias.span,
            relation: resolved,
        },
    );
    Ok(())
}
