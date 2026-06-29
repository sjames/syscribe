---
type: ADR
id: ADR-SYS-MCP-001
name: "Expose the model to LLMs via an MCP stdio server (syscribe mcp) with a curated, guarded-write tool surface"
status: accepted
tags:
  - mcp
  - llm
  - tooling
---

## Context

Syscribe is designed for LLM-assisted model authoring, but today an LLM can only reach the
model two ways, both lossy and token-inefficient:

1. **Filesystem access** — read and `grep` the `.md` files directly. The LLM must re-derive
   structure (qualified names, the containment/trace graph, cross-reference resolution) that
   the tool already computes, and it gets no validation feedback while editing.
2. **Shelling the CLI** — run any of the ~60 subcommands and parse human-formatted Markdown
   back into structure. Output is sized for humans, not token budgets, and round-tripping
   Markdown is brittle.

The Model Context Protocol (MCP) is the emerging standard for exposing tools, resources, and
prompts to LLM clients (Claude Code, Claude Desktop). It lets us offer the model as a small
set of structured, typed operations the client can call directly.

Several axes were evaluated:

- **Transport** — stdio (client spawns the server as a subprocess) vs. HTTP/SSE (a network
  endpoint, e.g. on the existing `syscribe-server`).
- **Capability** — read-only navigation vs. read plus the ability to mutate the model.
- **Tool surface** — mirror the full CLI 1:1 (~60 tools) vs. a small curated set.
- **Placement** — a new `syscribe mcp` subcommand in the existing CLI, a dedicated crate, or
  a route inside `syscribe-server`.
- **SDK** — the official Rust SDK `rmcp` vs. third-party crates vs. hand-rolling JSON-RPC.

## Decision

A new **`syscribe mcp`** subcommand runs an **MCP server over stdio**, built on the official
**`rmcp`** Rust SDK, exposing a **curated (~13 tool) surface** with **read and guarded-write**
capability.

- **Transport: stdio.** The LLM client spawns `syscribe mcp -m <root>` as a subprocess. This
  is the native local-tool model; no network surface, per-session model load.
- **Capability: read + guarded write.** Query/navigation/validation tools, plus
  `create_element`, `update_element`, and `move_element`. Every write tool takes a `dry_run`
  flag that **defaults to true** and returns the validation delta; a real commit refuses any
  write that breaks referential integrity (leaves a previously-resolving cross-reference
  dangling) unless explicitly overridden. The gate targets graph corruption rather than every
  validator `Error`, so incomplete drafts stay creatable (see REQ-TRS-MCP-008).
- **Curated tool surface, not a 1:1 CLI mirror.** ~13 composable tools (search, get_element,
  list_by_type, tree, neighbors, a generic graph_query, trace, impact, validate,
  validate_element, reload, and the three write tools). Token efficiency is a first-class
  design constraint: summaries by default with an opt-in `detail`/`fields` projection,
  `limit`/`offset` on list shapes, and every result carries `qname`+`id` for chaining. The
  report/render family (`export`, `plantuml`, `n2`, `matrix`, `sbom`, …) and the
  feature-model/projection commands are deliberately not exposed in the first cuts.
- **Placement: a subcommand in the `syscribe` CLI crate**, reusing the existing query,
  validation, and move internals rather than reimplementing them, and reusing the
  `syscribe-server` `ModelStore`/frontmatter round-trip patterns.
- **SDK: `rmcp`** (official Rust MCP SDK), stdio transport, macro-driven tool definitions.

## Rationale

**Transport — stdio over HTTP/SSE.** The primary use case is an LLM authoring the model on
the local disk. stdio is the simplest, dependency-free path: no bind address, no auth, no CORS,
and it is exactly how Claude Code/Desktop launch local MCP servers. An HTTP/SSE transport (for
remote/multi-client use, sharing the live-reloaded server state) remains a clean later addition
behind a flag, but adds concurrency and security surface that the first release does not need.

**Curated surface over full-CLI mirror.** Exposing ~60 tools would bloat the client's tool
list (every tool schema is tokens the model pays for on every call) and present many tools
that produce large human-formatted documents the LLM can reconstruct from the structured
primitives. A small composable set keeps the schema cheap and steers the model toward
efficient query patterns. The CLI itself remains the escape hatch for un-exposed reports.

**Guarded write over read-only or unguarded write.** Read-only would leave the LLM editing
files blind, with no validation loop. Unguarded write risks the model silently corrupting the
model. `dry_run`-by-default plus a new-error commit gate gives the LLM a safe
propose-then-commit loop and a validation delta to reason about before mutating disk.

**Subcommand over new crate / server route.** It ships in the binary users already have,
reuses the CLI's query and `move` code directly, and avoids a third crate's wiring. A server
route only makes sense once an HTTP transport is wanted.

**`rmcp` over alternatives.** It is the official SDK, macro-driven (`#[tool_router]`,
`ServerHandler`), supports stdio out of the box, and produces a single static binary with no
runtime dependency — consistent with Syscribe being a self-contained Rust tool.

## Consequences

- A new dependency (`rmcp`, `schemars`) enters the `syscribe` crate. The otherwise-synchronous
  CLI builds a local tokio runtime for the `mcp` subcommand only; `main` stays synchronous.
- The CLI's `query.rs` (and `impact.rs`/`connectivity.rs`) handlers, which currently print
  Markdown to stdout, must be refactored to expose `*_value` data producers shared by both the
  CLI and the MCP tools, so the two never diverge.
- Write tools reuse the `syscribe-server` frontmatter round-trip so unknown frontmatter keys
  and the Markdown body are preserved on edit; `create_element` adds an existence check the
  server's overwrite-style `put_element` does not have.
- The model is loaded once into a shared store and rebuilt after each successful write; a
  `reload` tool covers external edits. No filesystem watcher in the stdio process.
- Subsequent requirements built atop this decision: per-element resources (REQ-TRS-MCP-017),
  feature-model/projection tools (REQ-TRS-MCP-028..032), and a guarded read-only `run_report`
  passthrough (REQ-TRS-MCP-033) that runs an allowlisted, model-root-confined report command and
  returns its output — restoring the deliberately-excluded report/analysis family (including the
  safety/security reports `metrics`, `cyber-risk`, `safety-case`, `fmea`, `fault-tree`, `zones`,
  `conduits`, `co-analysis`, `sbom`, `behavioral-coverage`) without a dedicated tool per report.
  An HTTP/SSE transport remains future work.
