---
type: Requirement
id: REQ-TRS-PLUGIN-004
name: "Two elements sharing a qualified name is a validation error, regardless of origin"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLUGIN-000]
breakdownAdr: Decisions::WasmPluginsADR
tags:
  - plugins
  - validation
---

Validation shall report `E108` when two elements — of any origin, native or plugin-emitted —
share the same qualified name, naming the file the first occurrence was seen in.

## Rationale

`Resolver::new` builds its `by_qname` index with a plain `HashMap::insert`, so a later element on
a colliding qname silently wins with no diagnostic today — unlike a duplicate stable `id`, which
already has `E101`. Foreign content is exactly the kind of external, not-locally-reviewed source
most likely to collide with a native qname, which is what surfaced this latent, pre-existing gap.
`E108` is deliberately origin-agnostic: it closes the gap for two native files that happen to
derive the same qname just as readily as for a plugin re-emitting one.
