---
id: REQ-TRS-PLANITEM-001
type: Requirement
name: A PlanningItem shall carry a stable PI-* id, a required status, and an optional itemType, each drawn from a fixed vocabulary
status: draft
reqDomain: software
verificationMethod: test
---

A `PlanningItem` **shall** be an id-identified native element type following the shared stable-id
convention (`REQ`/`TC`/`ADR`-style): a `PI-*` id matching
`^PI(-[A-Z0-9]{2,12})*-[0-9]{3,8}$`, a required `name`, and a required `status` drawn from
`todo | in_progress | blocked | done`. It **shall** additionally accept an optional `itemType`
drawn from `bug | task | feature`.

An id not matching the `PI-*` pattern, a missing `id`/`name`/`status`, an out-of-vocabulary
`status`, or an out-of-vocabulary `itemType` **shall** each be reported as a validation error.

**Source:** `REQ-TRS-PLANITEM-001` (product model), `ADR-SYS-PLANITEM-001`.

**Acceptance criteria:** a `PlanningItem` with a valid `PI-*` id, `name`, `status`, and (optionally)
a valid `itemType` validates cleanly; a malformed id, a missing required field, an unrecognised
`status`, and an unrecognised `itemType` are each independently reported as errors.
