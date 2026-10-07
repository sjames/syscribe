---
type: Requirement
id: REQ-TRS-SYSMLV2-096
name: "An expression payload exports unquoted when it reads back identically"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

When a `SendAction`/`AcceptAction` entry's `payload:` is not a plain (possibly qualified or dotted) name, `export-sysml` shall write it as an unquoted expression (`send new Cmd() via p;`) whenever the text parses as an expression that ingestion renders back identically, and as a quoted name only otherwise, so an ingested expression payload round-trips without quoting.
