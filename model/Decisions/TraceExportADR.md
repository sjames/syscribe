---
type: ADR
id: ADR-SYS-TREX-001
name: "Traceability export: one read-only JSON document of the whole requirement trace graph, projected by configuration, with a stable sort"
status: accepted
tags:
  - traceability
  - export
  - coverage
---

## Context

Coverage questions — which requirements are derived from what, which are satisfied by
architecture and verified by test cases, and which are not — are answered today one
requirement at a time (`trace`, `why`, `who-verifies`), as a Requirement × Configuration grid
(`matrix`), or by reading the whole model dump (`export`) and reconstructing the links. There is
no single machine-readable document that lists every requirement with its full trace, suitable
for a coverage check in CI or for an external tool, and none of the existing outputs can be
restricted to a product-line variant and sorted predictably.

## Decision

1. A new read-only command, `syscribe trace-export`, and a matching read-only MCP tool,
   `trace_export`, emit **one JSON document** covering every `Requirement` (native and SysML
   `RequirementDef`/`Requirement`) in the model: its identity (full qualified name, stable id,
   name, type, status, class, domain, file), its derivation (`derivedFrom` parents and derived
   children, with the breakdown ADR), its satisfaction (every satisfying element, with type and
   domain), its verification (every verifying `TestCase`, with level, status and the ingested
   verdict when a results sidecar is present), its refinements, and a computed `coverage` block
   (leaf or parent, satisfied, verified) that mirrors exactly what `W300`/`W002`/`W305` check.
   Custom link types that `extends` `satisfies`/`verifies`/`derivedFrom`/`refines` with
   `coverage = true` contribute to the same lists, as the reverse indices already do.
2. **Every reference is a full qualified name** plus the stable id when one exists; no short
   names, no file paths as identities.
3. An optional `--config <C>` projects the model onto a `Configuration` (stored id/qname or an
   ad-hoc feature set) exactly as `trace --config` and `export --config` do: inactive elements
   are omitted from every list, and the document records the configuration and its active
   features so the reader knows what it is looking at.
4. An optional `--sort directory|asc|desc` fixes the order of the top-level requirement list
   and of every nested reference list: `directory` (the default) is the walker's file order,
   `asc`/`desc` is by qualified name. The output is deterministic for a given model, option set
   and sort.
5. The document is versioned (`"version": 1`) and the schema is documented in `docs/cli` and the
   MCP help; new fields are additive.

## Rationale

One document, one schema, one computation shared by CLI and MCP keeps coverage tooling honest:
the numbers an external checker derives are the same ones `validate` enforces, because they come
from the same reverse indices. Projection by configuration reuses the existing projection engine
rather than a second filter. A declared sort makes diffs between two runs meaningful.

## Consequences

- A new `trace_export` module in `crates/syscribe` (or `syscribe-model` if the MCP server
  needs it without the CLI) built on the existing resolver reverse indices and the projection
  engine; no new validation codes.
- `export` (whole-model dump) and `trace` (one requirement) are unchanged; `docs/cli/index.md`
  gains the command, `prompts/help/trace-export.md` is added, the MCP tool list grows by one.
- Qualification: `REQ-TRS-TREX-*` mirrors with CLI test cases on a fixture model.
