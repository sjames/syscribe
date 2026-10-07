---
type: Requirement
id: REQ-TRS-SYSMLV2-068
name: "Expression spellings that ingestion canonicalises compare equal in the export read-back check"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

Ingestion renders every guard, condition, assigned value, assignment target and `for` sequence through one expression renderer (for example `and` becomes `&&`). `syscribe export-sysml` shall compare a native entry with its read-back after passing both through that same renderer, so a semantically equal spelling (`a and b` against `a && b`) exports as text rather than a comment. An expression the renderer cannot parse is compared verbatim and, if it differs, still degrades to a comment.
