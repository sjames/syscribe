---
type: Requirement
id: REQ-TRS-PLUGIN-000
name: "A model package can be authored in a different modeling methodology and still participate in the traceability graph"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - plugins
  - interop
---

Syscribe shall let a directory inside the model tree be authored in a different text-based
modeling methodology — plain SysMLv2 textual notation, or a project's own domain-specific
language — with the resulting elements appearing as first-class citizens of the model graph, so a
native `Requirement`/`Allocation`/`derivedFrom:`/`satisfies:`/`verifies:` can reference them
exactly as it would a hand-authored Markdown element.

## Rationale

Other teams and tools already express systems models in other notations. Multi-repository
composition (§14) brings external content into a model but only as an existence check — a peer
repo's qnames are indexed into a flat set purely to avoid false "dangling reference" warnings, with
no real graph traversal and nothing from a peer repo visible in `/api/elements` or the web UI. That
is not sufficient here: a foreign-format element must be a genuine target for every cross-reference
kind, not just pass an existence check.

## Scope

- In scope: read-only ingestion. The foreign folder stays authoritative and is edited by its own
  native tooling; Syscribe only parses it into graph elements.
- Out of scope (this requirement and its children): bidirectional editing (Syscribe's web UI or
  diagram editor writing back into the foreign format) — deferred indefinitely absent a concrete
  need.
- Which parsing technology implements the foreign-format ingestion (a sandboxed WASM plugin
  authored in TypeScript, per `ADR-SYS-PLUGIN-001`) is an architectural decision, not part of this
  requirement.
