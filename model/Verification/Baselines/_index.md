---
type: Package
name: Baselines
---

Test cases for release-baseline management (`REQ-TRS-BL-*`): the `Baseline` element type and
schema, the git-anchored full-content seal, scope resolution, `baseline create` +
manifest, drift detection and the validator-frozen release lifecycle, `baseline diff`, and
`baseline list` / `show` / `verify`.

Leaf test cases (L1/L2) verify individual `REQ-TRS-BL-001..011`; the integration test case
(L3) verifies the stakeholder parent `REQ-TRS-BL-000` end-to-end.
