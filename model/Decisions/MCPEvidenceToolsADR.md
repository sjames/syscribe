---
type: ADR
id: ADR-SYS-MCP-002
name: "MCP evidence and diagram/doc-integrity tools wrap existing CLI logic; render_diagram returns source, not images"
status: accepted
tags:
  - mcp
  - tooling
---

## Context

The MCP server exposes a subset of the CLI. Two capability families that LLM clients and
architects repeatedly need are CLI-only today: **verification evidence** (`matrix`,
`ingest-results`) and **diagram/documentation integrity** (`lint-docs`, the diagram structural
checks `W400`–`W415`). A proposal (authored externally for syscribe maintainers) also asks for
generically-valuable new tools: coverage projections, a per-requirement evidence drill-down, a
diagram-coverage report, and a model-graph view generator.

The decisions here cover how to surface these as MCP tools without duplicating logic or
compromising the server's safety discipline, and resolve the proposal's open questions.

## Decision

Add eight MCP tools, built requirement-first (`REQ-TRS-MCP-034..042`), each wrapping existing
syscribe-model/CLI logic so behaviour matches the CLI `--json` contract:

- **Evidence:** `ingest_results` (guarded write), `coverage_matrix`, `coverage_gaps`, `evidence`.
- **Diagram/doc:** `lint_docs`, `render_diagram`, `diagram_coverage`, `generate_view`.

Resolved choices:

1. **`render_diagram` returns diagram *source*, not rendered images.** It produces PlantUML source
   (or the Mermaid source for a Mermaid diagram) plus the diagram's `W400`–`W415` structural
   findings. The server does **not** shell out to or bundle any external renderer (`mmdc`,
   PlantUML); rendering to SVG/PNG is a separate concern left to the client's toolchain. This keeps
   the tool pure, offline, and single-binary.
2. **`ingest_results` supports only the CLI's formats** — `cargo-json` and `junit` — in this scope.
3. **No coverage history** sidecar in this scope (the proposal's optional run-history is deferred).
4. **`generate_view` is read-only**, returning synthesised diagram source; it does not persist a
   `Diagram` element.

The existing `coverage` tool (integration-sufficiency lens: `unverifiedLeaves` /
`parentsMissingIntegrationTest`) is unchanged; `coverage_gaps` is a **complementary** new
projection (variant- and verdict-aware: uncovered / failing / unverified-claim).

## Rationale

Wrapping existing logic (reusing `matrix`'s `Coverage::rollup`/`cell_state`, `ingest`'s parsers +
`ResultsData::write_sidecar`, `query::tc_verdict`, the `lint-docs` scanner, `validate_with_config`
findings, the `graph` API, and the `plantuml` source generator) makes CLI `--json` parity the
contract and avoids drift. **Decision 1** is the significant deviation from the external proposal
(which wanted rendered SVG/PNG with renderer-availability errors): bundling/shelling a renderer
would add a heavy external dependency and break the offline single-binary property for marginal
benefit, and the model already generates first-class PlantUML/Mermaid source — so returning source
is both cheaper and more composable. **Decisions 2–4** keep the initial surface tight and preserve
the read-only/guarded-write discipline (`REQ-TRS-MCP-008`-style).

## Consequences

- Three CLI handlers gain extracted value producers (shared by CLI + MCP, CLI output unchanged):
  the `matrix` JSON builder, the `lint-docs` scanner (→ `Vec<Finding>`), and the `plantuml` source
  generator.
- `ingest_results` is a guarded write (dry-run default, new-error commit gate, store rebuild after
  commit) writing `.syscribe/results.json`; the other evidence tools observe its result in-session.
- Read tools are side-effect-free and carry `readOnlyHint`. All tools return structured JSON with
  canonical syscribe finding codes (`W010`, `W015`, `W029`, `W099`–`W102`, `W400`–`W415`).
- Rendering diagram source to images remains entirely outside syscribe.
