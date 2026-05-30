---
id: REQ-TRS-TARA-001
type: Requirement
name: TARA Sheet Validation Rules
title: Tool shall enforce TARASheet validation rules E940–E941, W905
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce all validation rules for the `TARASheet` element type as defined in Tier 4 of the Syscribe validation specification.

| Code | Element | Condition |
|---|---|---|
| `E940` | `TARASheet` | Missing required field (`id`, `title`, or `status`) |
| `E941` | `TARASheet` | `id` present but does not match the `TARA-*` pattern |
| `W905` | `TARASheet` | `TARASheet` has no rows in any section table (`damageTable`, `threatTable`, `goalTable`, `controlTable` all absent or empty) |

**Source:** §T4 — TARA (ISO/SAE 21434) validation rules.

**Acceptance criteria:** For each code, a crafted model fixture that triggers exactly that condition produces at least one finding with that code when the tool is invoked.
