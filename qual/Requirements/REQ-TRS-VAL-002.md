---
id: REQ-TRS-VAL-002
type: Requirement
title: Tool shall enforce all model-time error rules E101–E106 and E310–E315
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce every model-time error rule in the following table, as defined in §11.12 of the Syscribe format specification. Each rule **shall** be emitted after full model loading and cross-reference resolution.

| Code | Condition |
|---|---|
| `E101` | Two elements have the same `id:` value |
| `E102` | A `verifies:` reference cannot be resolved |
| `E103` | A `derivedFrom:` reference cannot be resolved |
| `E104` | A `verifies:` reference resolves to a non-`Requirement` element |
| `E105` | A `derivedFrom:` reference resolves to a non-`Requirement` element |
| `E106` | A `testFunctions[].scenario` string does not match any Gherkin scenario title |
| `E310` | `Requirement` has `derivedFrom:` but no `breakdownAdr:` |
| `E311` | `breakdownAdr:` cannot be resolved or does not resolve to an `ADR` |
| `E312` | A parent `Requirement` appears in a `satisfies:` list |
| `E313` | A `satisfies:` link connects incompatible `domain` / `reqDomain` values |
| `E314` | A deployment package `Part`/`PartDef` has no `Allocation` to a hardware element |
| `E315` | Cross-domain `supertype:` or `typedBy:` reference |

**Source:** §11.12 (model-time errors), §12

**Acceptance criteria:** For each code, a crafted multi-file model that triggers exactly that condition produces exactly one finding with that code.
