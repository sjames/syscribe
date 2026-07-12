---
type: Requirement
id: REQ-TRS-SUS-LINKS-000
name: "Suspect trace-link detection via content baselines"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - traceability
  - suspect-links
---

Syscribe shall detect when the target of a trace link has changed since the link was last
reviewed, and surface such **suspect** links so they can be re-reviewed. Detection shall
be based on a stored content hash (a **baseline**) of the link's target, captured at
review time and recompared during validation.

## Rationale

A trace link asserts a relationship — a test verifies a requirement, a requirement is
derived from a parent, an architecture element satisfies a requirement — that was valid
only at the moment a human reviewed it. When the target subsequently changes, the
assertion may silently become stale, eroding the integrity of the V-model trace chain
that safety-critical engineering depends on.

Existing signals are insufficient: a version-control diff flags *any* edit (including
cosmetic ones) and so produces noise that reviewers learn to ignore, while the multi-repo
ref-drift checks operate at whole-repository, commit granularity and cannot see an
individual element's content change. A per-element content baseline gives a precise,
low-noise, version-control-agnostic signal that a specific reviewed relationship needs
human re-confirmation.
