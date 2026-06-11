---
type: Package
name: ToolQual
version: "0.1"
status: draft
---

Tool Requirements Specification (TRS) for the Syscribe CLI validator and format parser.
Intended for use as qualification evidence under ISO 26262 Part 8 Clause 11 (TCL2) and IEC 61508 Part 3 Annex D.

**Scope**: The Syscribe CLI binary (`syscribe`) and the Syscribe format specification.
The web browser (`syscribe-server`) is out of scope for this TRS.

**Standard**: ISO 26262:2018 Part 8 §11 (Tool Confidence Level 2).

Each requirement in `Requirements/` carries a stable `REQ-TRS-*` identifier and a `verificationMethod` field.
Test cases (`TestCases/`) will reference these requirements via `verifies:`.
