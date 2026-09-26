# mcp — run an MCP server over stdio for LLM clients

`syscribe -m <root> mcp` starts a Model Context Protocol (MCP) server that
speaks newline-delimited JSON-RPC 2.0 over **stdio**. It lets an MCP-capable LLM
client query and guard-write the Syscribe model bound at `-m`.

## SYNOPSIS
    syscribe -m <root> mcp [--read-only] [--no-watch]

## USAGE

The server runs until its stdin is closed. It is intended to be spawned by an
MCP client (an editor, an agent runtime, …), not invoked interactively.

- `--read-only` starts the server with the guarded-write tools (listed under
  **Guarded-write tools** below) hidden and rejected; the full read/query surface
  stays available.
- `--no-watch` disables the file watcher (see **Live reload** below); the model
  is then re-read only after the server's own writes or an explicit `reload`.
- The server advertises `tools`, `resources`, `prompts`, `completions`, and
  `logging` capabilities, and emits a `resources/list_changed` notification after
  any committed write, a `reload`, or an automatic reload.

## Live reload

The server watches the model root (including `.syscribe.toml` and the
`.syscribe/results.json` verdict sidecar) and every `[repos]` peer model root,
so edits made outside it (an editor save, a branch switch, another tool) are
picked up without calling `reload`. `--read-only` servers watch too.

- Bursts of file events are debounced (~250 ms). The server then fingerprints the
  model inputs (relative path, size and mtime of every `.md`, `.sysml`, `.kerml`
  and `.rhai` file, `.syscribe.toml`, `.sysmlignore`, `.syscribe/results.json`,
  and every file under a `foreignFormat:`/`annotationFormat:` package) and
  reloads only if the fingerprint changed since the loaded store was built, so
  the server's own writes, and edits to other files, cause no reload. `.git/`,
  `.syscribe/cache/` and editor swap/backup files (`*.swp`, `*~`, `.#*`,
  `4913`, `*.tmp`) are ignored.
- The new store is built in the background and swapped in, so reads are never
  blocked. After a reload the server sends the logging message
  `{"event":"reload","source":"watch","count":N}` and
  `notifications/resources/list_changed`.
- If the reload fails, or a file's frontmatter stops parsing (typically a
  half-saved file), the reload is deferred: the current model is kept, a warning
  `{"event":"reload_deferred","source":"watch",...}` is logged (and printed to
  stderr), and the next file change retries. The `reload` tool always re-reads.
- Watching is best-effort: if the watcher cannot be created (e.g. the inotify
  watch limit is exhausted), a `watch_unavailable` warning is logged once and the
  server keeps serving without it.

Every tool returns structured JSON. Element references (`ref`) are accepted as a
stable id, a qualified name, or a display name. List/grid tools accept
`limit`/`offset`. Finding codes are the canonical syscribe codes (`E***`/`W***`).

## Read tools — navigate & query

- `get_element {ref, detail?, fields?}` — one element; summary by default,
  `detail:true` adds the body + frontmatter, `fields:[..]` projects keys.
- `search {query?, type?, status?, domain?, where?, limit?, offset?}` — ranked
  search over id/qname/name and the documentation body, with filters.
- `list_by_type {type, limit?, offset?}` — every element of a type.
- `tree {ref?, depth?}` — containment subtree.
- `neighbors {ref, edges?, direction?}` — adjacent graph nodes (one hop).
- `graph_query {from, to?, edges?, direction?, depth?}` — typed-edge graph walk.
- `trace {ref, kind?}` — a requirement's verification/derivation slice.
- `impact {ref, direction?, depth?, edges?}` — change-impact reachability.
- `link_types {}` — the project's user-defined link types (`[linkTypes]` in
  `.syscribe.toml`) with their rules and instance counts; same data as
  `link-types --json`. Call it before authoring a `links:` field.
- `follow {element, link, reverse?, transitive?, depth?}` — walk one named link
  (a declared type, its inverse, or a built-in link/reverse-index name); same data
  as `follow --format json`.
- `validate {file?, severity?, limit?}` / `validate_element {ref}` — findings.
- `reload {}` — re-read the model from disk now (normally unnecessary: the
  server reloads automatically on file changes, see **Live reload**).

## Read tools — authoring helpers

- `describe_type {type?}` — a type's frontmatter schema (fields + enum domains).
- `template {type}` — a ready-to-edit frontmatter skeleton.
- `explain_finding {code}` — what a validation code means and how to fix it.
- `check_ref {ref}` — whether a reference resolves (and to what).
- `next_id {prefix}` — the next free stable id for a prefix.
- `coverage {}` — verification coverage: verified count, unverified leaves, and
  parents missing an integration test.

## Read tools — variability (feature model / projection)

- `features {feature?}` — the feature model, or one feature's card.
- `feature_check {deep?}` — feature-model validation (with optional SAT-deep).
- `configure {config}` — a Configuration's completability + forced/free features.
- `project {config}` — a variant's active elements + projected validation.
- `diff_configs {a, b}` — elements active only in A vs only in B.
- `why_active {ref, config}` — whether/why an element is active in a variant.

## Read tools — large-model overview & search

These mirror the CLI corpus commands of the same name (`--json` output).

- `stats {group_by?, where?, status?, tag?, config?}` — per-facet histograms plus
  coverage and orphan rollups; the fast first look at a large requirement set.
