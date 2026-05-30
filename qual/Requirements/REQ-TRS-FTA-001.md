---
id: REQ-TRS-FTA-001
type: Requirement
name: Fault Tree Analysis Validation Rules
title: Tool shall enforce FaultTree, FaultTreeGate, and FaultTreeEvent validation rules E900–E909, W900–W901
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce all validation rules for the `FaultTree`, `FaultTreeGate`, and `FaultTreeEvent` element types as defined in Tier 4 of the Syscribe validation specification.

| Code | Element | Condition |
|---|---|---|
| `E900` | `FaultTree` | Missing required field (`id`, `title`, `status`, or `topEvent`) |
| `E901` | `FaultTree` | `id` present but does not match the `FT-*` pattern |
| `E902` | `FaultTree` | `topEvent` does not resolve to a `SafetyGoal` element |
| `E903` | `FaultTreeGate` | Missing required field (`id`, `title`, or `gateType`) |
| `E904` | `FaultTreeGate` | `id` present but does not match the `FTG-*` pattern |
| `E905` | `FaultTreeGate` | `gateType` is not one of `AND`, `OR`, `XOR`, `NOT`, `inhibit` |
| `E906` | `FaultTreeGate` | An entry in `inputs` does not resolve to a `FaultTreeGate` or `FaultTreeEvent` |
| `E907` | `FaultTreeEvent` | Missing required field (`id`, `title`, or `eventKind`) |
| `E908` | `FaultTreeEvent` | `id` present but does not match the `FTE-*` pattern |
| `E909` | `FaultTreeEvent` | `eventKind` is not one of `basic`, `undeveloped`, `house` |
| `W900` | `FaultTree` | `FaultTree` has no `FaultTreeGate` or `FaultTreeEvent` children in its directory |
| `W901` | `FaultTreeGate` | `FaultTreeGate` has no `inputs` field (dead end in the tree) |

**Source:** §T4 — Fault Tree Analysis (IEC 61025 / ISO 26262-9) validation rules.

**Acceptance criteria:** For each code, a crafted model fixture that triggers exactly that condition produces at least one finding with that code when the tool is invoked.
