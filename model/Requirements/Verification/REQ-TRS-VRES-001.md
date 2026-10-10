---
type: Requirement
id: REQ-TRS-VRES-001
name: "A requirement whose active verifier fails in the ingested results is flagged at requirement level"
status: draft
reqDomain: software
reqClass: system
tags:
  - verification
  - results
---

When test results are ingested, a `Requirement` at `approved`, `implemented` or `verified` that has an active verifying `TestCase` whose ingested verdict is Fail shall be reported on the requirement itself, not only as `W010` on the TestCase (GH #257).

## Behavior

- `approved` or `implemented`: warning `W312`.
- `verified`: error `E319`, because a verified requirement with a failing verifier is a contradiction.
- Only active TestCases count. A draft or retired failing test raises nothing here.
- Only a Fail verdict triggers it. Unknown (missing, skipped, flaky) is not a failure; `W010` and the plan checks cover those.
- With no results ingested the check is silent.
- The message names the failing TestCase ids (all of them, sorted).
