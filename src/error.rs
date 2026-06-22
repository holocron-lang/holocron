use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use thiserror::Error;

use crate::span::Span;

/// The crate's single root error type. Each variant carries the source span
/// of the offending token (zero-default when the span is unknown), so the
/// renderer can underline the right place in the YAML.
#[derive(Debug, Error)]
pub enum HolocronError {
    /// The YAML was malformed or did not fit the declared schema shape (L1).
    #[error("parse error: {message}")]
    Parse { message: String, span: Span },

    /// A column's type is neither a built-in nor a declared enum (L2).
    #[error("unknown type `{type_name}` on column `{relation}.{column}`")]
    UnknownType {
        relation: String,
        column: String,
        type_name: String,
        span: Span,
    },

    /// Two relations share a name (L2/L3).
    #[error("duplicate relation `{name}`")]
    DuplicateRelation {
        name: String,
        /// The site that originally declared this name.
        first_span: Span,
        /// The redeclaration that triggered the error.
        span: Span,
    },

    /// Two enum types share a name (L2).
    #[error("duplicate enum type `{name}`")]
    DuplicateEnum {
        name: String,
        first_span: Span,
        span: Span,
    },

    /// A view source references a relation that does not exist (L3).
    #[error("view `{view}`: source `{alias}` references unknown relation `{relation}`")]
    UnknownSource {
        view: String,
        alias: String,
        relation: String,
        /// Relations known at the time the source was resolved (rendered as
        /// a "available" note alongside the underline).
        candidates: Vec<String>,
        span: Span,
    },

    /// Two sources in a view share an alias (L3).
    #[error("view `{view}`: duplicate alias `{alias}`")]
    DuplicateAlias {
        view: String,
        alias: String,
        first_span: Span,
        span: Span,
    },

    /// A select item's `from` is not a declared alias in the view (L3).
    #[error("view `{view}`: select references unknown alias `{alias}`")]
    UnknownAlias {
        view: String,
        alias: String,
        candidates: Vec<String>,
        span: Span,
    },

    /// A select omitted `from` but the view has more than one source (L3).
    #[error("view `{view}`: column `{column}` needs an explicit `from` (multiple sources)")]
    AmbiguousSource {
        view: String,
        column: String,
        /// Aliases the user could choose from (rendered as a note).
        candidates: Vec<String>,
        span: Span,
    },

    /// A select references a column the relation does not have (L3/L4).
    #[error("column `{column}` does not exist in relation `{relation}`")]
    UnknownColumn {
        relation: String,
        column: String,
        candidates: Vec<String>,
        span: Span,
    },

    /// A construct that is recognised but not yet implemented.
    #[error("unsupported: {message}")]
    Unsupported { message: String, span: Span },

    /// A query targets a relation the catalog has no entry for (L4).
    #[error("unknown relation `{0}`")]
    UnknownRelation(String),

    /// A query filter references a column declared non-filterable (L4).
    #[error("column `{relation}.{column}` is not filterable")]
    NotFilterable { relation: String, column: String },

    /// An operator is not valid for the column's type (L4).
    #[error("operator `{operator}` not supported on `{relation}.{column}` of type `{data_type}`")]
    OperatorNotSupported {
        relation: String,
        column: String,
        data_type: String,
        operator: String,
    },
}

impl HolocronError {
    pub(crate) fn parse(message: impl Into<String>, span: Span) -> Self {
        Self::Parse {
            message: message.into(),
            span,
        }
    }

    pub(crate) fn unknown_type(
        relation: impl Into<String>,
        column: impl Into<String>,
        type_name: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::UnknownType {
            relation: relation.into(),
            column: column.into(),
            type_name: type_name.into(),
            span,
        }
    }

    pub(crate) fn duplicate_relation(
        name: impl Into<String>,
        first_span: Span,
        span: Span,
    ) -> Self {
        Self::DuplicateRelation {
            name: name.into(),
            first_span,
            span,
        }
    }

