---
id: REQ-TRS-TYPE-012
type: Requirement
name: State and Exhibit-State Usages
title: "Tool shall recognise and validate the State and ExhibitState elements"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and validate the `State` and `ExhibitState` usage elements. (Their definition `StateDef` is covered by [[REQ-TRS-ELEM-001]]; this requirement specifies the usages.)

- **SysMLv2 mapping:** `State` maps to `state` and `ExhibitState` to `exhibit state` (§2.2). A `State` is the usage of a `StateDef` (`typedBy:`). An `ExhibitState` is a referential perform-action usage declaring that a part/occurrence exhibits a state machine defined elsewhere (§8.8.4): `isReference` is always `true` (and must not be set `false`), `typedBy:` references the StateDef, and `bindingConnections:` follow §8.4.3. The shorthand `exhibitsStates:` on a part is the inline equivalent.
- **Identity style:** both are **name-based (Style B)** — identity is the path-derived qualified name (basic-name grammar), with `typedBy:` for typing. They carry **no stable id**.
- **Required vs optional fields:** `type:` is required; `name:` is optional and defaults to the filename stem. For `ExhibitState`, `typedBy:` is required (it must reference a `StateDef`).
- **Recognition behaviour:** a file with `type: State` or `type: ExhibitState` is parsed as that element type (never `Unknown`); an unknown `type:` value is `E005` ([[REQ-TRS-ELEM-002]]). Base-library supertype application for `StateDef` (`States::StateAction`) follows [[REQ-TRS-ELEM-003]] (§11.4).

Related: [[REQ-TRS-ELEM-001]], [[REQ-TRS-ELEM-002]], [[REQ-TRS-ELEM-003]].

**Source:** §2.2, §8.8, §8.8.4

**Acceptance criteria:** A model containing a minimal `State` (typed by a `StateDef`) and an `ExhibitState` referencing that `StateDef` parses both at their declared types (visible in `export`) and raises no `E005` for either; a sibling file with an unrecognised `type:` value raises `E005`.
