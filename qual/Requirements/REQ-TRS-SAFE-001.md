---
id: REQ-TRS-SAFE-001
type: Requirement
title: Tool shall enforce all HazardousEvent validation rules E800-E804, E833-E836, and W800
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce every HazardousEvent validation rule in the following table. Each rule **shall** be emitted when the condition is detected in a model file of type `HazardousEvent`.

| Code | Condition |
|---|---|
| `E800` | HazardousEvent is missing a required field (`id`, `title`, or `status`) |
| `E801` | `severity` is present but not in `S0`, `S1`, `S2`, `S3` |
| `E802` | `exposure` is present but not in `E0`, `E1`, `E2`, `E3`, `E4` |
| `E803` | `controllability` is present but not in `C0`, `C1`, `C2`, `C3` |
| `E804` | `id` is present but does not match the `HE-*` pattern |
| `E833` | `consequence` is present but not in `Ca`, `Cb`, `Cc`, `Cd` (IEC 61508 risk graph) |
| `E834` | `freqExposure` is present but not in `Fa`, `Fb` (IEC 61508 risk graph) |
| `E835` | `avoidance` is present but not in `Pa`, `Pb` (IEC 61508 risk graph) |
| `E836` | `demandRate` is present but not in `W1`, `W2`, `W3` (IEC 61508 risk graph) |
| `W800` | A valid `HazardousEvent` is not referenced by any `SafetyGoal.hazardousEvents` |

**Source:** §11.12 (Tier 2 safety analysis validation rules)

**Acceptance criteria:** For each code, a crafted model that triggers exactly that condition produces at least one finding with that code and no spurious findings of a different code from the same set.
