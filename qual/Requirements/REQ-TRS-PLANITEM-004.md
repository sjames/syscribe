---
id: REQ-TRS-PLANITEM-004
type: Requirement
name: A PlanningItem shall be gateable to a product-line feature via the existing, universal appliesWhen mechanism
status: draft
reqDomain: software
verificationMethod: test
---

A `PlanningItem` **shall** accept the existing, universal `appliesWhen:` field, so that a
`PlanningItem` representing work to implement a product-line feature is included/excluded by
`--config` projection and `feature-check --deep` exactly like any other gated element, with **no**
new gating mechanism.

**Source:** `REQ-TRS-PLANITEM-004` (product model), `ADR-SYS-PLANITEM-001`.

**Acceptance criteria:** a `PlanningItem` with `appliesWhen: <FEAT-id>` is reported active under a
`Configuration` selecting that feature true and inactive under one selecting it false
(`why-active`); `feature-check --deep` reports the feature model as sound (`void model: false`,
no invalid configurations).
