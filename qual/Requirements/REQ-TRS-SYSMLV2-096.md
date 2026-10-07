---
id: REQ-TRS-SYSMLV2-096
type: Requirement
name: "An expression payload exports unquoted when it reads back identically"
status: verified
reqDomain: software
verificationMethod: test
---

When a `SendAction`/`AcceptAction` entry's `payload:` is not a plain (possibly qualified or dotted) name, `export-sysml` shall write it as an unquoted expression (`send new Cmd() via p;`) whenever the text parses as an expression that ingestion renders back identically, and as a quoted name only otherwise, so an ingested expression payload round-trips without quoting.

**Source:** `REQ-TRS-SYSMLV2-096` (product model), `ADR-SYS-SYSMLV2-001`.