- `digest {where?, status?, tag?, config?, limit?, offset?}` — one compact row per
  Requirement, paginated (narrow with `stats`, then page rows here).
- `search_text {query, type?, status?, config?, limit?}` — BM25 full-text search
  over element body + name (use `search` to match identifiers).
- `summarize {scope?, depth?, no_cache?, config?}` — bottom-up per-package rollup
  with extractive one-line summaries.
- `topics {type?, top?, config?}` — distinctive per-package keywords (TF-IDF).
- `clusters {k?, type?, config?}` — topical k-means clusters across packages.

## Read tools — suspect links & release baselines

- `suspect_list {include_unbaselined?}` — suspect trace links (the `W090` set)
  and, by default, links with no baseline yet.
- `baseline_list {}` — every release Baseline (`BL-*`) with its seal metadata.
- `baseline_verify {ref?}` — recompute and check one baseline's seal (or all).
- `baseline_diff {from, to}` — element-level added / removed / changed between two
  baselines. Sealing a baseline stays a CLI/CI action (`baseline create`).

## Read tools — evidence & coverage

- `coverage_matrix {config?, status?, tag?, gaps_only?, linked_only?, limit?, offset?}`
  — the Requirement × Configuration grid + rollup (matches `matrix --json`).
- `coverage_gaps {config?, status?, class?}` — the actionable subset:
  `uncovered` / `failing` / `unverified-claim` (`W010`/`W015`/`W029`).
- `evidence {ref}` — a requirement's verification chain with ingested verdicts.

## Read tools — diagram & documentation integrity

- `lint_docs {paths, codes?}` — unresolvable references in `.md`/`.svg`
  (`W099`–`W102`), plus the advisory `W103` (a package `_index.md` hand-listing
  its own members).
- `render_diagram {ref, format?}` — a Diagram's **source** (PlantUML by default,
  or the Mermaid source) plus its `W400`–`W415` structural findings. It does not
  render an image; rendering is left to your toolchain.
- `diagram_coverage {root?, types?}` — elements referenced by no Diagram shape,
  plus shape refs that don't resolve (the `W402` set).
- `generate_view {kind, root?, format?}` — synthesise Mermaid from the model
  graph: `traceability` | `containment` | `feature` | `allocation`.

## Read tools — report passthrough

- `run_report {command, args?, format?}` — run an allowlisted, read-only report
  command (`audit`, `stats`, `digest`, `summarize`, `matrix`, `magicgrid`,
  `trade-study`, `verification-depth`, `testplan`, `metrics`, `cyber-risk`,
  `co-analysis`, `safety-case`, `behavioral-coverage`, `sbom`, `zones`,
  `conduits`, `n2`, `fmea`, `fault-tree`, `impact`, `lint-docs`) confined to the
  served model root.

## Guarded-write tools

All write tools default to `dry_run: true`: they report the validation delta of
the proposed change without touching disk. Pass `dry_run: false` to commit.

The delta (`validationDelta`) lists every finding the change introduces or
resolves: `newErrors` / `resolvedErrors` / `newWarnings` / `resolvedWarnings`.
Each error carries a `gating` flag. A commit is refused (`written: false`)
unless `SYSCRIBE_MCP_ALLOW_NEW_ERRORS=1` is set only when it introduces a
`gating: true` error — a new unresolved reference (`EREF`) or a new
user-defined-link error (`E630`–`E636`, e.g. an undeclared link type, whose
message names the declared ones). Every other new error (for example `E310`, a
derived requirement with no `breakdownAdr`) is reported with `gating: false`
and does not block, so incomplete drafts stay creatable — read `newErrors` and
fix them, or run `validate`. After a successful commit the in-memory store is
rebuilt. Writes are confined to the model root. (Hidden under `--read-only`.)

The change is validated in a copy of the model staged beside the model root (a
hidden `.syscribe-mcp-cand-*` directory in its parent, removed afterwards; the
system temp dir if the parent is not writable), so paths relative to the model
root that leave it (`style_file = "../…"`, `sourceFile: ../tests/…`) resolve as
they do in the real tree.

- `create_element {qname, type, fields?, doc?, dry_run?}`
- `update_element {ref, fields?, doc?, dry_run?}`
- `move_element {ref, dest, dry_run?}`
- `delete_element {ref, force?, dry_run?}` — refuses if other elements reference
  the target unless `force:true`.
- `apply_changes {operations:[…], dry_run?}` — an ordered create/update/move/delete
  batch applied atomically (all-or-nothing).
- `ingest_results {format?, path?, content?, dry_run?}` — parse a `cargo-json` or
  `junit` report and merge it into the `.syscribe/results.json` verdict sidecar
  (replaces the function-level verdicts, keeps any session-log ones); dry-run
  returns the verdict delta.
- `suspect_accept {source, target, dry_run?}` — baseline one reviewed trace link
  (capture the target's content hash into the source's `traceBaselines:`); a
  cleared link's `W090` is reported under `resolvedWarnings`.

## Resources & prompts

- Resources: the format spec sections (`syscribe://spec/<section>`), the project
  config (`syscribe://config`), and each element via the
  `syscribe://element/{qname}` template; element references support completion.
- Prompts: `create-model`, `create-magicgrid-model`, and the task prompts
  `add-requirement`, `break-down-requirement`, `add-testcase-for`,
  `traceability-review`.
