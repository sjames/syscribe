---
type: Requirement
id: REQ-TRS-SYSMLV2-020
name: "A SysMLv2 view def/view maps to the native ViewDef/View schema — rendering, and (usage only) expose/viewpoint"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - views
---

A `view def`/`view` usage shall be synthesized into a native `ViewDef`/`View` element carrying the
same `expose:`/`viewpoint:`/`rendering:` shape a hand-authored one uses
(`model/Views/SystemArchitectureView.md`'s existing convention), so a SysMLv2-authored view
participates fully in the existing `W500`/`W502` cross-reference checks, with zero validator
changes. A `view def` synthesizes a `ViewDef`; a `view` usage synthesizes a `View`.

## Rationale

`ViewDef`/`View` already carry real, tested `viewpoint:`/`expose:` cross-reference participation for
hand-authored elements (`W500`/`W502`). Without this requirement, a SysMLv2-authored view is
invisible to the graph entirely — it cannot be linked to a viewpoint, browsed, or checked — leaving
that capability asymmetric between hand-authored and SysMLv2-authored content for no reason tied to
the feature itself.

## Scope

- `ViewDefBodyElement` (a `view def`'s own body) carries no `Expose`/`Satisfy` variant at all — the
  grammar structurally cannot carry `expose:`/`viewpoint:` on a `view def`, only on a `view` usage
  (`ViewBodyElement`, which does carry both). This lines up exactly with `W500`/`W502` already being
  scoped to `ElementType::View` only, never `ViewDef` — no tension to resolve.
- `expose:` — one entry per `Expose(ExposeMember)` member on a `view` usage, always the **plain
  `target` string as-is** (never the richer `{ref, isRecursive, filter}` map form). `ExposeMember`'s
  `target` already includes any `::*`/`::**` suffix textually, so nothing is lost; this also matches
  every real hand-authored `expose:` list in `model/` and sidesteps a pre-existing, unrelated `W502`
  inconsistency (its map-form branch reads a `ref` key, while `spec/markdown-sysml-format.md`
  §8.14.3 documents `target` as the schema key) rather than replicating or silently fixing it as a
  side effect of this requirement.
- `viewpoint:` — the `Satisfy(SatisfyViewMember)` member's `viewpoint_ref` on a `view` usage. Multiple
  `satisfy` clauses: the first one wins, matching the native field's own single-string shape.
- `rendering:` — the first `render <name> [: <Type>]` clause (`ViewRenderingUsage`) found in either a
  `view def`'s or a `view` usage's own body, preferring the referenced type's name and falling back
  to the render clause's own name when untyped. A second `render` clause is silently not represented
  — the native field is a single string.
- `ExposeMember.filter` (the BNF's optional `[ expr ]` suffix) and any braced `expose`/`satisfy` body
  content are parsed and then discarded by the vendored parser itself (`ExposeMember`/
  `SatisfyViewMember.body: ConnectBody`, a bodyless `Semicolon | Brace` enum) — a real, non-negotiable
  ceiling, not a Syscribe-side design choice.
- No recursion into a `view def`/`view` usage's own body: none of its body-element variants
  (`Doc`, `MetadataAnnotation`, `Filter`, `ViewRendering`, `Expose`, `Satisfy`) produce a further,
  separate `RawElement`. The one nested-view shape that exists in this grammar — a `RenderingUsage`
  body nesting a `view :>> columnView[N] { render ...; }` redefinition — is out of scope for this
  requirement too (see `REQ-TRS-SYSMLV2-022`).

**Acceptance criteria:** a package-wrapped `view def` with a `render` clause synthesizes a real
`ViewDef` with `rendering:` set and `expose:`/`viewpoint:` unset; a `view` usage typed by it, with
`expose`/`satisfy` clauses, synthesizes a real `View` with `expose:` as flat strings and `viewpoint:`
set; `W500`/`W502` fire on a deliberately dangling `viewpoint:`/`expose:` target exactly as they
would on hand-authored input, with no `validator.rs` changes required.
