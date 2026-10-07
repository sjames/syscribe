---
id: REQ-TRS-SYSMLV2-074
type: Requirement
name: "A guarded succession is ingested as a guarded successionConnections entry and exported in the same form"
status: verified
reqDomain: software
verificationMethod: test
---

`first a if <guard> then b;` in an action body shall be ingested as a `successionConnections:` entry `{after: a, before: b, guard: <guard text>}` (the native, documented `guard:` sub-field), and `export-sysml` shall write a `successionConnections:` entry that has a `guard:` as `first a if <guard> then b;` whenever that statement reads back identically. The repository export ratchet budget is lowered accordingly.

**Source:** `REQ-TRS-SYSMLV2-074` (product model), `ADR-SYS-SYSMLV2-001`.
