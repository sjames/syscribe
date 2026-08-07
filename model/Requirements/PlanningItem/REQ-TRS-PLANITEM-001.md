---
type: Requirement
id: REQ-TRS-PLANITEM-001
name: "PlanningItem carries a stable PI-* id, a GitHub-referenced status, and an independent itemType per node"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLANITEM-000]
breakdownAdr: Decisions::PlanningItemADR
tags:
  - planning
---

A `PlanningItem` shall be an id-identified native element type, following the same conventions as
`Requirement`/`TestCase`/`ADR`: a stable `PI-*` id (pattern `^PI(-[A-Z0-9]{2,12})+-[0-9]{3,8}$`,
matching the shared id-scheme grammar), a required free-prose `name` label, and two additional
required/optional fields:

- **`status`** (required): one of `todo | in_progress | blocked | done` — GitHub Projects' three
  built-in defaults (`Todo`/`In Progress`/`Done`) plus `blocked`, needed so a consumer (human or LLM)
  can distinguish "not started" from "can't proceed."
- **`itemType`** (optional): one of `bug | task | feature` — exactly GitHub's own current default
  Issue Types.

## Rationale

Reusing GitHub's own vocabulary verbatim (verified against current GitHub documentation, not
assumed) means anyone already familiar with GitHub Issues/Projects needs to learn nothing new. Both
fields are plain strings, not closed Rust enums pinned to this exact set — extending either
vocabulary later is a documentation change, not a schema migration.

## Scope

- A child `PlanningItem` (one with a `parent:`, see `REQ-TRS-PLANITEM-002`) may set an `itemType`
  independent of its parent's — e.g. a `feature`-typed parent broken down into `task`-typed
  children. No inheritance or matching constraint between a parent's and a child's `itemType` is
  imposed by this requirement or any other in this series.
- An `itemType` outside the three-value set, or a `status` outside the four-value set, is a
  validation error — exact code assigned at implementation time, distinct from any existing
  `Requirement`/`TestCase` status-vocabulary code.
- Id pattern validation follows the same mechanism as every other stable-id type (`CLAUDE.md`'s "ID
  Scheme"); no new configurable-prefix behavior is introduced by this requirement.
