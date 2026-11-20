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
amendments:
  - date: 2026-11-20
    by: claude
    reason: Adds the trace display (worst over plans) decided with the owner
    kind: clarification
---

A `TestPlan` that declares required coverage (`REQ-TRS-PLANCOV-001`) shall be judged against it from the ingested result records (ADR-SYS-COVERAGE-001, GH #500, GH #501). The grid is every effective member (one active in at least one of the plan's configurations) × every `runsOn:` rig (any rig when it names none) × every required coordinate (the single unconditioned cell when only `minUnits:` is given). A record counts toward a cell when its `condition` includes every axis and point of the coordinate and it was made on that rig; a record of a retired or out-of-calibration rig counts for nothing. A cell has a pass when its records include a pass and none failed; with `minUnits:` it needs that many distinct named units whose records all passed, and a pass that names no unit counts for none. A flaky, blocked, not-run, inconclusive, ignored or waived result is not a pass. The plan verdict shall be `fail` when a cell has a failure, `incomplete` when no cell failed but one has no pass (also when its members would otherwise pass), and otherwise what it was without the declaration; a plan with no declaration shall be judged exactly as before. `testplan TP-X` shall list the gaps, as `<test> @ <rig> [axis=point] (missing|FAILED|n of m units)`, and carry them in `--json` under `requiredCoverage`; `audit` and the plan list shall use the same verdict; an `approved` plan with a gap shall raise the results-gated `W617` listing them. `trace <REQ>` shall show the requirement's own standing under required coverage when results are loaded: the worst, `fail` over `incomplete` over `pass`, over every non-retired plan that declares coverage and includes one of its verifying TestCases, judged on those TestCases only, with the first gaps of each plan that is not passing; a requirement no such plan covers shall show nothing more than before.
