---
type: Package
name: PlanningItem
---

Requirements for the native `PlanningItem` element type: a durable, in-graph representation of
planning/tracking work (the shape a Jira epic/story/task or GitHub issue hierarchy fills today),
structurally part of the traceability graph so it can guide an LLM step-by-step through
development with the same rigor as `Requirement`/`TestCase`/`ADR`.

All requirements derive from `REQ-TRS-PLANITEM-000` and are governed by `ADR-SYS-PLANITEM-001`
(`Decisions::PlanningItemADR`): the `PI-*` id scheme and `status`/`itemType` vocabulary
(`REQ-TRS-PLANITEM-001`), single-parent hierarchy (`REQ-TRS-PLANITEM-002`), top-level
`achieves:` linkage to `Requirement`s (`REQ-TRS-PLANITEM-003`), product-line `appliesWhen:` gating
reusing the existing universal mechanism with zero new code (`REQ-TRS-PLANITEM-004`), dual-form
`evidence:` with per-entry rationale waivers (`REQ-TRS-PLANITEM-005`), and the status-graded
leaf-evidence validation rule (`REQ-TRS-PLANITEM-006`).

This phase is schema and validation only: no MCP tools, no CLI reports, no Jira/GitHub sync —
`PlanningItem` is a pure, standalone replacement for an external tracker, not an integration with
one.
