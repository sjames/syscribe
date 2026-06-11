---
id: REQ-TRS-VAL-009
type: Requirement
title: Tool shall enforce E500-E503, W500-W502, and W600-W601
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce every validation rule in the following table for `Allocation` cross-references, `View` cross-references, and element documentation completeness.

| Code | Condition |
|---|---|
| `E500` | An `Allocation` element has `allocatedFrom:` that cannot be resolved to a known element |
| `E501` | An `Allocation` element has `allocatedTo:` that cannot be resolved to a known element |
| `E502` | A non-`Allocation` element has `allocatedFrom:` that cannot be resolved to a known element |
| `E503` | A non-`Allocation` element has `allocatedTo:` that cannot be resolved to a known element |
| `W500` | A `View` element has `viewpoint:` that does not resolve to a `ViewpointDef` element |
| `W501` | A `View` element has `exhibitsStates:` entries that do not resolve to known elements |
| `W502` | A `View` element has `expose:` entries that do not resolve to known elements |
| `W600` | A `PartDef` or `Part` element has an empty documentation body |
| `W601` | An `ActionDef` or `Action` element has an empty documentation body |

**Source:** §11.12

**Acceptance criteria:** For each code, a crafted model that triggers exactly that condition produces at least one finding with that code.
