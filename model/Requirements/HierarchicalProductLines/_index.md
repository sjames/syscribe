---
type: Package
name: HierarchicalProductLines
---

Requirements for hierarchical product-line composition: a top-level `Configuration` consolidated
from already-configured lower-tier product-line models, each maintained and versioned
independently (a Multiple Software Product Lines / staged-configuration structure, per
`ADR-SYS-HPLE-001`).

All requirements derive from `REQ-TRS-HPLE-000` and are governed by `ADR-SYS-HPLE-001`
(`Decisions::HierarchicalProductLineADR`): the `subConfigurations:` field and its resolution
(`REQ-TRS-HPLE-001`), `parameterBindings:` extended to reach transitively through a consolidated
subtree via ordinary qname resolution (`REQ-TRS-HPLE-002`), illegal/redundant binding rejection
(`REQ-TRS-HPLE-003`), the opt-in, `--deny`-gateable open-required-parameter completeness check
(`REQ-TRS-HPLE-004`), and the hard architectural constraint that a lower tier never carries any
awareness of, or reference to, anything above it (`REQ-TRS-HPLE-005`).

This is a schema-and-validation extension of existing multi-repo composition (§14) and single-model
product-line engineering — no new cross-repo addressing syntax, no new parameter-propagation
mechanism distinct from what `isRequired`/`default` already express.
