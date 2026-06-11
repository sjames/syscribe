---
id: REQ-TRS-VAL-008
type: Requirement
title: Tool shall enforce safety-level, standards-compliance, and type-field validation rules E019–E022, W008, W701–W703
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce every safety-level and standards-compliance validation rule in the following table, as defined in §11.12 of the Syscribe format specification.

| Code | Condition |
|---|---|
| `E019` | `dalLevel:` value is not one of `A`, `B`, `C`, `D`, or `E` |
| `E020` | `verificationMethod:` value is not one of `test`, `inspection`, `analysis`, or `demonstration` |
| `E021` | `coverageTarget:` value is not one of `statement`, `branch`, or `MCDC` |
| `E022` | `requirementKind:` value is not one of `stakeholder`, `system`, `software`, or `hardware` |
| `W008` | A `.md` file has valid YAML frontmatter but no `type:` field — element is ignored by most commands |
| `W701` | `Requirement` with `asilLevel:` B, C, or D has no `verificationMethod:` |
| `W702` | `Requirement` with `asilLevel:` D has an active `TestCase` but none at `testLevel: L5` (HIL) |
| `W703` | Both `asilLevel:` (ISO 26262) and `dalLevel:` (DO-178C) are set on the same element |

**Source:** §11.12 (parse-time errors and warnings), ISO 26262-6 §9

**Acceptance criteria:** For each code, a crafted model file that triggers exactly that condition produces exactly one finding with that code.
