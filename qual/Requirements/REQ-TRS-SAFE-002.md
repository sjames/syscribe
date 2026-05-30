---
id: REQ-TRS-SAFE-002
type: Requirement
name: SafetyGoal Validation Rules
title: Tool shall enforce all SafetyGoal validation rules E805-E806, E825, E837, W801, W805, and W806
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce every SafetyGoal validation rule in the following table. Each rule **shall** be emitted when the condition is detected in a model file of type `SafetyGoal` or when a cross-reference from a `SafetyGoal` fails to resolve.

| Code | Condition |
|---|---|
| `E805` | SafetyGoal is missing a required field (`id`, `title`, or `status`) |
| `E806` | `id` is present but does not match the `SG-*` pattern |
| `E825` | An entry in `hazardousEvents` does not resolve to a `HazardousEvent` element |
| `E837` | `plLevel` is present but not in `a`, `b`, `c`, `d`, `e` (ISO 13849-1) |
| `W801` | SafetyGoal has no integrity level field (`asilLevel`, `silLevel`, or `plLevel`) |
| `W805` | SafetyGoal is not referenced by any `Requirement.derivedFromSafetyGoal` |
| `W806` | SafetyGoal has no `hazardousEvents` field — it is not grounded in any hazard analysis |

**Source:** §11.12 (Tier 2 safety analysis validation rules)

**Acceptance criteria:** For each code, a crafted model that triggers exactly that condition produces at least one finding with that code.
