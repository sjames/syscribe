---
id: REQ-TRS-TYPE-007
type: Requirement
title: "Tool shall recognise and validate the VerificationCase element"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and validate the `VerificationCase` usage element. (Its definition `VerificationCaseDef` is covered by [[REQ-TRS-ELEM-001]]; this requirement specifies the usage.)

- **SysMLv2 mapping:** `VerificationCase` maps to `verification` (§2.2) and is the usage of a `VerificationCaseDef` (`verification def`, §8.12.3). It shares the common case fields of §8.12.1 plus the verification-specific fields `verifies:`, `verdictExpression:`, `verdictType:` (default `VerificationCases::VerdictKind`), `returnType:`, and `typedBy:` identifying the VerificationCaseDef.
- **Identity style:** **name-based (Style B)** — identity is the path-derived qualified name (basic-name grammar), with `typedBy:` for typing. It carries **no stable id** (it is distinct from the native `TestCase`, which does).
- **Required vs optional fields:** `type:` is required; `name:` is optional and defaults to the filename stem. All case/verification fields are optional.
- **Recognition behaviour:** a file with `type: VerificationCase` is parsed as that element type (never `Unknown`); an unknown `type:` value is `E005` ([[REQ-TRS-ELEM-002]]). Base-library supertype application for the definition follows [[REQ-TRS-ELEM-003]] (§11.4).

Related: [[REQ-TRS-ELEM-001]], [[REQ-TRS-ELEM-002]], [[REQ-TRS-ELEM-003]], [[REQ-TRS-TYPE-005]].

**Source:** §2.2, §8.12.1, §8.12.3

**Acceptance criteria:** A model containing a minimal `VerificationCase` (with its `VerificationCaseDef`) parses it at its declared type (visible in `export`) and raises no `E005` for it; a sibling file with an unrecognised `type:` value raises `E005`.
