---
type: ADR
id: ADR-SYS-STATS-001
name: "stats is a facet-histogram rollup that reuses coverage/validator indices; one document producer backs CLI and MCP; coverage is never narrowed by scoping filters"
status: accepted
tags:
  - stats
  - reporting
  - mcp
  - tooling
---

## Context

A model can hold tens of thousands of native `Requirement` elements. An LLM client
cannot ingest that many files (millions of tokens), so it needs a cheap **first hop**
that returns the *shape* of the corpus — distributions and coverage/orphan totals — in
one call, before it drills into individual elements with `get_element`. This is Tier A
of the LLM-scale scanning work (`REQ-TRS-OUT-021`).

The existing `audit` command (`REQ-TRS-OUT-013`) already rolls up a *fixed*
safety-readiness verdict with a status split, SIL/ASIL distribution, coverage and
orphans. What is missing is a **general, pivotable** histogram surface that is primarily
machine-facing and cheap to call repeatedly with different scopes.

## Decision

Add a read-only `stats` command (`REQ-TRS-OUT-021`), built requirement-first, that
aggregates the native `Requirement` population into per-facet histograms plus coverage
and orphan rollups. Resolved choices:

1. **Reuse, don't reimplement.** Coverage is `coverage::coverage_summary` (the same
   partition `coverage` and `matrix` report); parent identification uses the validator
   `derived_children` reverse index; scoping uses the existing `query` custom-field
   `--where` predicate and the tag/status filters; the `--config` lens uses
   `projection::project`. `stats` computes only the facet histograms itself.

2. **One document producer backs both surfaces.** `stats::stats_document` returns a
   `serde_json::Value`; the CLI (`stats` / `stats --json`) and the MCP `stats` tool both
   call it, so they return byte-identical JSON. The MCP tool is first-class and
   read-only (alongside `coverage` / `coverage_matrix`), and `stats` is added to the
   `run_report` allow-list.

3. **Coverage is never narrowed by the scoping filters.** `--where` / `--status` /
   `--tag` restrict the *facet and orphan* requirement set (so `total` reflects the
   filtered count), but the `coverage` rollup always reflects the whole **active** model.
   This is deliberate: the acceptance contract requires `stats` coverage to equal what
   `coverage`/`matrix` report for the same model, and filtering the requirement set out
   from under a coverage computation (which needs the full TestCase universe) would
   produce numbers that silently disagree with every other command.

4. **`--group-by` crosses one facet with the top-level package**, emitting a per-package
   histogram under `byPackage` and dropping that facet from the flat map. The valid axes
   are `status | reqDomain | silLevel | asilLevel | tags`; `package` is a facet but not a
   `group_by` axis (crossing package by package is degenerate). An unknown axis is a
   usage error (exit 1), matching the unresolvable-`--config` error.

5. **Parent exclusion mirrors `audit`.** A requirement with `derivedChildren` is
   excluded from the `unsatisfied`/`unverified` orphan sets and from `untraced` (it is
   satisfied/verified transitively and forbidden from any `satisfies:` list — §12.4 /
   `E312`), so `stats` and `audit`/`validate` agree.

## Rationale

Wrapping the existing coverage/validator/projection logic makes CLI `--json` parity the
contract and avoids the drift that a second, parallel coverage implementation would
invite. **Decision 3** is the one non-obvious call: an LLM scanning "approved software
requirements" still wants the *model's* true coverage number, not a coverage figure
recomputed over an arbitrary sub-slice that no other command would reproduce — so the
filters scope the histograms but leave coverage whole. **Decision 2** keeps the machine
and human surfaces from diverging and lets the MCP tool stay a thin adapter.

## Consequences

- `stats.rs` owns only the facet aggregation and text rendering; coverage, parent
  detection and projection are borrowed. A change to the coverage definition flows into
  `stats` automatically.
- The MCP `stats` tool carries `readOnlyHint` and returns the CLI document verbatim; a
  bad `where`/`config` is an `invalid_params` error rather than a silent empty result.
- `coverage` in the `stats` document is defined to equal the `coverage`/`matrix`
  numbers; a divergence is a test failure (`TC-TRS-OUT-021`).
- Semantic search / embeddings and per-requirement bulk export are **out of scope** here
  (later tiers); `stats` is metadata rollup only.
