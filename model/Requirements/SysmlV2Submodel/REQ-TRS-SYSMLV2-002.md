---
type: Requirement
id: REQ-TRS-SYSMLV2-002
name: "Native parsing and qname-mapped merge into the graph as first-class elements"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

Syscribe shall parse every `.sysml`/`.kerml` file in a `sysmlSubmodel: true` subtree in-process via
the `sysml-v2-parser` crate — no external process, no sandbox — merge SysML v2 packages declared
across multiple files in the subtree into one namespace, and inject the result into the element
graph as ordinary origin-agnostic `RawElement`s. An element's qname is `<owning Syscribe package
qname>::<SysML v2 fully-qualified name>`, resolvable by `derivedFrom:`/`satisfies:`/`verifies:`/
`Allocation` and every other cross-reference kind exactly like a hand-authored element.

## Rationale

`RawElement`/`Resolver`/`validate_with_config` are already origin-agnostic — proven by the existing
FMEA/TARA row-explosion passes in `walker.rs` and reused unchanged by the stdio-subprocess plugin
merge (`ADR-SYS-PLUGIN-002`), which both synthesize sibling elements through the same injection
point. Reusing it here means zero special-casing anywhere in the resolver or validator for
SysMLv2-originated elements.

## Scope

- Which element kinds are actually synthesized (versus parsed-but-unmapped) is
  `REQ-TRS-SYSMLV2-007`, not this requirement.
- A duplicate qname between a SysMLv2-originated element and any other element (hand-authored,
  plugin-originated, or another SysMLv2-originated one) is `E108` — the origin-agnostic diagnostic
  that shipped with `ADR-SYS-PLUGIN-002`. No new diagnostic is introduced for this requirement
  specifically.
- This requirement covers ingestion and graph merge only; the three cross-boundary trace-link
  directions are `REQ-TRS-SYSMLV2-003`/`004`/`005`.
