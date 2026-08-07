---
type: Requirement
id: REQ-TRS-PLANITEM-000
name: "Planning/tracking work is a native, in-graph element type that can guide an LLM step-by-step through development"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - planning
---

Syscribe shall represent planning/tracking work — the shape a Jira epic/story/task or a GitHub
issue hierarchy fills today — as a native `PlanningItem` element type, structurally part of the
traceability graph, so that breakdown, dependency, and completion evidence are resolvable by id/qname
the same way `Requirement`/`TestCase`/`ADR` already are, and so a human or an LLM being guided
through development has a durable, versioned record of the work graph instead of an ephemeral
todo list or an external, disconnected tracker.

## Rationale

A concrete precedent for this gap surfaced organically while building the SysMLv2 submodel feature
(`ADR-SYS-SYSMLV2-001`): a flat, session-scoped todo list drove a sequence of implement → review →
fix → verify → commit steps, repeated per requirement, under human direction. That list was
ephemeral and un-versioned. `PlanningItem` is that same shape of thing made durable and checked
into git.

## Scope

- In scope this phase: the element schema and its validation rules only.
- Out of scope (this requirement and its children): MCP tools that actively drive an LLM through a
  `PlanningItem` graph (e.g. "what's next," "mark this done"), a formal state-machine-backed status
  model, and any Jira/GitHub sync or external-tracker integration — `PlanningItem` is a pure
  replacement, not a mirror of an external system.
- Which specific fields and vocabulary implement this (id scheme, hierarchy, evidence, status) are
  architectural decisions captured in `ADR-SYS-PLANITEM-001`, not part of this requirement.
