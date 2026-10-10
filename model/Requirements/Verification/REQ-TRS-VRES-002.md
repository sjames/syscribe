---
type: Requirement
id: REQ-TRS-VRES-002
name: "verification-depth counts only tests that did not fail or go unrun"
status: draft
reqDomain: software
reqClass: system
tags:
  - verification
  - results
---

When results are ingested, `verification-depth` shall not count a failing test, nor an automated test whose functions did not run, as a verification level, and shall report them separately (GH #257 b, c).

## Behavior

- A TestCase whose ingested verdict is Fail does not contribute its `testLevel`; its id is listed in a `Failing` column (`failing` array in JSON).
- A TestCase with `testFunctions` whose verdict is Unknown (a function skipped or missing) does not contribute its level; its id is listed as `notRun`.
- A TestCase with no `testFunctions` and no recorded scenario results (manually verified) still counts, as without results.
- Without a results sidecar the report is unchanged.
- `--min-levels` gates on the counted levels.
