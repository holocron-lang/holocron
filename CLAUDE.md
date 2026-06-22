# CLAUDE.md

Guidance for Claude Code (claude.ai/code) working in this repository.

A **declarative, compiler-checked data layer**, written in Rust. One human-readable YAML
file is the single source of truth for a database's *structure* **and** a semantic
*rulebook* (catalog: which fields are filterable/searchable/sortable, identity/reference
roles, read surfaces, the auth-owner). Any query — in **RSQL** or **YAML** — is
**type-checked against the catalog before it runs**, with no database connection, because
the YAML *is* the schema. The engine is a single fast Rust core, exposed to any language
via CLI + WASM + FFI. See `DESIGN.md` for the full vision, rationale, roadmap, and the
reference implementation under `reference/`.

It is, at heart, a **compiler whose target language is SQL** — parse → resolve → check →
lower → emit. Treat it like one.

> **Rule-ID legend.** `HOLO-*` = this project's own rules (`HOLO` = Holocron). `M-*` =
> external Microsoft Rust API guidelines kept verbatim. Nothing here is carried over from
> any other codebase's domain.

---

## How to work with me (highest priority)

- **Read the rules first.** Read this file (and `DESIGN.md`) before designing. They prune
  the design space.
- **Discuss → lock → implement.** Exhaustive design discussion first; **wait for an
  explicit lock word** ("go" / "do it" / "lock") before writing code. Then edit files
  directly.
- **HOLO-EXHAUST-BEFORE-BUILD** — map every scenario, edge case, and decision point and weigh
  the options *before* implementation, never mid-way.
- **HOLO-VERIFY-DONT-ASSUME** — read the actual file/symbol/config; never infer from names or
  memory. When the source contradicts an assumption, the source wins.
- **HOLO-USER-COMMITS** — never run `git commit` or `git push`. Staging, branching, writing
  code are fine; the commit is mine. Don't offer to commit unless asked.

## Response style

- **HOLO-TLDR-FIRST** — lead with the answer, not the reasoning. The conclusion first; I'll
  ask when I want more.
- **HOLO-CITE-PATH-LINE** — cite code as `path:line`; don't paste blocks unless the block is
  the answer.
- **HOLO-NO-HEADERS** — no `##` everywhere for routine answers; no tables for things that fit
  in two sentences.
- **HOLO-NO-PREAMBLE** — no "honest framing" / "to be clear" preambles; no restating the
  question. State the caveat directly.
- **HOLO-NO-NEXT-STEPS** — no "next steps / would you like me to…" unless a real decision
  needs making.
- **HOLO-EXPAND-ON-REQUEST** — expand only when asked ("deep dive", "explain", "why").

---

## Compiler architecture (this project is a compiler — build it like one)

- **HOLO-PHASES** — strict, one-way pipeline: parse → resolve names → semantic-check → lower
  to IR → emit SQL. Each phase has a clean boundary; no phase reaches back into an earlier
  one.
- **HOLO-AST-NOT-IR** — the AST mirrors the *source* (YAML/RSQL shape); the IR is the
  *normalized semantic model* (catalog + query plan). Lower one into the other explicitly;
  never blur them.
- **HOLO-SMALL-PASSES** — many small semantic passes, one concern each — not one mega-pass.
- **HOLO-NAME-RESOLUTION** — resolving a reference (a `from` alias, a column, an entity name)
  to its declaration is its own phase that produces a *resolved* tree.
- **HOLO-SCOPES** — names resolve within a scope (e.g. a `from` alias is visible only inside
  its view). Model scope explicitly; it is lexical.
- **HOLO-NO-FALLBACK** — an unresolved reference is a **compile error**, never a default (no
  "fall back to `Text`"). Every keyword is a typed reference with resolution rules.
- **HOLO-DIAGNOSTICS** — every diagnostic carries a source span (exact file/line); collect
  all errors and report together — don't die on the first.
- **HOLO-DETERMINISTIC** — same input → same output, byte-for-byte. No `HashMap` iteration
  order (or any nondeterminism) leaking into generated artifacts. Use ordered structures
  where order matters.
- **HOLO-PARSE-DONT-VALIDATE** — make illegal states unrepresentable: construct a `ResolvedX`
  type that can *only* exist if resolution/validation succeeded, rather than passing
  loosely-typed data plus an "is it valid?" flag. Highest-leverage discipline here.
- **HOLO-TYPED-ESCAPE** — where raw SQL must be allowed, require a declared type on it so the
  compiler checks *around* it; contain the unchecked part behind a type, never let it leak.
