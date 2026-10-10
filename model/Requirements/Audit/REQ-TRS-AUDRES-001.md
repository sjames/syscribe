---
type: Requirement
id: REQ-TRS-AUDRES-001
name: "audit reports verification results and fails on failing evidence"
status: draft
reqDomain: software
reqClass: system
tags:
  - audit
  - results
---

When test results are ingested, `audit` shall report the execution evidence and shall not return a PASS verdict while safety evidence is failing (GH #256).

## Behavior

- A "Verification results" section (`verification` in JSON) gives: active TestCases by verdict (pass / fail / unknown), SafetyGoals by safety-case verdict (supported / incomplete / failing), and TestPlans by verdict (pass / fail / incomplete / empty).
- The section is absent (`null` in JSON) when no results are ingested.
- The readiness verdict is FAIL when at least one SafetyGoal is `failing`; the reason names the goals.
- `W312` (an approved/implemented requirement whose active verifier fails) is in the default `[audit] fail_on` set. `E319` already fails as an error.
- A model that sets `[audit] fail_on` replaces the default set wholesale, as before; the failing-goal reason is not configurable.
