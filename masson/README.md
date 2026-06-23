# Mason package definition

This folder mirrors the package file we contribute to
[`mason-org/mason-registry`](https://github.com/mason-org/mason-registry) so
Neovim users can install our LSP with `:Mason install holocron-lsp`.

The file lives here so version updates can be tracked in this repo alongside the
release that produced them; the actual installable copy is the one in the
registry's `packages/holocron-lsp/package.yaml`.

## How to update the registry after a new release

1. Open [`mason-org/mason-registry`](https://github.com/mason-org/mason-registry) in a fork.
2. Bump the `@vX.Y.Z` in `source.id` here, copy the file to
   `packages/holocron-lsp/package.yaml` in the registry, and open a PR.
3. (Optional) Mason has an auto-bump bot that opens these PRs for new releases —
   manual edits only needed when the asset layout changes.

## Why the spelling "masson"

The directory name is intentional (not a typo) — keeps it separate from any
future `packages/` directory we might publish under our own name.