    pub(crate) fn duplicate_enum(name: impl Into<String>, first_span: Span, span: Span) -> Self {
        Self::DuplicateEnum {
            name: name.into(),
            first_span,
            span,
        }
    }

    pub(crate) fn unknown_source(
        view: impl Into<String>,
        alias: impl Into<String>,
        relation: impl Into<String>,
        candidates: Vec<String>,
        span: Span,
    ) -> Self {
        Self::UnknownSource {
            view: view.into(),
            alias: alias.into(),
            relation: relation.into(),
            candidates: sorted_unique(candidates),
            span,
        }
    }

    pub(crate) fn duplicate_alias(
        view: impl Into<String>,
        alias: impl Into<String>,
        first_span: Span,
        span: Span,
    ) -> Self {
        Self::DuplicateAlias {
            view: view.into(),
            alias: alias.into(),
            first_span,
            span,
        }
    }

    pub(crate) fn unknown_alias(
        view: impl Into<String>,
        alias: impl Into<String>,
        candidates: Vec<String>,
        span: Span,
    ) -> Self {
        Self::UnknownAlias {
            view: view.into(),
            alias: alias.into(),
            candidates: sorted_unique(candidates),
            span,
        }
    }

    pub(crate) fn ambiguous_source(
        view: impl Into<String>,
        column: impl Into<String>,
        candidates: Vec<String>,
        span: Span,
    ) -> Self {
        Self::AmbiguousSource {
            view: view.into(),
            column: column.into(),
            candidates: sorted_unique(candidates),
            span,
        }
    }

    pub(crate) fn unknown_column(
        relation: impl Into<String>,
        column: impl Into<String>,
        candidates: Vec<String>,
        span: Span,
    ) -> Self {
        Self::UnknownColumn {
            relation: relation.into(),
            column: column.into(),
            candidates: sorted_unique(candidates),
            span,
        }
    }

    pub(crate) fn unsupported(message: impl Into<String>, span: Span) -> Self {
        Self::Unsupported {
            message: message.into(),
            span,
        }
    }

    pub(crate) fn unknown_relation(name: impl Into<String>) -> Self {
        Self::UnknownRelation(name.into())
    }

    pub(crate) fn not_filterable(relation: impl Into<String>, column: impl Into<String>) -> Self {
        Self::NotFilterable {
            relation: relation.into(),
            column: column.into(),
        }
    }

    pub(crate) fn operator_not_supported(
        relation: impl Into<String>,
        column: impl Into<String>,
        data_type: impl Into<String>,
        operator: impl Into<String>,
    ) -> Self {
        Self::OperatorNotSupported {
            relation: relation.into(),
            column: column.into(),
            data_type: data_type.into(),
            operator: operator.into(),
        }
    }

    /// The source span associated with this error, if any.
    pub fn span(&self) -> Option<Span> {
        match self {
            Self::Parse { span, .. }
            | Self::UnknownType { span, .. }
            | Self::DuplicateRelation { span, .. }
            | Self::DuplicateEnum { span, .. }
            | Self::UnknownSource { span, .. }
            | Self::DuplicateAlias { span, .. }
            | Self::UnknownAlias { span, .. }
            | Self::AmbiguousSource { span, .. }
            | Self::UnknownColumn { span, .. }
            | Self::Unsupported { span, .. } => Some(*span),
            Self::UnknownRelation(_)
            | Self::NotFilterable { .. }
            | Self::OperatorNotSupported { .. } => None,
        }
    }

