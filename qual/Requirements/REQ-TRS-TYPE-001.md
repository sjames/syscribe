---
id: REQ-TRS-TYPE-001
type: Requirement
name: Constraint Definition and Usage
title: "Tool shall recognise and validate the ConstraintDef and Constraint elements"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and validate the `ConstraintDef` definition element and its `Constraint` usage.

- **SysMLv2 mapping:** `ConstraintDef` maps to `constraint def` and `Constraint` to `constraint` (§2.1, §2.2). A `ConstraintDef` is a boolean-valued condition classifier; a `Constraint` is a usage that applies one in context. Type-specific fields are defined in §8.10: on `ConstraintDef` — `parameters:`, `expression:`, `expressionLanguage:` (default `ocl`); on `Constraint` — `typedBy:` (the ConstraintDef), `isAsserted:` (the `assert` keyword), `isNegated:`.
- **Identity style:** both are **name-based (Style B)** — identity is the path-derived qualified name following the SysMLv2 basic-name grammar, with `supertype:` for specialization (ConstraintDef) and `typedBy:` for typing (Constraint). They carry **no stable id**.
- **Required vs optional fields:** `type:` is required; `name:` is optional and defaults to the filename stem. All §8.10 type-specific fields are optional.
- **Recognition behaviour:** a file with `type: ConstraintDef` or `type: Constraint` is parsed as that element type (never `Unknown`); an unknown `type:` value is `E005` ([[REQ-TRS-ELEM-002]]). A `ConstraintDef` with no explicit `supertype:` receives the implicit base-library supertype `Constraints::Constraint` per [[REQ-TRS-ELEM-003]] (§11.4).

Related: [[REQ-TRS-ELEM-001]], [[REQ-TRS-ELEM-002]], [[REQ-TRS-ELEM-003]].

**Source:** §2.1, §2.2, §8.10, §11.4

**Acceptance criteria:** A model containing a minimal `ConstraintDef` and a `Constraint` typed by it parses both at their declared types (visible in `export`) and raises no `E005` for either; a sibling file with an unrecognised `type:` value raises `E005`.
