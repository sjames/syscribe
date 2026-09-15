---
id: REQ-TRS-PLANITEM-012
type: Requirement
name: Tool shall warn when two active PlanningItems have overlapping scope
status: draft
reqDomain: software
verificationMethod: test
---

The validator **shall** raise warning `W311` for each pair of `PlanningItem`s that are both
"active" — `status: in_progress`, **or** a non-empty `claimedBy:` (an item claimed but not yet
flipped to `in_progress`, or `in_progress` with nobody having run `claim`, both count) — and
that overlap by either:

- a shared member of `achieves:` (the same Requirement id/qname resolving identically for
  both), or
- an `evidence[].path` entry resolving to the same repo-relative path string, for both.

The rule **shall** fire once per overlapping pair, per overlap kind — not once per side — and
**shall not** fire for a pair of `PlanningItem`s that are neither `in_progress` nor claimed.

**Source:** GitHub issue #115, `ADR-SYS-PLANITEM-001` (addendum).

**Acceptance criteria:**
- Two `in_progress` PlanningItems sharing an `achieves:` Requirement raise `W311` exactly once.
- Two PlanningItems sharing an `evidence[].path` entry raise `W311`.
- A claimed-but-`todo` item still counts as active for this check.
- Two `todo`, unclaimed PlanningItems sharing scope raise nothing.
- Two PlanningItems with disjoint `achieves:`/`evidence.path` raise nothing.
