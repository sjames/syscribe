---
type: Requirement
id: REQ-TRS-TPNOISE-002
name: "W615 is reported once per plan, not once per failing member function"
status: draft
reqDomain: software
reqClass: system
tags:
  - testplan
---

`W615` (an approved TestPlan with a member whose ingested verdict is Fail or Missing) shall be a single finding per plan that lists the affected members, instead of one finding per failing or missing test function (GH #260). The root cause is still reported once, per TestCase, as `W010`.

## Behavior

- One `W615` per approved plan with at least one failing or missing member function.
- The message gives the number of affected functions and lists them as `<TestCase id> <function> (FAILED|missing)`, in a stable order, capped at 10 with `and N more`.
- A plan with no failing or missing members raises no `W615`.
