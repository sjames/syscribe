---
type: PlanningItem
id: PI-RTH-CLOUDLOG-001
name: "Sync RTH event log to the cloud fleet-management dashboard"
status: todo
itemType: task
parent: PI-RTH-001
appliesWhen: FEAT-CLOUD-SYNC
tags:
  - rth
  - logging
---

Premium-tier-only work: upload each recorded RTH event
(`Requirements::REQ-RTH-002`) to the cloud fleet-management dashboard.
Gated on the `FEAT-CLOUD-SYNC` `FeatureDef` via the existing, universal
`appliesWhen:` mechanism — no new gating logic needed
(`REQ-TRS-PLANITEM-004`). Included in `CONF-PREMIUM-001`'s projection,
excluded from `CONF-BASE-001`'s (see the README for actual `why-active`
output). `status: todo` with no `evidence:`, which is fine.
