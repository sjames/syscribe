---
type: Package
name: HierarchicalProductLines
---

Requirements for hierarchical product-line composition: a top-level `Configuration` consolidated
from already-configured lower-tier product-line models, each maintained and versioned
independently (a Multiple Software Product Lines / staged-configuration structure).

All requirements derive from `REQ-TRS-HPLE-000` and are governed by `ADR-SYS-HPLE-001`
(`Decisions::HierarchicalProductLineADR`). The scope covers the `subConfigurations:` field and
its resolution, `parameterBindings:` reaching transitively through a consolidated subtree,
rejection of illegal and redundant bindings, the opt-in `--deny`-gateable open-required-parameter
check, and the hard constraint that a lower tier never references anything above it.

This is a schema-and-validation extension of existing multi-repo composition (§14) and single-model
product-line engineering — no new cross-repo addressing syntax, no new parameter-propagation
mechanism distinct from what `isRequired`/`default` already express.

The member requirements are listed by `syscribe show Requirements::HierarchicalProductLines` (generated from
this directory — not maintained here).
