---
id: REQ-TRS-SAFE-004
type: Requirement
name: CybersecurityGoal, SecurityControl, and VulnerabilityReport Validation Rules
title: Tool shall enforce all cybersecurity element validation rules E815-E824, E827-E832, W802-W804, and W807
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce every CybersecurityGoal, SecurityControl, and VulnerabilityReport validation rule in the following table.

| Code | Condition |
|---|---|
| `E815` | CybersecurityGoal is missing a required field (`id`, `title`, or `status`) |
| `E816` | CybersecurityGoal `id` does not match the `CSG-*` pattern |
| `E817` | `securityProperty` is not in `confidentiality`, `integrity`, `availability`, `authenticity` |
| `E818` | `calLevel` is not in `CAL1`, `CAL2`, `CAL3`, `CAL4` |
| `E819` | SecurityControl is missing a required field (`id`, `title`, or `status`) |
| `E820` | SecurityControl `id` does not match the `SC-*` pattern |
| `E821` | `controlType` is not in `prevention`, `detection`, `response`, `recovery` |
| `E822` | VulnerabilityReport is missing a required field (`id`, `title`, or `status`) |
| `E823` | VulnerabilityReport `id` does not match the `VR-*` pattern |
| `E824` | `cvssScore` is out of range `0.0`–`10.0` |
| `E827` | An entry in `threatScenarios` does not resolve to a `ThreatScenario` element |
| `E828` | An entry in `implementsGoals` does not resolve to a `CybersecurityGoal` element |
| `E829` | An entry in `mitigatedBy` does not resolve to a `SecurityControl` element |
| `E830` | An entry in `affectedElements` does not resolve to any known model element |
| `E831` | `derivedFromSecurityGoal` does not resolve to a `CybersecurityGoal` element |
| `E832` | `derivedFromSafetyGoal` does not resolve to a `SafetyGoal` element |
| `W802` | CybersecurityGoal is not implemented by any `SecurityControl.implementsGoals` |
| `W803` | VulnerabilityReport has `status: open` |
| `W804` | CybersecurityGoal is not referenced by any `Requirement.derivedFromSecurityGoal` |
| `W807` | A `Requirement` with `derivedFromSecurityGoal` set has no `verificationMethod` |

**Source:** §11.12 (Tier 2 security analysis validation rules)

**Acceptance criteria:** For each code, a crafted model that triggers exactly that condition produces at least one finding with that code.
