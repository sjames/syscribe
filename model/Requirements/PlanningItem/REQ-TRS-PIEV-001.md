---
type: Requirement
id: REQ-TRS-PIEV-001
name: "An active TestCase that no PlanningItem's evidence names can be warned about and is reported"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-PLANITEM-000]
breakdownAdr: Decisions::PlanningItemADR
tags:
  - planning
---

Syscribe shall be able to tell a project which `active` TestCases are recorded as no `PlanningItem`'s evidence, so work that was done and tested but never recorded against a plan is visible (GH #456).

## Behavior

- An opt-in `[planning]` table in `<model_root>/.syscribe.toml` carries the PlanningItem evidence gates. Keys are snake_case or camelCase; every key defaults to `false`, so a model without the table behaves exactly as before.
- `W322`: the `[planning]` value is not a table, a key is unknown, or a value is not `true`/`false`. The entry is ignored and the default kept.
- `[planning] require_testcase_evidence = true` raises `W319` on each `status: active` TestCase that no PlanningItem's `evidence: - ref:` names. A `ref:` names a TestCase when it resolves to it by stable id or qualified name (the resolution `E716` uses), whatever the item's status or the entry's `rationale:`.
- The existing finding baseline (`validate --baseline-file` / `--new-since`, GH #431) applies to `W319` unchanged, so a project can accept the existing gap and fail only on new ones.
- `coverage --json` (and the MCP `coverage` tool) report the same set as `testCasesWithoutPlanningEvidence`, whether or not the warning is switched on; plain `coverage` lists it.
