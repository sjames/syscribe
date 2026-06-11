---
id: REQ-TRS-CONF-001
type: Requirement
title: Tool shall enforce E200, E201, and E209 on Configuration elements and appliesWhen fields
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce every `Configuration` element validation rule and `appliesWhen:` cross-reference rule in the following table.

| Code | Condition |
|---|---|
| `E200` | A `Configuration` element has an `id:` that does not match the `CONF-*` pattern |
| `E201` | A `Configuration` element is missing a required field (`id`, `title`, `status`, or `featureModel`) |
| `E209` | An element has `appliesWhen:` that references an element which is not a `FeatureDef` |

**Source:** §11.12

**Acceptance criteria:** For each code, a crafted model that triggers exactly that condition produces at least one finding with that code.
