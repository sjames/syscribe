---
id: REQ-TRS-FMEA-001
type: Requirement
name: FMEA Validation Rules
title: Tool shall enforce FMEASheet and FMEAEntry validation rules E911–E914, W902–W904
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce all validation rules for the `FMEASheet` and `FMEAEntry` element types as defined in Tier 4 of the Syscribe validation specification.

| Code | Element | Condition |
|---|---|---|
| `E911` | `FMEASheet` | Missing required field (`id`, `title`, or `status`) |
| `E912` | `FMEASheet` | `id` present but does not match the `FMEA-*` pattern |
| `E913` | `FMEAEntry` | `id` present but does not match the `FM-*` pattern |
| `E914` | `FMEAEntry` | `fmeaSeverity`, `occurrence`, or `detection` field is out of range 1–10 |
| `W902` | `FMEASheet` | `FMEASheet` has no `entries` field |
| `W903` | `FMEAEntry` | `FMEAEntry` RPN > 100 but has no `recommendedAction` field |
| `W904` | `FMEAEntry` | `ref` field does not resolve to a known element |

**Source:** §T4 — FMEA (IEC 60812 / SAE J1739) validation rules.

**Acceptance criteria:** For each code, a crafted model fixture that triggers exactly that condition produces at least one finding with that code when the tool is invoked.
