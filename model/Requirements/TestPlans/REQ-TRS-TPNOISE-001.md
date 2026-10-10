---
type: Requirement
id: REQ-TRS-TPNOISE-001
name: "W616 judges plan redundancy by effective member overlap"
status: draft
reqDomain: software
reqClass: system
tags:
  - testplan
---

`W616` shall flag two TestPlans that share an identical `(configurations, scope)` pair only when their effective TestCase sets are redundant, so deliberately distinct plans that happen to share a coarse `scope` are not reported (GH #255).

## Behavior

- Plans with different `(configurations, scope)` are never compared.
- Two plans in the same bucket are redundant when both have members and either member set contains the other (including equal sets), or the Jaccard overlap of the member sets is at least 0.5.
- Plans with disjoint or lightly overlapping members (overlap below 0.5, neither a subset of the other) are not flagged.
- A plan with no effective members is not compared here (`W612` already reports it).
- The finding names the compared plan and the overlap, e.g. `shares 3 of 4 members with <file>`.
