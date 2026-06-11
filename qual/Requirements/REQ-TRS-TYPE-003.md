---
id: REQ-TRS-TYPE-003
type: Requirement
title: "Tool shall recognise and validate the ConcernDef and Concern elements"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and validate the `ConcernDef` definition element and its `Concern` usage.

- **SysMLv2 mapping:** `ConcernDef` maps to `concern def` and `Concern` to `concern` (§2.1, §2.2). `ConcernDef` specializes `RequirementDef` and therefore inherits all RequirementDef fields. Type-specific fields are defined in §8.11.5: `subject:`, `stakeholders:`, `requires:`/`assume:` (constraint clauses, §8.11.2), `parameters:`. A `Concern` usage is typed by a ConcernDef (`typedBy:`).
- **Identity style:** both are **name-based (Style B)** — identity is the path-derived qualified name (basic-name grammar), with `supertype:` for specialization (ConcernDef, over ConcernDefs/RequirementDefs) and `typedBy:` for typing (Concern). They carry **no stable id** (distinct from the native `Requirement`, which does).
- **Required vs optional fields:** `type:` is required; `name:` is optional and defaults to the filename stem. All §8.11.5 type-specific fields are optional.
- **Recognition behaviour:** a file with `type: ConcernDef` or `type: Concern` is parsed as that element type (never `Unknown`); an unknown `type:` value is `E005` ([[REQ-TRS-ELEM-002]]). Base-library supertype application follows [[REQ-TRS-ELEM-003]] (§11.4).

Related: [[REQ-TRS-ELEM-001]], [[REQ-TRS-ELEM-002]], [[REQ-TRS-ELEM-003]].

**Source:** §2.1, §2.2, §8.11.5

**Acceptance criteria:** A model containing a minimal `ConcernDef` and a `Concern` typed by it parses both at their declared types (visible in `export`) and raises no `E005` for either; a sibling file with an unrecognised `type:` value raises `E005`.
