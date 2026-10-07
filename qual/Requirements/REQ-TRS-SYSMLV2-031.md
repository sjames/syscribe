---
id: REQ-TRS-SYSMLV2-031
type: Requirement
name: Tool shall provide a sysml command inspecting SysMLv2 submodels
status: verified
reqDomain: software
verificationMethod: test
---

The tool **shall** provide `syscribe -m <root> sysml [--json]`, reporting per `sysmlSubmodel: true` package the files parsed, ingested element counts per kind, unmapped-construct counts and W540-W543 findings; with no submodel it exits 0 with a message.

**Source:** `REQ-TRS-SYSMLV2-031` (product model), `ADR-SYS-SYSMLV2-001` addendum.
