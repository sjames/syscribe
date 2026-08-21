---
type: Requirement
id: REQ-TRS-SYSMLV2-000
name: "A model directory can hold a native SysML v2/KerML submodel that fully participates in the traceability graph"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - sysmlv2
  - interop
---

Syscribe shall let a directory inside the model tree hold real SysML v2/KerML textual files,
parsed in-process and merged into the element graph as first-class elements, with links from
SysMLv2 elements to native Syscribe `Requirement`s, from native `TestCase`s to SysMLv2 elements,
and from SysMLv2 variation points to native `FeatureDef`s, so a team's or a tool ecosystem's
existing SysML v2 content can sit inside a Syscribe model without losing traceability to it.

## Rationale

Real systems-engineering organizations already hold content in standards-track SysML v2 textual
notation, and standards-compliant tooling for it (`spec42` and others) already exists. Without this,
that content is either duplicated by hand into Markdown (losing its single source of truth and its
native tooling) or left outside the traceability graph entirely (a `Requirement`'s `derivedFrom:`,
a `TestCase`'s `verifies:`, an `Allocation` cannot target it).

## Scope

- In scope: read-only ingestion — the SysMLv2 subtree stays authoritative and is edited by its own
  native tooling; Syscribe only parses it into graph elements — plus the three cross-reference
  directions above.
- Out of scope (this requirement and its children): a writer/serializer back into `.sysml`/
  `.kerml` text, two-way round-trip authoring, and full SysML v2 static semantic validation
  (type-checking, multiplicity legality, standard-library-aware inheritance) — deferred
  indefinitely absent a concrete need.
- That this is implemented as a native, in-process Rust integration (`sysml-v2-parser`) rather
  than routed through the stdio-subprocess plugin mechanism of `ADR-SYS-PLUGIN-002` (formerly
  planned as a sandboxed WASM mechanism under the never-shipped `ADR-SYS-PLUGIN-001`) is an
  architectural decision, not part of this requirement (`ADR-SYS-SYSMLV2-001`).
