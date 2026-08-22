---
type: Requirement
id: REQ-TRS-SYSMLV2-022
name: "A SysMLv2 rendering def/rendering maps to the native RenderingDef/Rendering schema"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - views
---

A `rendering def`/`rendering` usage shall be synthesized into a native `RenderingDef`/`Rendering`
element, so a `view`'s `render` clause (`REQ-TRS-SYSMLV2-020`'s `rendering:` field) can reference a
real, browsable element rather than a dangling name.

## Rationale

Closes the last sibling of the view/viewpoint family left unmapped after `REQ-TRS-SYSMLV2-020`/
`-021`. Without this requirement, a `view`'s `rendering:` reference would always be dangling for a
SysMLv2-authored model, since the `RenderingDef`/`Rendering` it points at would never exist as a
real element.

## Scope

- Thinnest of the three view-family requirements: `RenderingDefBodyElement`/
  `RenderingUsageBodyElement` carry no field the native schema (§8.14.4: `supertype`, `features`) has
  room for beyond `doc`/`supertype`/`typedBy`. `Filter` (def only) and the nested `ViewRendering`
  variant stay unmapped — no native field, same "no native field" posture as
  `ViewDefBodyElement::Filter` (`REQ-TRS-SYSMLV2-020`).
- `RenderingUsageBodyElement::ViewUsage` — the narrow `view :>> columnView[N] { render ...; }`
  redefinition shape nested inside a `rendering`/`render` usage's own body, confirmed against real
  SysML v2 standard-library fixtures — is deliberately **not** recursed into: narrow, not
  representative of ordinary modeling, and there is no native "nested view" field to hold it.
- No recursion otherwise: neither `RenderingDefBodyElement` nor `RenderingUsageBodyElement` carries a
  variant that produces a further, separate `RawElement` besides the excluded case above.

**Acceptance criteria:** a package-wrapped `rendering def` synthesizes a real `RenderingDef`; a
`rendering` usage typed by it synthesizes a real `Rendering` with `typedBy:` set; a `view`'s
`rendering:` reference (`REQ-TRS-SYSMLV2-020`) resolves against a SysMLv2-synthesized
`RenderingDef`/`Rendering` the same as it would a hand-authored one.
