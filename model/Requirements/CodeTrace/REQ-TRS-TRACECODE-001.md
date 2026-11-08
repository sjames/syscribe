---
type: Requirement
id: REQ-TRS-TRACECODE-001
name: "trace-code lists the code units that no implementedBy or sourceFile names"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - traceability
---

`syscribe trace-code` shall report the code in the repository that no design element or TestCase names (GH #451).

## Behavior

- An opt-in `[traceCode]` table in `<model_root>/.syscribe.toml` (snake_case or camelCase keys) names `roots` (repository-relative, resolved against the repository root `repo:` paths use, else the model root's parent), `exclude` globs (`target/`, `node_modules/`, `.git/` always excluded) and `units` rules.
- A unit rule is a kind, optionally limited by `under` (a repository-relative subtree) and, for `file`, by `extensions`: `cargo-crate` is a directory holding a `Cargo.toml` with a `[package]` table; `build-dir` is a directory holding one of `build_files` (default `Cargo.toml`, `package.json`, `CMakeLists.txt`, `Makefile`, `go.mod`, `pyproject.toml`, `setup.py`); `file` is a single file. `units` may be one kind as a string; the default is `build-dir`. Units do not nest.
- A unit is traced when an `implementedBy:` or `sourceFile:` entry of any element, resolved exactly as for `W023`, names the unit path itself, a path inside it, or a directory containing it. Remote URIs and package-registry references name no local code.
- `--all` also lists the traced units with the elements (and field) that name them; `--root <repo>` checks another repository; `--json` emits `{schemaVersion, repoRoot, baseline, untraced:[{path, unit, inBaseline}], stale:[{path, reason, why}], traced?, peers?}`, published as `crates/syscribe/schemas/trace-code.schema.json`.
- A missing `[traceCode]` table, a malformed entry, a root that is not a directory or an unreadable baseline exits `1` with a message saying how to fix it.
- The MCP server offers the same report as the read-only `trace_code` tool.
