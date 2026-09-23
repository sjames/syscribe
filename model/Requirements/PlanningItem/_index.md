---
type: Package
name: PlanningItem
---

Requirements for the native `PlanningItem` element type: a durable, in-graph representation of
planning/tracking work (the shape a Jira epic/story/task or GitHub issue hierarchy fills today),
structurally part of the traceability graph so it can guide an LLM step-by-step through
development with the same rigor as `Requirement`/`TestCase`/`ADR`.

All requirements derive from `REQ-TRS-PLANITEM-000` and are governed by `ADR-SYS-PLANITEM-001`
(`Decisions::PlanningItemADR`) and its addenda. The scope covers the `PI-*` id scheme and the
GitHub-derived `status`/`itemType` vocabulary, the single-parent hierarchy, top-level
`achieves:` linkage to `Requirement`s, product-line `appliesWhen:` gating, `evidence:` with
per-entry waivers and the leaf-evidence rule, `blockedBy:` dependencies, `assignedTo:` against
a `[users]` roster, the achieved-requirement verification check, and advisory `claim`/`release`
ownership markers for concurrent multi-agent work.

The member requirements are listed by `syscribe show Requirements::PlanningItem` (generated from
this directory — not maintained here).
