---
type: Requirement
id: REQ-TRS-TESTENV-001
name: "TestEnvironment models the rig a test runs on, and runsOn ties tests and plans to it"
status: draft
reqDomain: software
reqClass: system
tags:
  - verification
---

The format shall have a `TestEnvironment` element (`TE-*`) so the rig an L4/L5 test needs (HIL, bench, climatic chamber, vehicle) is data, not prose in a TestPlan body (GH #238).

## Behavior

- Frontmatter: `id: TE-<…>-NNN`, `name`, `status` (`planned` · `available` · `retired`), optional `environmentKind` (`hil` · `bench` · `chamber` · `vehicle` · `simulation` · `other`), `capabilities:` (list of strings), `calibrationStatus` (`valid` · `expired` · `unknown`) and `calibrationDue` (a `YYYY-MM-DD` date).
- `runsOn:` (string or list) on a `TestCase` or `TestPlan` names the environment(s) it executes on; `requiresCapabilities:` (list of strings) on a `TestCase` or `TestPlan` names what it needs.
- `E893` — `id`, `name` or `status` missing, an id not `TE-*`, or a status / `environmentKind` / `calibrationStatus` outside its enum, or a malformed `calibrationDue`. `E894` — a `runsOn` entry does not resolve to a `TestEnvironment`.
- `W891` — a test or plan with `requiresCapabilities` and `runsOn` whose environments together lack a required capability (case-insensitive). `W892` — a non-draft test or plan runs on a `retired` environment or one whose `calibrationStatus` is `expired`.