- **HOLO-GOLDEN-TESTS** — lock compiler output with snapshot tests (`insta`): compile samples,
  snapshot the SQL/catalog, diff on change. Test the IR, not just the final SQL.

(Canon to consult: *Crafting Interpreters*, rust-analyzer's architecture notes,
"Parse, don't validate".)

---

## Errors (layered enums)

- **HOLO-LAYERED-ERRORS** — errors form layered enums; each layer wraps the one below, never
  skips. A single root error type.
- **HOLO-ERROR-ENUM-FILE** — every error enum in its own file under a shared error module.
- **HOLO-ERROR-FROM** — `#[from]` for direct child types; a `transitive_from!`-style macro for
  internal types 2+ hops from the root.
- **HOLO-ERROR-HELPERS** — variant-construction helpers return the root error directly (build
  variant + `.into()` inside the helper); call sites never construct variants directly and
  never chain `.into()`. Helpers take `impl Into<String>` and `.into()` internally.
- **HOLO-INTERNAL-ERR** — internal error types use `?` directly.
- **HOLO-EXTERNAL-ERR** — external crate errors use `.map_err(LeafError::from)?`; `#[from]` for
  the external type lives on the leaf enum only.
- **HOLO-NO-GENERIC-ERR** — no `Generic { source: Box<dyn Error> }`, no `wrap()`, no external
  types on the root error.
- **Rejected: `anyhow` / `eyre`.** Use the layered enum hierarchy.

## Naming

- **HOLO-DESCRIPTIVE-NAMES** — names describe what the value represents, not its type/position
  (`error` not `e`, `connection` not `conn`, `column_count` not `n`).
- **HOLO-NO-ABBREV** — no single-letter variables; no vowel/syllable-dropped abbreviations.
- **HOLO-LOOP-NAMES** — loop/closure params name the item (`for column in &columns`,
  `.map(|value| …)`).
- **M-CONCISE-NAMES** — no weasel words (`Service`, `Manager`, `Factory`); use meaningful
  terms.
- **HOLO-GENERIC-NAMES** — generic params are `T`-prefixed PascalCase nouns (`TBackend`,
  `TDialect`, `TInput`), never single letters.
- **HOLO-NAME-END-TO-END** — one concept carries one name across every layer (source keyword →
  AST → IR → emitted SQL). Don't rename a field as it crosses a phase. Two different concepts
  never share a name.

## Imports & modules

- **HOLO-IMPORTS-TOP** — all `use` at the top of the file, before any code.
- **HOLO-IMPORTS-CRATE** — within-crate `use crate::…`; across crates `use crate_name::…`.
  Import the short name; no inline `crate::module::Type` paths.
- **HOLO-NO-SUPER-SELF** — no `use super::`, no `use self::`.
- **HOLO-NO-LOCAL-USE** — no `use` inside functions, blocks, `impl` blocks, or closures.
- Glob re-exports (`pub use foo::*`) are allowed for a deliberate single-import facade.

## Types & functions

- **HOLO-PORT-FIELD-DYN** — port-bearing struct fields are `Arc<dyn Trait>`, never
  `Arc<ConcreteAdapter>`.
- **HOLO-DEBUG-NON-EXHAUSTIVE** — hand-write `Debug` for types holding `Arc<dyn Trait>`
  (`finish_non_exhaustive()`); never `#[derive(Debug)]` on them.
- **HOLO-FREE-HELPERS-BELOW** — free helpers used by an `impl` live below it in the same file,
  default visibility.
- **M-REGULAR-FN** — associated functions are for construction (`new`, `from_*`, `with_*`);
  general computation is free functions or instance methods.
- **HOLO-VALUE-OBJECTS** — invariant-bearing types use private fields + `fn new(...) -> Result<Self>`
  enforcing the invariant, so an invalid value can't exist (pairs with HOLO-PARSE-DONT-VALIDATE).

## Magic values

- **HOLO-CONST-LITERALS** — named `const` for any literal that isn't `0`, `1`, `true`, `false`.
- **HOLO-CONST-REASON** — every `const` carries a brief comment explaining *why* the value.
- **HOLO-NO-MAGIC-STRINGS** — no magic strings for keys/prefixes/identifiers; name them `const`.

## Documentation

- **HOLO-LIB-ONELINER** — every `lib.rs` opens with a single `//!` line stating the crate's
  purpose; no filler.
