---
id: REQ-TRS-TYPE-008
type: Requirement
name: Use Case Usage
title: "Tool shall recognise and validate the UseCase element"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and validate the `UseCase` usage element. (Its definition `UseCaseDef` is covered by [[REQ-TRS-ELEM-001]]; this requirement specifies the usage.)

- **SysMLv2 mapping:** `UseCase` maps to `use case` (§2.2) and is the usage of a `UseCaseDef` (`use case def`, §8.12.4). It shares the common case fields of §8.12.1 plus the use-case-specific fields `actors:`, `includes:`, `extends:` (extension maps), `extensionPoints:`, and `typedBy:` identifying the UseCaseDef.
- **Identity style:** **name-based (Style B)** — identity is the path-derived qualified name (basic-name grammar), with `typedBy:` for typing. It carries **no stable id**.
- **Required vs optional fields:** `type:` is required; `name:` is optional and defaults to the filename stem. All case/use-case fields are optional.
- **Recognition behaviour:** a file with `type: UseCase` is parsed as that element type (never `Unknown`); an unknown `type:` value is `E005` ([[REQ-TRS-ELEM-002]]). Base-library supertype application for the definition follows [[REQ-TRS-ELEM-003]] (§11.4).

Related: [[REQ-TRS-ELEM-001]], [[REQ-TRS-ELEM-002]], [[REQ-TRS-ELEM-003]], [[REQ-TRS-TYPE-005]].

**Source:** §2.2, §8.12.1, §8.12.4

**Acceptance criteria:** A model containing a minimal `UseCase` (with its `UseCaseDef`) parses it at its declared type (visible in `export`) and raises no `E005` for it; a sibling file with an unrecognised `type:` value raises `E005`.
