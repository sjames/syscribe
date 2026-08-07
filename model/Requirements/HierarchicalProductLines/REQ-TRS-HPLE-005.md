---
type: Requirement
id: REQ-TRS-HPLE-005
name: "A lower-tier product-line model carries zero awareness of, or reference to, anything above it"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-HPLE-000]
breakdownAdr: Decisions::HierarchicalProductLineADR
tags:
  - variability
  - multi-repo
---

A product-line model that may be consolidated by a higher tier shall require no authoring change,
no field, and no foreknowledge of that consolidation to function as a lower tier. A `FeatureDef`
parameter that needs an externally-supplied value declares this exactly as it already would in an
ordinary, single-model `Configuration` — `isRequired: true`, no `default`, `isFixed: false` — with
no field naming, or capable of naming, whoever eventually supplies that value.

## Rationale

Independent, parallel development is the entire point of a product-line-of-product-lines: a
lower-tier product line (a battery-pack line, an infotainment line) is developed once and must be
consolidatable by any number of higher-tier integrators — zero, one, or many — without modification.
A mechanism that required a lower tier to name its consolidator in advance would recreate exactly
the coupling multi-repo composition (§14) already exists to avoid, and would make a lower-tier
model's validity depend on which of possibly several unrelated integrators happens to be consuming
it at any given time.

## Scope

- `bindTo:` (the existing component→system parameter-propagation mechanism, scoped to one model)
  is explicitly **not** the mechanism for this feature and must not be repurposed for it — a
  `bindTo:` target continues to resolve purely within its own model, never across a
  `subConfigurations:` boundary, and this requirement does not change `bindTo:`'s existing behavior
  in any way.
- This requirement is architectural/structural rather than independently unit-testable in
  isolation — its observable consequence is that `REQ-TRS-HPLE-001`–`004`'s mechanisms never
  require, accept, or resolve an upward-pointing reference from a descendant, which their own tests
  cover directly.
