---
id: REQ-TRS-SYSMLV2-068
type: Requirement
name: "Expression spellings that ingestion canonicalises compare equal in the export read-back check"
status: verified
reqDomain: software
verificationMethod: test
---

Ingestion renders every guard, condition, assigned value, assignment target and `for` sequence through one expression renderer (for example `and` becomes `&&`). `syscribe export-sysml` shall compare a native entry with its read-back after passing both through that same renderer, so a semantically equal spelling (`a and b` against `a && b`) exports as text rather than a comment. An expression the renderer cannot parse is compared verbatim and, if it differs, still degrades to a comment.

**Source:** `REQ-TRS-SYSMLV2-068` (product model), `ADR-SYS-SYSMLV2-002`.
