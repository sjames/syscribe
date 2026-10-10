---
type: Requirement
id: REQ-TRS-DFA-001
name: "DependentFailureAnalysis records the independence argument of decomposed or co-hosted elements"
status: draft
reqDomain: software
reqClass: system
tags:
  - safety
---

The format shall have a `DependentFailureAnalysis` element (`DFA-*`, ISO 26262-9 clause 7) so the independence argument of an ASIL decomposition or of mixed-criticality co-hosting is a model element, not a planning item (GH #235, v1: the element, its validation and the W034 excusal; audit coverage and decomposition-rule enforcement follow).

## Behavior

- Frontmatter: `id: DFA-<…>-NNN`, `name`, `status` (`draft` · `review` · `approved` · `retired`), `analyses:` — at least two references to the elements argued independent — and optional `sharedResources:`, a list of mappings `{resource, kind, initiators, couplingFactor, mitigation}` with `kind` one of `power`, `clock`, `memory`, `bus`, `software`, `other` and `couplingFactor` a number in 0..1.
- `E890` — `id`, `name`, `status` or `analyses` missing, an id not of the `DFA-*` form, a status outside the enum, or fewer than two `analyses` entries. `E891` — an `analyses` entry resolves to no element. `E892` — a `sharedResources` entry without `resource`, with an unknown `kind` or a `couplingFactor` outside 0..1. `W890` — an `approved` DFA with a shared resource that has no `mitigation`.
- `W034` (mixed-criticality sharing without an FFI argument) is excused for a pair of elements that both appear in the `analyses` of an `approved` DFA.
