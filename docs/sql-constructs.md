# SQL Vocabulary Holocron Emits

The complete set of SQL constructs Holocron generates — nothing more. This is the
checklist the emitter is built against: each entry is one construct, what it means, the
rule it carries, the syntax shape, and a tiny example.

**Scope.** Core SQL, **PostgreSQL dialect first** (other dialects are "accents" added
later). Anything too complex to model is passed through as a raw, type-declared SQL
fragment (the *typed escape hatch* — see `DESIGN.md` §6.1), so this list stays small.

Two groups: **schema** (build the structure) and **query** (read the data).

---

## Schema constructs (DDL)

### `CREATE TYPE … AS ENUM`
Define a fixed set of allowed values (e.g. a status).
**Rule:** a column of this type may only hold one of the listed values.
```sql
CREATE TYPE <name> AS ENUM ('Value1', 'Value2', 'Value3');
```
```sql
CREATE TYPE execution_status AS ENUM ('Pending', 'Running', 'Completed');
```

### `CREATE TABLE`
Real storage that holds rows. Lists columns, each with a name and a type.
**Rule:** name unique in the schema; every column declares a type.
```sql
CREATE TABLE [IF NOT EXISTS] <name> (
  <column> <type> [NOT NULL] [DEFAULT <expr>],
  ...,
  [CONSTRAINT <pk_name>] PRIMARY KEY (<columns>)
);
```
```sql
CREATE TABLE snap_executions (
  execution_id uuid NOT NULL,
  status execution_status NOT NULL DEFAULT 'Pending',
  version bigint NOT NULL,
  PRIMARY KEY (execution_id, version)
);
```

### Column definition
One column inside a table: a name, a type, and optional modifiers.
**Rule:** `NOT NULL` forbids empty values; `DEFAULT` supplies a value when none is given.
```sql
<name> <type> [NOT NULL] [DEFAULT <expr>]
```

### `PRIMARY KEY`
The column(s) that uniquely identify a row.
**Rule:** never empty, only one per table, no two rows may share the same value.
```sql
PRIMARY KEY (<column> [, <column> ...])
```

### `CREATE INDEX`
A lookup shortcut that makes searches/sorts on column(s) fast.
**Rule:** changes performance only, never results. Optional `UNIQUE` also enforces no
duplicates; optional `WHERE` makes it partial.
```sql
CREATE [UNIQUE] INDEX [IF NOT EXISTS] <name>
  ON <table> [USING <method>] (<columns>) [WHERE <condition>];
```
```sql
CREATE INDEX index_snap_executions_status ON snap_executions (status);
```

### `CREATE VIEW … AS`
A saved query that looks like a table. Stores no data; the query underneath runs each
time the view is read.
**Rule:** the body must be a valid `SELECT`; the view's columns are whatever that
`SELECT` returns. `OR REPLACE` redefines an existing view in place.
```sql
CREATE [OR REPLACE] VIEW <name> AS
<select-statement>;
```
```sql
CREATE OR REPLACE VIEW executions AS
SELECT execution_id, status FROM snap_executions WHERE is_deleted = false;
```

---

## Query constructs (`SELECT` and its parts)

Listed in the order SQL logically applies them. A full read statement is assembled from
these pieces.

### `SELECT`
Pick which columns or expressions come back.
**Rule:** at least one item required.
```sql
SELECT <item> [, <item> ...]
```

### `*`
Shorthand for "all columns of the source."
**Rule:** expands to every column; can be qualified as `alias.*`.

### `AS`
Rename something in the output — a column or a table nickname (alias).
**Rule:** pure relabeling; the alias is how later clauses refer to it.
```sql
<expr> AS <output_name>          -- column alias
<table> AS <alias>               -- table alias
```

### `FROM`
Which table or view to read from.
**Rule:** the source must exist; an alias declared here is visible to the rest of the
statement. The source may itself be a subquery: `(SELECT …) AS alias`.
```sql
FROM <table|view|subquery> [AS <alias>]
```

### `JOIN … ON`
Combine rows from two sources by a matching rule.
**Rule:** `ON` states how rows pair up. `INNER` keeps only matches; `LEFT` keeps all
left-side rows, filling unmatched right-side columns with NULL.
```sql
[INNER | LEFT | RIGHT | FULL] JOIN <source> [AS <alias>] ON <condition>
```
```sql
LEFT JOIN snap_nodes AS n ON n.execution_id = e.execution_id
```

### `WHERE`
Keep only rows that match a condition (the filter).
**Rule:** evaluated per row, before grouping; only rows that pass continue.
```sql
WHERE <condition>
```

### `GROUP BY`
Collapse many rows into groups, so aggregates (COUNT, SUM…) compute per group.
**Rule:** every selected column must either be in `GROUP BY` or wrapped in an aggregate.
```sql
GROUP BY <expr> [, <expr> ...]
```

### `HAVING`
Like `WHERE`, but filters the *groups* after grouping.
**Rule:** only valid alongside `GROUP BY`; conditions reference aggregates.
```sql
HAVING <condition>
```

### `ORDER BY`
Sort the result.
**Rule:** `ASC` (default) ascending, `DESC` descending; multiple keys break ties
left-to-right.
```sql
ORDER BY <expr> [ASC | DESC] [, <expr> [ASC | DESC] ...]
```

### `LIMIT` / `OFFSET`
Take only N rows / skip N rows — the basis of pagination.
**Rule:** `OFFSET` skips first, then `LIMIT` caps the count.
```sql
LIMIT <count> [OFFSET <skip>]
```

### `DISTINCT` / `DISTINCT ON`
Remove duplicate rows.
**Rule:** `DISTINCT` dedupes whole rows; `DISTINCT ON (cols)` keeps the first row per
distinct value of `cols` (pairs with `ORDER BY` to decide which "first").
```sql
SELECT DISTINCT <items> ...
SELECT DISTINCT ON (<cols>) <items> ... ORDER BY <cols>, ...
```

---

## Summary

| Group | Constructs |
|---|---|
| Schema | `CREATE TYPE … AS ENUM`, `CREATE TABLE`, column definition, `PRIMARY KEY`, `CREATE INDEX`, `CREATE VIEW … AS` |
| Query | `SELECT`, `*`, `AS`, `FROM`, `JOIN … ON`, `WHERE`, `GROUP BY`, `HAVING`, `ORDER BY`, `LIMIT`/`OFFSET`, `DISTINCT`/`DISTINCT ON` |

Roughly 6 schema pieces and 11 query pieces. Everything beyond this is either a dialect
"accent" or a raw fragment passed through the typed escape hatch — not new core
vocabulary.
