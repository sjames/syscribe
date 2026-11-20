---
type: Requirement
id: REQ-TRS-PLANCOV-002
name: "A TestPlan passes only when every cell of its declared coverage has a pass"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - testplan
  - results
---

A `TestPlan` that declares required coverage (`REQ-TRS-PLANCOV-001`) shall be judged against it from the ingested result records (ADR-SYS-COVERAGE-001, GH #500, GH #501). The grid is every effective member (one active in at least one of the plan's configurations) × every `runsOn:` rig (any rig when it names none) × every required coordinate (the single unconditioned cell when only `minUnits:` is given). A record counts toward a cell when its `condition` includes every axis and point of the coordinate and it was made on that rig; a record of a retired or out-of-calibration rig counts for nothing. A cell has a pass when its records include a pass and none failed; with `minUnits:` it needs that many distinct named units whose records all passed, and a pass that names no unit counts for none. A flaky, blocked, not-run, inconclusive, ignored or waived result is not a pass. The plan verdict shall be `fail` when a cell has a failure, `incomplete` when no cell failed but one has no pass (also when its members would otherwise pass), and otherwise what it was without the declaration; a plan with no declaration shall be judged exactly as before. `testplan TP-X` shall list the gaps, as `<test> @ <rig> [axis=point] (missing|FAILED|n of m units)`, and carry them in `--json` under `requiredCoverage`; `audit` and the plan list shall use the same verdict; an `approved` plan with a gap shall raise the results-gated `W617` listing them.
