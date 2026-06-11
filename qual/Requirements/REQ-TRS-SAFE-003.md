---
id: REQ-TRS-SAFE-003
type: Requirement
title: Tool shall enforce all DamageScenario and ThreatScenario validation rules E807-E814 and E826
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce every DamageScenario and ThreatScenario validation rule in the following table. Each rule **shall** be emitted when the condition is detected in a model file of type `DamageScenario` or `ThreatScenario`.

| Code | Condition |
|---|---|
| `E807` | DamageScenario is missing a required field (`id`, `title`, or `status`) |
| `E808` | DamageScenario `id` does not match the `DS-*` pattern |
| `E809` | `damageSeverity` is present but not in `severe`, `major`, `moderate`, `negligible` |
| `E810` | An entry in `impactCategories` is not in `safety`, `financial`, `operational`, `privacy` |
| `E811` | ThreatScenario is missing a required field (`id`, `title`, or `status`) |
| `E812` | ThreatScenario `id` does not match the `TS-*` pattern |
| `E813` | `attackFeasibility` is present but not in `high`, `medium`, `low`, `very_low` |
| `E814` | `attackVector` is present but not in `network`, `adjacent`, `local`, `physical` |
| `E826` | An entry in `damageScenarios` does not resolve to a `DamageScenario` element |

**Source:** §11.12 (Tier 2 safety analysis validation rules)

**Acceptance criteria:** For each code, a crafted model that triggers exactly that condition produces at least one finding with that code.