- **HOLO-DEEP-DOCS-EXTERNAL** — complex modules pull deep docs from `docs/` via
  `#![doc = include_str!(...)]`.
- **HOLO-FIRST-DOC-SENTENCE** — first sentence of any doc comment is under 15 words.
- **HOLO-DOC-NON-OBVIOUS** — `///` only where behavior is non-obvious; never restate the
  signature.
- **HOLO-DOC-ERRORS-PANICS** — `# Errors` when failure modes aren't obvious from the return
  type; `# Panics` when it can panic.
- **HOLO-NO-DOC-FILLER** — no mechanical docs, no parameter tables, no long `//!` in `mod.rs`.
- **HOLO-WHY-COMMENT** — multi-line `//` blocks are welcome when they explain *why* code is
  shaped a way; never to describe *what* it does.

## Universal / correctness

- **HOLO-PUBLIC-DEBUG** — all public structs/enums derive `Debug`.
- **HOLO-ERROR-TRAITS** — error types implement `Display` + `std::error::Error`.
- **HOLO-USER-DISPLAY** — user-facing types implement `Display`.
- **HOLO-PANIC-INVARIANT** — panics only for detected programming bugs / impossible states;
  return `Result` for recoverable conditions.
- **HOLO-EXPECT-REASON** — `.expect("reason")` naming the invariant, over `.unwrap()`.
- **HOLO-NO-UNWRAP** — no `.unwrap()` in library/production code.
- **HOLO-EXPECT-LINT** — `#[expect(lint, reason = "…")]` over `#[allow(lint)]`.

## Safety

- **M-UNSAFE / M-UNSAFE-IMPLIES-UB / M-UNSOUND** — `unsafe` only for novel abstractions,
  perf-critical paths, or FFI, with written justification; misuse must imply UB; all code
  sound. The FFI bindings for the polyglot story are a legitimate `unsafe` site — document
  each one.

## Library guidelines (this ships as a public library — these matter)

- **M-TYPES-SEND** — public types are `Send`; all futures are `Send`; no `!Send` (`Rc`,
  `RefCell`) in async fns.
- **M-DONT-LEAK-TYPES** — don't leak vendor types (parser/driver internals) across crate
  boundaries; convert at the edge.
- **M-AVOID-WRAPPERS** — keep `Arc`/`Rc`/`Box`/`RefCell` out of public API surfaces; expose
  clean inherent methods.
- **M-SIMPLE-ABSTRACTIONS** — don't expose nested parametrized types in the public surface.
  Keep the compiler's public API clean.
- **M-SERVICES-CLONE** — heavyweight types implement shared-ownership `Clone` via `Arc<Inner>`.
- **M-STRONG-TYPES** — `PathBuf` for paths, `Uuid` for IDs, `chrono::DateTime` for time —
  never `String`.
- **M-ESSENTIAL-FN-INHERENT** — core functionality is inherent; traits forward to it.
- **M-INIT-BUILDER / M-INIT-CASCADED** — 4+ init permutations → builder; 4+ params → cascade
  through semantic helper types.
- **M-AVOID-STATICS** — no `static mut`, no global mutable state; thread state through typed
  handles.
- **M-MOCKABLE-SYSCALLS** — I/O (file reads, DB introspection) goes behind a trait so tests
  can mock it.

## Architectural posture

- **HOLO-HEXAGONAL-CORE** — the compiler's heart (parse → resolve → check → lower) depends on
  nothing external. Parsers, SQL renderers, DB introspection, and FFI are *adapters* at the
  edges — swappable. Keep the moat (the semantic pass) framework-free.
- **M-DI-HIERARCHY** — prefer concrete > generic > `dyn Trait`; reach for `dyn` only for
  deliberate heterogeneous collections.

## Performance

- **M-HOTPATH** — identify hot paths early, benchmark, profile. Schema compilation and query
  validation throughput are the candidate hot paths — treat them as such.
- **M-THROUGHPUT** — optimize items/CPU-cycle; batch; avoid wasted work.
- `mimalloc` as the global allocator for any shipped binary (the CLI).

## Verification (run before any commit)

- `cargo fmt --all`
- `cargo clippy --workspace` — treat warnings as errors
- `cargo check --workspace`
- `cargo test` — including `insta` snapshot tests for compiler output

## Tech-debt tracking

- **Flag, merge, track — don't block.** Blocking is for correctness/safety/data-loss, not
  structural debt. File one tracking issue per accepted-debt change and cite the specific
  rule it violates ("why it's debt" without a cited rule is an opinion — resolve it in
  review).
