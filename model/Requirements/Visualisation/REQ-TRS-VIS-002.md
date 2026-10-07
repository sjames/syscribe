---
type: Requirement
id: REQ-TRS-VIS-002
name: "A single manifest parser honours the whole §8.16 shape/edge/layout schema and reports malformed manifests as E405/W416"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - validation
---

One parser (`vis::manifest`) shall build the IR from a `Diagram` element's `shapes:`, `edges:`
and `layout:` frontmatter, accepting every form spec §8.16.3–8.16.4 defines: the string
shorthand (`id: Qualified::Name`), the map form, an omitted `kind:` defaulting to `block`,
`parent:` nesting, `label:` on shapes and edges, and `layout:` entries with optional `w`/`h`.
A `parent:` shall produce IR nesting; a `port`-kind shape shall become a port node of its
parent.

A `shapes:`, `edges:` or `layout:` value that is present but malformed — not a map, an entry
that is neither a string nor a map, an entry missing `ref` (shapes) or `source`/`target`
(edges), a `kind:` outside the §8.16.8 vocabulary for the diagram's kind, or a `layout:` entry
without numeric `x`/`y` — shall raise error `E405` naming the entry, instead of rendering the
diagram empty. A `layout:` key that names no shape in the diagram shall raise warning `W416`.

## Rationale

The current `DiagramShape` requires `kind:` and cannot deserialise the shorthand, so a manifest
written as the spec permits silently renders nothing; `parent:` is never read, so IBD ports are
flattened into siblings; every malformed manifest falls back to an empty map with no finding.
Silent emptiness is the worst possible failure for a visual artefact.

## Scope

- `E405` and `W416` are added to `prompts/spec/validation.md` and `docs/validation/rules.md` in
  the same commit as the validator change; the catalogue tests enforce this.
- `W402`/`W403` (unresolved ref, dangling edge endpoint) keep their current meaning and are not
  duplicated by `E405`.
- The retired helpers in `crates/syscribe-model/src/diagram.rs` are deleted, not wrapped.
