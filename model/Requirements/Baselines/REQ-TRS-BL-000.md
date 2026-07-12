---
type: Requirement
id: REQ-TRS-BL-000
name: "Release-baseline management as a first-class model concern"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - baseline
  - release-management
  - traceability
---

Syscribe shall support **release baselines**: named, dated, approved, reproducible
snapshots of a defined scope of the model, so that an assessor can point to exactly what
was released or assessed and prove that it has not since changed.

A baseline shall record its provenance (name, date, approver, and the source-control state
it was sealed at), enumerate the elements in its scope, and carry a content **seal** that
allows drift to be detected later and two baselines to be compared.

## Rationale

Safety and systems assessment (ISO 26262, DO-178C, IEC 61508) is performed against a frozen
configuration. A source-control tag alone is insufficient: it is commit-granularity, does
not enumerate scope, and gives no portable, tool-checkable proof of content. Suspect-link
detection freezes an individual reviewed relationship but not a whole release. A first-class
baseline closes this gap — it makes "the release as of REL-YYYY-MM" a referenceable,
verifiable, and diffable object that travels with the model.
