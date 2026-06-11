---
id: REQ-TRS-PROJ-003
type: Requirement
title: validate --config shall flag references from active elements to elements inactive in the variant
status: draft
reqDomain: software
verificationMethod: test
---

Under `--config C`, an element that is **active** in C may reference an element that is **inactive** in C — the reference dangles in that variant even though it resolves in the 150% model. The tool **shall** detect these "escaping references", classified by reference kind.

### Reference taxonomy

| Class | Fields | Severity when target is inactive |
|---|---|---|
| **Structural / typing** | `typedBy`, `supertype`, `subsets`, `redefines`, connection endpoints (`from`/`to`), `allocatedFrom`, `allocatedTo`, inline `feature.typedBy` | `E226` — **error** (the variant is structurally broken) |
| **Traceability** | `verifies`, `satisfies`, `derivedFrom`, `breakdownAdr`, `derivedFromSafetyGoal`, `derivedFromSecurityGoal` | `W019` — **warning** (the target simply isn't in this variant; often a sign `appliesWhen` is inconsistent) |
| **Meta** | `appliesWhen` operands (reference `FeatureDef`s) | excluded — features are the selection basis, not projected elements |

### Behaviour

- A finding is emitted only for an **active** source element whose referenced target is **inactive** in C; references between two active elements, or from an inactive source, produce nothing.
- `E226`/`W019` are emitted only in the lens (`--config` given) and only when the variability dimension is active; gateable like other findings.
- The finding **shall** name the source element, the target, the reference kind, and the configuration.

**Source:** ADR-PROJ-001.

**Acceptance criteria:** An active `Part` whose `typedBy` resolves to a `PortDef` inactive in C yields `E226`; an active `TestCase` whose `verifies` target is a Requirement inactive in C yields `W019`; an active element referencing an active element yields neither; an `appliesWhen` operand (a `FeatureDef`) never yields an escape finding; with no `--config` (or no feature model) neither code is emitted.
