---
id: REQ-TRS-PLANITEM-002
type: Requirement
name: A PlanningItem shall accept at most one parent, forming a multi-level breakdown with graceful cycle detection
status: draft
reqDomain: software
verificationMethod: test
---

A `PlanningItem` **shall** accept an optional `parent: <PI-id-or-qname>` field naming at most one
other `PlanningItem`, forming a strict single-parent tree (not a DAG) that may extend to any depth.
A `parent:` that does not resolve, or that resolves to something other than a `PlanningItem`,
**shall** be reported as a validation error. A `parent:` chain that cycles back to itself **shall**
be reported gracefully as a validation error — never a panic or an infinite loop — regardless of
whether the cycle is two nodes or longer.

**Source:** `REQ-TRS-PLANITEM-002` (product model), `ADR-SYS-PLANITEM-001` Decision 1.

**Acceptance criteria:** a well-formed multi-level `parent:` chain validates with no
hierarchy-related errors; a `parent:` naming a non-existent element is rejected; a `parent:`
naming an element that is not a `PlanningItem` is rejected; a 2-node cycle and a 3+-node cycle are
both rejected, with no crash.
