---
id: REQ-TRS-TYPE-005
type: Requirement
name: "Tool shall recognise and validate the CaseDef element"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and validate the `CaseDef` definition element.

- **SysMLv2 mapping:** `CaseDef` maps to `case def` (§2.1). It is the base classifier for analysis, verification, and use-case definitions. It shares the common case fields of §8.12.1: `subject:`, `actors:`, `objectives:`, `result:` (plus common action fields). Its usage counterpart is `Case` (`case`, §2.2).
- **Identity style:** **name-based (Style B)** — identity is the path-derived qualified name (basic-name grammar), with `supertype:` for specialization. It carries **no stable id**.
- **Required vs optional fields:** `type:` is required; `name:` is optional and defaults to the filename stem. All §8.12.1 case fields are optional.
- **Recognition behaviour:** a file with `type: CaseDef` is parsed as that element type (never `Unknown`); an unknown `type:` value is `E005` ([[REQ-TRS-ELEM-002]]). Base-library supertype application follows [[REQ-TRS-ELEM-003]] (§11.4).

Related: [[REQ-TRS-ELEM-001]], [[REQ-TRS-ELEM-002]], [[REQ-TRS-ELEM-003]], [[REQ-TRS-TYPE-006]], [[REQ-TRS-TYPE-007]], [[REQ-TRS-TYPE-008]].

**Source:** §2.1, §8.12.1

**Acceptance criteria:** A model containing a minimal `CaseDef` parses it at its declared type (visible in `export`) and raises no `E005` for it; a sibling file with an unrecognised `type:` value raises `E005`.
