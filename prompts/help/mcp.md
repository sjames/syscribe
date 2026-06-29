# mcp — run an MCP server over stdio for LLM clients

`syscribe mcp -m <model>` starts a Model Context Protocol (MCP) server that
speaks newline-delimited JSON-RPC 2.0 over **stdio**. It lets an MCP-capable LLM
client query and guard-write the Syscribe model bound at `-m`.

## Usage

    syscribe -m <model> mcp [--read-only]

The server runs until its stdin is closed. It is intended to be spawned by an
MCP client (an editor, an agent runtime, …), not invoked interactively.

- `--read-only` starts the server with the six write tools hidden and rejected;
  the full read/query surface stays available.
- The server advertises `tools`, `resources`, `prompts`, `completions`, and
  `logging` capabilities, and emits a `resources/list_changed` notification after
  any committed write or a `reload`.

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
- `validate {file?, severity?, limit?}` / `validate_element {ref}` — findings.
- `reload {}` — re-read the model from disk.

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

## Read tools — evidence & coverage

- `coverage_matrix {config?, status?, tag?, gaps_only?, linked_only?, limit?, offset?}`
  — the Requirement × Configuration grid + rollup (matches `matrix --json`).
- `coverage_gaps {config?, status?, class?}` — the actionable subset:
  `uncovered` / `failing` / `unverified-claim` (`W010`/`W015`/`W029`).
- `evidence {ref}` — a requirement's verification chain with ingested verdicts.

## Read tools — diagram & documentation integrity

- `lint_docs {paths, codes?}` — unresolvable references in `.md`/`.svg`
  (`W099`–`W102`).
- `render_diagram {ref, format?}` — a Diagram's **source** (PlantUML by default,
  or the Mermaid source) plus its `W400`–`W415` structural findings. It does not
  render an image; rendering is left to your toolchain.
- `diagram_coverage {root?, types?}` — elements referenced by no Diagram shape,
  plus shape refs that don't resolve (the `W402` set).
- `generate_view {kind, root?, format?}` — synthesise Mermaid from the model
  graph: `traceability` | `containment` | `feature` | `allocation`.

## Read tools — report passthrough

- `run_report {command, args?, format?}` — run an allowlisted, read-only report
  command (`audit`, `matrix`, `metrics`, `cyber-risk`, `safety-case`, `fmea`,
  `fault-tree`, `zones`, `conduits`, `co-analysis`, `sbom`, `n2`, `impact`,
  `behavioral-coverage`, `trade-study`, `verification-depth`, `testplan`,
  `magicgrid`, `lint-docs`) confined to the served model root.

## Guarded-write tools

All write tools default to `dry_run: true`: they report the validation delta of
the proposed change without touching disk. Pass `dry_run: false` to commit. A
commit that would introduce a *new* validation error is refused
(`written: false`) unless `SYSCRIBE_MCP_ALLOW_NEW_ERRORS=1` is set; after a
successful commit the in-memory store is rebuilt. Writes are confined to the
model root. (Hidden under `--read-only`.)

- `create_element {qname, type, fields?, doc?, dry_run?}`
- `update_element {ref, fields?, doc?, dry_run?}`
- `move_element {ref, dest, dry_run?}`
- `delete_element {ref, force?, dry_run?}` — refuses if other elements reference
  the target unless `force:true`.
- `apply_changes {operations:[…], dry_run?}` — an ordered create/update/move/delete
  batch applied atomically (all-or-nothing).
- `ingest_results {format?, path?, content?, dry_run?}` — parse a `cargo-json` or
  `junit` report into the `.syscribe/results.json` verdict sidecar; dry-run
  returns the verdict delta.

## Resources & prompts

- Resources: the format spec sections (`syscribe://spec/<section>`), the project
  config (`syscribe://config`), and each element via the
  `syscribe://element/{qname}` template; element references support completion.
- Prompts: `create-model`, `create-magicgrid-model`, and the task prompts
  `add-requirement`, `break-down-requirement`, `add-testcase-for`,
  `traceability-review`.
