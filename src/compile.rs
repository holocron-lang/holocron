use crate::ast::parse_schema;
use crate::catalog::{build_catalog, Catalog};
use crate::emit::emit_schema;
use crate::error::HolocronError;
use crate::resolve::resolve_views;

/// The result of compiling a YAML schema: the validated catalog (the symbol
/// table queries are checked against) and the rendered PostgreSQL DDL.
#[derive(Debug, Clone)]
pub struct Compiled {
    pub catalog: Catalog,
    pub ddl: String,
}

/// Compile a YAML schema end-to-end: parse → build catalog → resolve views →
/// emit DDL. The single entry point that strings every phase together.
///
/// # Errors
/// Any error from the parse, catalog-build, or view-resolve phases. Emit is
/// infallible by construction (it runs only after every check has passed).
pub fn compile(input: &str) -> Result<Compiled, HolocronError> {
    let document = parse_schema(input)?;
    let catalog = build_catalog(&document)?;
    let catalog = resolve_views(catalog, &document)?;
    let ddl = emit_schema(&document);
    Ok(Compiled { catalog, ddl })
}
