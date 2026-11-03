---
type: Requirement
id: REQ-TRS-ENVMATRIX-001
name: "matrix --by environment shows each requirement's verification per TestEnvironment"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - evidence
---

`syscribe matrix --by environment` shall show, for each requirement and each `TestEnvironment`, the verdict of the tests that verify it there, how current that evidence is, and whether the requirement is verified on every environment its tests declare (GH #437).

## Behavior

- **Rows** are the native requirements (filterable with `--status` and `--tag`; `--config` applies the Configuration lens, as for the other matrix). **Columns** are the TestEnvironments that a verifying `TestCase` declares in `runsOn:` (directly, or through a `TestPlan` that lists it), or that a recorded result names.
- A cell holds the verdict of the active verifying tests on that environment, judged on that environment's own records: `pass`, `fail`, `waived`, `gap` (an environment a test declares, or has been run on, without a passing result there — including one that is retired or out of calibration, whose results do not count), or `stale`. A cell with evidence also carries the latest record's build commit, version and date. An environment no verifying test declares or ran on is empty.
- **Stale** means the evidence is older than the model, either by the content check `validate` already makes (`W091`: the TestCase, or a requirement it verifies, changed since the records were ingested), or because the record's build commit does not contain the last git commit that changed the requirement or a verifying TestCase. A build commit the repository does not know is noted, not judged stale.
- The **roll-up** answers "verified on every environment the verifying tests declare": `verified`, `failing`, `gap`, `stale`, `waived`, or — when no environment is declared — `unscoped` (evidence exists) or `none`.
- `--gaps-only` keeps the requirements whose roll-up is `gap`, `failing` or `stale`. `--json` emits `schemaVersion: "1.0"`. The MCP `coverage_matrix` tool takes `by: "environment"` and returns the same data.
- A requirement with a passing simulator run and no target-rig run shows `pass` in one column and `gap` in the other; evidence recorded for a build older than the last change to the requirement is marked `stale`.