    /// Short label printed next to the underline (the "what went wrong here").
    fn label(&self) -> String {
        match self {
            Self::Parse { .. } => "here".to_string(),
            Self::UnknownType { type_name, .. } => format!("unknown type `{type_name}`"),
            Self::DuplicateRelation { name, .. } => format!("`{name}` redeclared here"),
            Self::DuplicateEnum { name, .. } => format!("`{name}` redeclared here"),
            Self::UnknownSource { relation, .. } => format!("`{relation}` is not declared"),
            Self::DuplicateAlias { alias, .. } => format!("alias `{alias}` redeclared here"),
            Self::UnknownAlias { alias, .. } => format!("alias `{alias}` not in scope"),
            Self::AmbiguousSource { .. } => "needs an explicit `from`".to_string(),
            Self::UnknownColumn {
                column, relation, ..
            } => {
                format!("`{column}` not in `{relation}`")
            }
            Self::Unsupported { .. } => "unsupported here".to_string(),
            _ => String::new(),
        }
    }

    /// Help / note lines added below the underline. Each variant returns the
    /// information that turns "what went wrong" into "what you can do about it".
    fn notes(&self) -> Vec<String> {
        match self {
            Self::UnknownType { .. } => vec![format!(
                "valid built-in types: {}",
                BUILTIN_TYPE_NAMES.join(", "),
            )],
            Self::UnknownSource { candidates, .. } if !candidates.is_empty() => {
                vec![format!("declared relations: {}", candidates.join(", "))]
            }
            Self::UnknownAlias { candidates, .. } if !candidates.is_empty() => {
                vec![format!("declared aliases: {}", candidates.join(", "))]
            }
            Self::UnknownColumn { candidates, .. } if !candidates.is_empty() => {
                vec![format!("available columns: {}", candidates.join(", "))]
            }
            Self::AmbiguousSource { candidates, .. } if !candidates.is_empty() => {
                vec![format!("available sources: {}", candidates.join(", "))]
            }
            _ => Vec::new(),
        }
    }

    /// A secondary label pinpointing the *original* occurrence on duplicate
    /// errors (rendered alongside the primary "redeclared here" underline).
    fn secondary_label(&self) -> Option<(Span, String)> {
        match self {
            Self::DuplicateRelation { first_span, .. }
            | Self::DuplicateEnum { first_span, .. }
            | Self::DuplicateAlias { first_span, .. } => {
                Some((*first_span, "first declared here".to_string()))
            }
            _ => None,
        }
    }

    /// Render this error as a coloured `ariadne` report against the given
    /// source. Falls back to the plain `Display` message when no span is set.
    pub fn render(&self, filename: &str, source: &str) -> String {
        let Some(span) = self.span() else {
            return format!("error: {self}");
        };
        // Clamp the span to the actual source length — guards against
        // out-of-range spans (e.g. when start/end markers were missing).
        let start = span.start.min(source.len());
        let end = span.end.min(source.len()).max(start + 1).min(source.len());
        let mut colours = ColorGenerator::new();
        let primary = colours.next();
        let mut builder = Report::build(ReportKind::Error, (filename, start..end))
            .with_message(self.to_string())
            .with_label(
                Label::new((filename, start..end))
                    .with_message(self.label())
                    .with_color(primary),
            );
        if let Some((secondary_span, secondary_message)) = self.secondary_label() {
            let secondary_color = colours.next();
            let sec_start = secondary_span.start.min(source.len());
            let sec_end = secondary_span
                .end
                .min(source.len())
                .max(sec_start + 1)
                .min(source.len());
            builder = builder.with_label(
                Label::new((filename, sec_start..sec_end))
                    .with_message(secondary_message)
                    .with_color(secondary_color),
            );
        }
        for note in self.notes() {
            builder = builder.with_note(note);
        }
        let mut buffer = Vec::new();
        builder
            .finish()
            .write((filename, Source::from(source)), &mut buffer)
            .expect("writing to a Vec<u8> is infallible");
        String::from_utf8(buffer).expect("ariadne emits UTF-8")
    }
}

/// PostgreSQL built-in type names Holocron recognises. Surfaced as a note on
/// `UnknownType` so users see what they could have typed instead.
const BUILTIN_TYPE_NAMES: &[&str] = &[
    "text",
    "uuid",
    "boolean",
    "timestamptz",
    "jsonb",
    "bigint",
    "integer",
    "smallint",
    "text[]",
];

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}
